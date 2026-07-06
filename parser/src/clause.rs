use kernel::combinator::{many1, optional, sep_by1};
use kernel::parser::{Alternative, Applicative, Functor, Parser, RightFunctor};
use tokenizer::sql_token::{SqlKeyword, SqlNumber, SqlValue};

use crate::ast::{OrderItem, SelectBody, SelectItem, SelectList, TableExpr};
use crate::expr::expr;
use crate::primitive::{TokenParser, alias, delimiter, keyword, operator, value};
use crate::table::table_expr;
use crate::token_stream::TokenStream;

/// SELECT 文を構成する句
/// 「句の集合」としてパースしてから意味的に組み立てることで、
/// FROM-first のような句順序の自由化を実現する
#[derive(Debug, Clone, PartialEq)]
enum Clause {
    Select { distinct: bool, list: SelectList },
    From(Vec<TableExpr>),
    Where(crate::ast::Expr),
    GroupBy(Vec<crate::ast::Expr>),
    Having(crate::ast::Expr),
    OrderBy(Vec<OrderItem>),
    Limit(u64),
    Offset(u64),
}

/// SELECT 文の本体: 句の並び(順序自由・各1回まで)を読み、論理評価順の正規形に組み立てる
pub fn select_body() -> TokenParser<SelectBody> {
    Parser(Box::new(|input: TokenStream| {
        let (clauses, rest) = many1(clause()).run(input)?;
        let body = assemble(clauses)?;
        Some((body, rest))
    }))
}

/// 句のリストを SelectBody に組み立てる
/// - 同じ種類の句が2回以上現れたら失敗(None)
/// - FROM は必須
/// - SELECT は省略可能(省略時は `SELECT *` とみなす: FROM-first の書き心地優先)
fn assemble(clauses: Vec<Clause>) -> Option<SelectBody> {
    let mut select: Option<(bool, SelectList)> = None;
    let mut from: Option<Vec<TableExpr>> = None;
    let mut where_clause = None;
    let mut group_by = None;
    let mut having = None;
    let mut order_by = None;
    let mut limit = None;
    let mut offset = None;

    /// 重複句なら None で失敗させるためのヘルパー
    fn set_once<T>(slot: &mut Option<T>, value: T) -> Option<()> {
        if slot.is_some() {
            return None;
        }
        *slot = Some(value);
        Some(())
    }

    for clause in clauses {
        match clause {
            Clause::Select { distinct, list } => set_once(&mut select, (distinct, list))?,
            Clause::From(tables) => set_once(&mut from, tables)?,
            Clause::Where(e) => set_once(&mut where_clause, e)?,
            Clause::GroupBy(keys) => set_once(&mut group_by, keys)?,
            Clause::Having(e) => set_once(&mut having, e)?,
            Clause::OrderBy(items) => set_once(&mut order_by, items)?,
            Clause::Limit(n) => set_once(&mut limit, n)?,
            Clause::Offset(n) => set_once(&mut offset, n)?,
        }
    }

    let (distinct, select_list) = select.unwrap_or((false, SelectList::Wildcard));
    Some(SelectBody {
        from: from?,
        where_clause,
        group_by: group_by.unwrap_or_default(),
        having,
        select: select_list,
        distinct,
        order_by: order_by.unwrap_or_default(),
        limit,
        offset,
    })
}

/// いずれかの句を1つパースする
fn clause() -> TokenParser<Clause> {
    select_clause()
        .alt(from_clause())
        .alt(where_clause())
        .alt(group_by_clause())
        .alt(having_clause())
        .alt(order_by_clause())
        .alt(limit_clause())
        .alt(offset_clause())
}

/// SELECT [DISTINCT] ( * | item, item, ... )
fn select_clause() -> TokenParser<Clause> {
    Parser(Box::new(|input: TokenStream| {
        let ((), rest) = keyword(SqlKeyword::Select).run(input)?;
        let (distinct, rest) = optional(keyword(SqlKeyword::Distinct)).run(rest)?;
        let (list, rest) = select_list().run(rest)?;
        Some((
            Clause::Select {
                distinct: distinct.is_some(),
                list,
            },
            rest,
        ))
    }))
}

