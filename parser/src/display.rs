//! AST を SQL 風の文字列に整形する(可視化ノードでの条件式表示用)
//!
//! AST は括弧を保持しないため、表示時に「再パースしたら同じ木になる」ことを
//! 優先順位の比較で保証する(必要な位置にだけ括弧を復元する)。
//! サブクエリは `(…)` に省略してノードをコンパクトに保つ。

use std::fmt;

use tokenizer::sql_token::{SqlNumber, SqlValue};

use crate::ast::{BinaryOp, Expr, FunctionArgs, ObjectName, UnaryOp};

impl fmt::Display for ObjectName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}

/// SqlValue は tokenizer の型なのでここでは関数として整形する
fn value_to_string(value: &SqlValue) -> String {
    match value {
        SqlValue::Number(SqlNumber::Integer(n)) => n.to_string(),
        SqlValue::Number(SqlNumber::Float(x)) => x.to_string(),
        SqlValue::String(s) => format!("'{s}'"),
        SqlValue::Boolean(true) => "TRUE".to_string(),
        SqlValue::Boolean(false) => "FALSE".to_string(),
        SqlValue::Null => "NULL".to_string(),
    }
}

/// 表示用の優先度(パーサーの層と同じ並び。大きいほど強く結合する)
const PREC_OR: u8 = 1;
const PREC_AND: u8 = 2;
const PREC_NOT: u8 = 3;
const PREC_COMPARISON: u8 = 4;
const PREC_ADDITIVE: u8 = 5;
const PREC_MULTIPLICATIVE: u8 = 6;
const PREC_UNARY: u8 = 7;
const PREC_ATOM: u8 = 8;

impl BinaryOp {
    fn symbol(&self) -> &'static str {
        match self {
            BinaryOp::Or => "OR",
            BinaryOp::And => "AND",
            BinaryOp::Eq => "=",
            BinaryOp::NotEq => "<>",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::LtEq => "<=",
            BinaryOp::GtEq => ">=",
            BinaryOp::Plus => "+",
            BinaryOp::Minus => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => PREC_OR,
            BinaryOp::And => PREC_AND,
            BinaryOp::Eq
            | BinaryOp::NotEq
            | BinaryOp::Lt
            | BinaryOp::Gt
            | BinaryOp::LtEq
            | BinaryOp::GtEq => PREC_COMPARISON,
            BinaryOp::Plus | BinaryOp::Minus => PREC_ADDITIVE,
            BinaryOp::Multiply | BinaryOp::Divide => PREC_MULTIPLICATIVE,
        }
    }
}

/// 式そのものの優先度
fn precedence(expr: &Expr) -> u8 {
    match expr {
        // EXISTS (…) や関数呼び出しは括弧で自己完結しているので原子扱い
        Expr::Value(_) | Expr::Column(_) | Expr::Function { .. } | Expr::Exists { .. } => PREC_ATOM,
        Expr::Unary {
            op: UnaryOp::Minus, ..
        } => PREC_UNARY,
        Expr::Unary {
            op: UnaryOp::Not, ..
        } => PREC_NOT,
        Expr::Binary { op, .. } => op.precedence(),
        Expr::IsNull { .. }
        | Expr::InList { .. }
        | Expr::InSubquery { .. }
        | Expr::Like { .. }
        | Expr::Between { .. } => PREC_COMPARISON,
    }
}

/// min_precedence の位置に置けない(弱い)式なら括弧で包む
fn operand(expr: &Expr, min_precedence: u8) -> String {
    if precedence(expr) < min_precedence {
        format!("({expr})")
    } else {
        expr.to_string()
    }
}

