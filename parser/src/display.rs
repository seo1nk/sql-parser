//! AST を SQL 風の文字列に整形する(可視化ノードでの条件式表示用)
//! サブクエリは `(…)` に省略してノードをコンパクトに保つ

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

    /// 表示時に括弧が必要かを判断するための優先度(大きいほど強い)
    fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Or => 1,
            BinaryOp::And => 2,
            BinaryOp::Eq
            | BinaryOp::NotEq
            | BinaryOp::Lt
            | BinaryOp::Gt
            | BinaryOp::LtEq
            | BinaryOp::GtEq => 3,
            BinaryOp::Plus | BinaryOp::Minus => 4,
            BinaryOp::Multiply | BinaryOp::Divide => 5,
        }
    }
}

/// 子の式が親より弱い演算子なら括弧で包む
fn child_to_string(child: &Expr, parent_precedence: u8) -> String {
    match child {
        Expr::Binary { op, .. } if op.precedence() < parent_precedence => {
            format!("({child})")
        }
        _ => child.to_string(),
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
            Expr::Unary { op: UnaryOp::Not, expr } => write!(f, "NOT {expr}"),
            Expr::Unary { op: UnaryOp::Minus, expr } => write!(f, "-{expr}"),
            Expr::Binary { left, op, right } => {
                let precedence = op.precedence();
                write!(
                    f,
                    "{} {} {}",
                    child_to_string(left, precedence),
                    op.symbol(),
                    child_to_string(right, precedence)
                )
            }
            Expr::IsNull { expr, negated } => {
                write!(f, "{expr} IS {}NULL", not_str(*negated))
            }
            Expr::InList { expr, list, negated } => {
                let items: Vec<String> = list.iter().map(|e| e.to_string()).collect();
                write!(f, "{expr} {}IN ({})", not_str(*negated), items.join(", "))
            }
            Expr::InSubquery { expr, negated, .. } => {
                write!(f, "{expr} {}IN (…)", not_str(*negated))
            }
            Expr::Like { expr, pattern, negated } => {
                write!(f, "{expr} {}LIKE {pattern}", not_str(*negated))
            }
            Expr::Between { expr, low, high, negated } => {
                write!(f, "{expr} {}BETWEEN {low} AND {high}", not_str(*negated))
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
}