/// `*` または選択項目のリスト
fn select_list() -> TokenParser<SelectList> {
    operator("*")
        .replace_with(SelectList::Wildcard)
        .alt(sep_by1(select_item(), delimiter(',')).map(SelectList::Items))
}

/// expr [[AS] alias]
fn select_item() -> TokenParser<SelectItem> {
    Parser(Box::new(|input: TokenStream| {
        let (e, rest) = expr().run(input)?;
        let (alias_name, rest) = alias().run(rest)?;
        Some((
            SelectItem {
                expr: e,
                alias: alias_name,
            },
            rest,
        ))
    }))
}

/// FROM table_expr, table_expr, ...
fn from_clause() -> TokenParser<Clause> {
    Parser(Box::new(|input: TokenStream| {
        let ((), rest) = keyword(SqlKeyword::From).run(input)?;
        let (tables, rest) = sep_by1(table_expr(), delimiter(',')).run(rest)?;
        Some((Clause::From(tables), rest))
    }))
}

/// WHERE expr
fn where_clause() -> TokenParser<Clause> {
    Parser(Box::new(|input: TokenStream| {
        let ((), rest) = keyword(SqlKeyword::Where).run(input)?;
        let (e, rest) = expr().run(rest)?;
        Some((Clause::Where(e), rest))
    }))
}

/// GROUP BY expr, expr, ...
fn group_by_clause() -> TokenParser<Clause> {
    Parser(Box::new(|input: TokenStream| {
        let ((), rest) = keyword(SqlKeyword::Group).run(input)?;
        let ((), rest) = keyword(SqlKeyword::By).run(rest)?;
        let (keys, rest) = sep_by1(expr(), delimiter(',')).run(rest)?;
        Some((Clause::GroupBy(keys), rest))
    }))
}

/// HAVING expr
fn having_clause() -> TokenParser<Clause> {
    Parser(Box::new(|input: TokenStream| {
        let ((), rest) = keyword(SqlKeyword::Having).run(input)?;
        let (e, rest) = expr().run(rest)?;
        Some((Clause::Having(e), rest))
    }))
}

/// ORDER BY order_item, order_item, ...
fn order_by_clause() -> TokenParser<Clause> {
    Parser(Box::new(|input: TokenStream| {
        let ((), rest) = keyword(SqlKeyword::Order).run(input)?;
        let ((), rest) = keyword(SqlKeyword::By).run(rest)?;
        let (items, rest) = sep_by1(order_item(), delimiter(',')).run(rest)?;
        Some((Clause::OrderBy(items), rest))
    }))
}

/// expr [ASC | DESC]
fn order_item() -> TokenParser<OrderItem> {
    Parser(Box::new(|input: TokenStream| {
        let (e, rest) = expr().run(input)?;
        let (asc, rest) = direction().run(rest)?;
        Some((OrderItem { expr: e, asc }, rest))
    }))
}

/// ソート方向: ASC | DESC | 指定なし(None)
fn direction() -> TokenParser<Option<bool>> {
    keyword(SqlKeyword::Asc)
        .replace_with(Some(true))
        .alt(keyword(SqlKeyword::Desc).replace_with(Some(false)))
        .alt(Applicative::pure(None))
}

/// LIMIT n
fn limit_clause() -> TokenParser<Clause> {
    Parser(Box::new(|input: TokenStream| {
        let ((), rest) = keyword(SqlKeyword::Limit).run(input)?;
        let (n, rest) = non_negative_integer().run(rest)?;
        Some((Clause::Limit(n), rest))
    }))
}

/// OFFSET n
fn offset_clause() -> TokenParser<Clause> {
    Parser(Box::new(|input: TokenStream| {
        let ((), rest) = keyword(SqlKeyword::Offset).run(input)?;
        let (n, rest) = non_negative_integer().run(rest)?;
        Some((Clause::Offset(n), rest))
    }))
}

/// 0以上の整数リテラル
fn non_negative_integer() -> TokenParser<u64> {
    Parser(Box::new(|input: TokenStream| {
        let (v, rest) = value().run(input)?;
        match v {
            SqlValue::Number(SqlNumber::Integer(n)) if n >= 0 => Some((n as u64, rest)),
            _ => None,
        }
    }))
}
