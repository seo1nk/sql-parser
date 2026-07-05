use kernel::combinator::{optional, sep_by1};
use kernel::parser::{Functor, Parser};
use tokenizer::sql_token::{SqlKeyword, SqlValue};

use crate::ast::{BinaryOp, Expr, FunctionArgs, ObjectName, UnaryOp};
use crate::primitive::{TokenParser, delimiter, identifier, keyword, operator, value};
use crate::query::query;
use crate::token_stream::TokenStream;

/// 式パーサーの入口。優先順位は低い方から
/// OR < AND < NOT < 比較(=, <>, IS NULL, IN, LIKE, BETWEEN) < 加減 < 乗除 < 単項マイナス・原子式
pub fn expr() -> TokenParser<Expr> {
    or_expr()
}

fn binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
    Expr::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }
}

/// 左結合の二項演算の並び `operand (op operand)*` をパースする(Haskell の chainl1)
/// ops の並び順に演算子を試すので、`<=` を `<` より先に置くこと
fn binary_chain(
    operand: fn() -> TokenParser<Expr>,
    ops: &'static [(&'static str, BinaryOp)],
) -> TokenParser<Expr> {
    Parser(Box::new(move |input: TokenStream| {
        let (mut left, mut rest) = operand().run(input)?;
        'chain: loop {
            for (symbol, op) in ops {
                if let Some(((), next)) = operator(symbol).run(rest.clone()) {
                    let (right, next) = operand().run(next)?;
                    left = binary(left, *op, right);
                    rest = next;
                    continue 'chain;
                }
            }
            break;
        }
        Some((left, rest))
    }))
}

/// OR(優先度最低)
fn or_expr() -> TokenParser<Expr> {
    Parser(Box::new(|input: TokenStream| {
        let (mut left, mut rest) = and_expr().run(input)?;
        while let Some(((), next)) = keyword(SqlKeyword::Or).run(rest.clone()) {
            let (right, next) = and_expr().run(next)?;
            left = binary(left, BinaryOp::Or, right);
            rest = next;
        }
        Some((left, rest))
    }))
}

/// AND
fn and_expr() -> TokenParser<Expr> {
    Parser(Box::new(|input: TokenStream| {
        let (mut left, mut rest) = not_expr().run(input)?;
        while let Some(((), next)) = keyword(SqlKeyword::And).run(rest.clone()) {
            let (right, next) = not_expr().run(next)?;
            left = binary(left, BinaryOp::And, right);
            rest = next;
        }
        Some((left, rest))
    }))
}

/// NOT(単項)
fn not_expr() -> TokenParser<Expr> {
    Parser(Box::new(|input: TokenStream| {
        if let Some(((), rest)) = keyword(SqlKeyword::Not).run(input.clone()) {
            let (inner, rest) = not_expr().run(rest)?;
            return Some((
                Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(inner),
                },
                rest,
            ));
        }
        comparison().run(input)
    }))
}

/// 比較演算子(2文字の演算子を1文字より先に試す)
const COMPARISON_OPS: [(&str, BinaryOp); 7] = [
    ("<=", BinaryOp::LtEq),
    (">=", BinaryOp::GtEq),
    ("<>", BinaryOp::NotEq),
    ("!=", BinaryOp::NotEq),
    ("=", BinaryOp::Eq),
    ("<", BinaryOp::Lt),
    (">", BinaryOp::Gt),
];