fn not_str(negated: bool) -> &'static str {
    if negated { "NOT " } else { "" }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Value(v) => write!(f, "{}", value_to_string(v)),
            Expr::Column(name) => write!(f, "{name}"),
            Expr::Unary {
                op: UnaryOp::Not,
                expr,
            } => write!(f, "NOT {}", operand(expr, PREC_NOT)),
            Expr::Unary {
                op: UnaryOp::Minus,
                expr,
            } => {
                // `--1` は行コメントとして再解釈されてしまうので、負数の入れ子は必ず括弧で包む
                if matches!(
                    expr.as_ref(),
                    Expr::Unary {
                        op: UnaryOp::Minus,
                        ..
                    }
                ) {
                    write!(f, "-({expr})")
                } else {
                    write!(f, "-{}", operand(expr, PREC_UNARY))
                }
            }
            Expr::Binary { left, op, right } => {
                let p = op.precedence();
                // 比較は連鎖できない(a = b = c は不可)ので左辺も一段強い優先度を要求する。
                // その他は左結合なので、右辺だけ一段強い優先度を要求して木の形を保存する
                let left_min = if p == PREC_COMPARISON { p + 1 } else { p };
                write!(
                    f,
                    "{} {} {}",
                    operand(left, left_min),
                    op.symbol(),
                    operand(right, p + 1)
                )
            }
            // IS / IN / LIKE / BETWEEN の対象はパーサー上 additive 層なので、
            // それより弱い式(OR / AND / 比較)は括弧が必要
            Expr::IsNull { expr, negated } => {
                write!(f, "{} IS {}NULL", operand(expr, PREC_ADDITIVE), not_str(*negated))
            }
            Expr::InList {
                expr,
                list,
                negated,
            } => {
                let items: Vec<String> = list.iter().map(|e| e.to_string()).collect();
                write!(
                    f,
                    "{} {}IN ({})",
                    operand(expr, PREC_ADDITIVE),
                    not_str(*negated),
                    items.join(", ")
                )
            }
            Expr::InSubquery { expr, negated, .. } => {
                write!(f, "{} {}IN (…)", operand(expr, PREC_ADDITIVE), not_str(*negated))
            }
            Expr::Like {
                expr,
                pattern,
                negated,
            } => {
                write!(
                    f,
                    "{} {}LIKE {}",
                    operand(expr, PREC_ADDITIVE),
                    not_str(*negated),
                    operand(pattern, PREC_ADDITIVE)
                )
            }
            Expr::Between {
                expr,
                low,
                high,
                negated,
            } => {
                write!(
                    f,
                    "{} {}BETWEEN {} AND {}",
                    operand(expr, PREC_ADDITIVE),
                    not_str(*negated),
                    operand(low, PREC_ADDITIVE),
                    operand(high, PREC_ADDITIVE)
                )
            }
            Expr::Exists { .. } => write!(f, "EXISTS (…)"),
            Expr::Function { name, args } => match args {
                FunctionArgs::Wildcard => write!(f, "{name}(*)"),
                FunctionArgs::List(list) => {
                    let items: Vec<String> = list.iter().map(|e| e.to_string()).collect();
                    write!(f, "{name}({})", items.join(", "))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;

    fn where_display(sql: &str) -> String {
        parse(sql).unwrap().body.where_clause.unwrap().to_string()
    }

    /// 表示した文字列を再パースしても同じ AST になること(表示の往復保証)
    fn assert_roundtrip(sql: &str) {
        let original = parse(sql).unwrap().body.where_clause.unwrap();
        let redisplayed = parse(&format!("FROM t WHERE {original}"))
            .unwrap()
            .body
            .where_clause
            .unwrap();
        assert_eq!(original, redisplayed, "roundtrip failed for: {original}");
    }

    #[test]
    fn displays_expressions() {
        assert_eq!(
            where_display("FROM t WHERE a + b * 2 >= c"),
            "a + b * 2 >= c"
        );
        // 優先順位が変わる括弧は保存される
        assert_eq!(where_display("FROM t WHERE (a + b) * 2 = c"), "(a + b) * 2 = c");
        assert_eq!(
            where_display("FROM t WHERE u.id = o.user_id AND NOT deleted"),
            "u.id = o.user_id AND NOT deleted"
        );
        assert_eq!(
            where_display("FROM t WHERE name NOT LIKE 'x%' OR age IS NULL"),
            "name NOT LIKE 'x%' OR age IS NULL"
        );
        assert_eq!(
            where_display("FROM t WHERE id IN (SELECT id FROM s)"),
            "id IN (…)"
        );
        assert_eq!(where_display("FROM t WHERE count(*) > 3"), "count(*) > 3");
    }

    #[test]
    fn restores_parens_needed_for_precedence() {
        // NOT / 単項マイナスの中の弱い式
        assert_eq!(where_display("FROM t WHERE NOT (a OR b)"), "NOT (a OR b)");
        assert_eq!(where_display("FROM t WHERE -(a + b) < 0"), "-(a + b) < 0");
        // IS NULL の対象が弱い式
        assert_eq!(
            where_display("FROM t WHERE (a OR b) IS NULL"),
            "(a OR b) IS NULL"
        );
        // 左結合の右側に同じ優先度の木
        assert_eq!(where_display("FROM t WHERE a - (b - c) = 0"), "a - (b - c) = 0");
        // 入れ子の負数は `--`(コメント)にならない
        assert_eq!(where_display("FROM t WHERE -(-1) = 1"), "-(-1) = 1");
    }

    #[test]
    fn display_roundtrips_to_same_ast() {
        for sql in [
            "FROM t WHERE NOT (a OR b)",
            "FROM t WHERE -(a + b) * 2 < c / (d - 1)",
            "FROM t WHERE (a OR b) IS NOT NULL",
            "FROM t WHERE a - (b - c) = 0 AND (x OR y) AND NOT z",
            "FROM t WHERE (a + b) BETWEEN c AND d + 1",
            "FROM t WHERE -(-1) = 1",
        ] {
            assert_roundtrip(sql);
        }
    }
}