/// 比較・述語(IS NULL / IN / LIKE / BETWEEN)。連鎖しない(a = b = c は不可)
fn comparison() -> TokenParser<Expr> {
    Parser(Box::new(|input: TokenStream| {
        let (left, rest) = additive().run(input)?;

        // 比較演算子
        for (symbol, op) in &COMPARISON_OPS {
            if let Some(((), next)) = operator(symbol).run(rest.clone()) {
                let (right, next) = additive().run(next)?;
                return Some((binary(left, *op, right), next));
            }
        }

        // IS [NOT] NULL
        if let Some(((), next)) = keyword(SqlKeyword::Is).run(rest.clone()) {
            let (not, next) = optional(keyword(SqlKeyword::Not)).run(next)?;
            let (v, next) = value().run(next)?;
            if v != SqlValue::Null {
                return None;
            }
            return Some((
                Expr::IsNull {
                    expr: Box::new(left),
                    negated: not.is_some(),
                },
                next,
            ));
        }

        // [NOT] IN / LIKE / BETWEEN
        let (not, after_not) = optional(keyword(SqlKeyword::Not)).run(rest.clone())?;
        let negated = not.is_some();

        if let Some(((), next)) = keyword(SqlKeyword::In).run(after_not.clone()) {
            let ((), next) = delimiter('(').run(next)?;
            // サブクエリを先に試し、失敗したら式リスト
            if let Some((q, next)) = query().run(next.clone()) {
                let ((), next) = delimiter(')').run(next)?;
                return Some((
                    Expr::InSubquery {
                        expr: Box::new(left),
                        query: Box::new(q),
                        negated,
                    },
                    next,
                ));
            }
            let (list, next) = sep_by1(expr(), delimiter(',')).run(next)?;
            let ((), next) = delimiter(')').run(next)?;
            return Some((
                Expr::InList {
                    expr: Box::new(left),
                    list,
                    negated,
                },
                next,
            ));
        }

        if let Some(((), next)) = keyword(SqlKeyword::Like).run(after_not.clone()) {
            let (pattern, next) = additive().run(next)?;
            return Some((
                Expr::Like {
                    expr: Box::new(left),
                    pattern: Box::new(pattern),
                    negated,
                },
                next,
            ));
        }

        if let Some(((), next)) = keyword(SqlKeyword::Between).run(after_not) {
            // low / high は AND を含まない層(additive)で読む
            let (low, next) = additive().run(next)?;
            let ((), next) = keyword(SqlKeyword::And).run(next)?;
            let (high, next) = additive().run(next)?;
            return Some((
                Expr::Between {
                    expr: Box::new(left),
                    low: Box::new(low),
                    high: Box::new(high),
                    negated,
                },
                next,
            ));
        }

        // NOT の後に IN/LIKE/BETWEEN が続かなかった場合、NOT を消費してはいけないので
        // rest(NOT を読む前)から続行する
        Some((left, rest))
    }))
}

/// 加減算
fn additive() -> TokenParser<Expr> {
    binary_chain(
        multiplicative,
        &[("+", BinaryOp::Plus), ("-", BinaryOp::Minus)],
    )
}

/// 乗除算
fn multiplicative() -> TokenParser<Expr> {
    binary_chain(
        unary,
        &[("*", BinaryOp::Multiply), ("/", BinaryOp::Divide)],
    )
}

/// 単項マイナス
fn unary() -> TokenParser<Expr> {
    Parser(Box::new(|input: TokenStream| {
        if let Some(((), rest)) = operator("-").run(input.clone()) {
            let (inner, rest) = unary().run(rest)?;
            return Some((
                Expr::Unary {
                    op: UnaryOp::Minus,
                    expr: Box::new(inner),
                },
                rest,
            ));
        }
        primary().run(input)
    }))
}

/// 原子式: リテラル / EXISTS (query) / (expr) / 関数呼び出し / 列参照
fn primary() -> TokenParser<Expr> {
    Parser(Box::new(|input: TokenStream| {
        // リテラル
        if let Some((v, rest)) = value().run(input.clone()) {
            return Some((Expr::Value(v), rest));
        }

        // EXISTS ( query )
        if let Some(((), rest)) = keyword(SqlKeyword::Exists).run(input.clone()) {
            let ((), rest) = delimiter('(').run(rest)?;
            let (q, rest) = query().run(rest)?;
            let ((), rest) = delimiter(')').run(rest)?;
            return Some((
                Expr::Exists {
                    query: Box::new(q),
                },
                rest,
            ));
        }

        // ( expr ) — 括弧は木構造に現れる必要がないので中身をそのまま返す
        if let Some(((), rest)) = delimiter('(').run(input.clone()) {
            let (inner, rest) = expr().run(rest)?;
            let ((), rest) = delimiter(')').run(rest)?;
            return Some((inner, rest));
        }

        // 関数呼び出し or 列参照
        let (name, rest) = object_name().run(input)?;
        if name.0.len() == 1
            && let Some(((), after_paren)) = delimiter('(').run(rest.clone()) {
                let function_name = name.0[0].clone();
                // count(*)
                if let Some(((), next)) = operator("*").run(after_paren.clone()) {
                    let ((), next) = delimiter(')').run(next)?;
                    return Some((
                        Expr::Function {
                            name: function_name,
                            args: FunctionArgs::Wildcard,
                        },
                        next,
                    ));
                }
                // 引数なし f()
                if let Some(((), next)) = delimiter(')').run(after_paren.clone()) {
                    return Some((
                        Expr::Function {
                            name: function_name,
                            args: FunctionArgs::List(vec![]),
                        },
                        next,
                    ));
                }
                // f(a, b, ...)
                let (args, next) = sep_by1(expr(), delimiter(',')).run(after_paren)?;
                let ((), next) = delimiter(')').run(next)?;
                return Some((
                    Expr::Function {
                        name: function_name,
                        args: FunctionArgs::List(args),
                    },
                    next,
                ));
            }
        Some((Expr::Column(name), rest))
    }))
}

/// `users` や `u.id` のような(`.` で修飾されうる)名前
pub fn object_name() -> TokenParser<ObjectName> {
    sep_by1(identifier(), delimiter('.')).map(ObjectName)
}
