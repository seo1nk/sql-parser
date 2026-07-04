use kernel::combinator::{many0, optional};
use kernel::parser::Parser;
use tokenizer::sql_token::SqlKeyword;

use crate::ast::{Join, JoinType, TableExpr, TablePrimary};
use crate::expr::{expr, object_name};
use crate::primitive::{TokenParser, alias, delimiter, keyword};
use crate::query::query;
use crate::token_stream::TokenStream;

/// FROM の1要素: テーブル + そこに連なる JOIN の列
pub fn table_expr() -> TokenParser<TableExpr> {
    Parser(Box::new(|input: TokenStream| {
        let (primary, rest) = table_primary().run(input)?;
        let (joins, rest) = many0(join()).run(rest)?;
        Some((TableExpr { primary, joins }, rest))
    }))
}

/// 実テーブル(または CTE 名)か、FROM 内サブクエリ
fn table_primary() -> TokenParser<TablePrimary> {
    Parser(Box::new(|input: TokenStream| {
        // ( query ) [AS] alias
        if let Some(((), rest)) = delimiter('(').run(input.clone()) {
            let (q, rest) = query().run(rest)?;
            let ((), rest) = delimiter(')').run(rest)?;
            let (alias_name, rest) = alias().run(rest)?;
            return Some((
                TablePrimary::Subquery {
                    query: Box::new(q),
                    alias: alias_name,
                },
                rest,
            ));
        }
        // name [AS] alias
        let (name, rest) = object_name().run(input)?;
        let (alias_name, rest) = alias().run(rest)?;
        Some((
            TablePrimary::Table {
                name,
                alias: alias_name,
            },
            rest,
        ))
    }))
}

/// join_type JOIN table [ON expr]。CROSS JOIN 以外は ON が必須
fn join() -> TokenParser<Join> {
    Parser(Box::new(|input: TokenStream| {
        let (join_type, rest) = join_type().run(input)?;
        let (table, rest) = table_primary().run(rest)?;
        if join_type == JoinType::Cross {
            return Some((
                Join {
                    join_type,
                    table,
                    on: None,
                },
                rest,
            ));
        }
        let ((), rest) = keyword(SqlKeyword::On).run(rest)?;
        let (on, rest) = expr().run(rest)?;
        Some((
            Join {
                join_type,
                table,
                on: Some(on),
            },
            rest,
        ))
    }))
}

/// JOIN | INNER JOIN | LEFT/RIGHT/FULL [OUTER] JOIN | CROSS JOIN
fn join_type() -> TokenParser<JoinType> {
    Parser(Box::new(|input: TokenStream| {
        // 修飾なしの JOIN は INNER
        if let Some(((), rest)) = keyword(SqlKeyword::Join).run(input.clone()) {
            return Some((JoinType::Inner, rest));
        }
        if let Some(((), rest)) = keyword(SqlKeyword::Inner).run(input.clone()) {
            let ((), rest) = keyword(SqlKeyword::Join).run(rest)?;
            return Some((JoinType::Inner, rest));
        }
        for (kw, join_type) in [
            (SqlKeyword::Left, JoinType::Left),
            (SqlKeyword::Right, JoinType::Right),
            (SqlKeyword::Full, JoinType::Full),
        ] {
            if let Some(((), rest)) = keyword(kw).run(input.clone()) {
                let (_, rest) = optional(keyword(SqlKeyword::Outer)).run(rest)?;
                let ((), rest) = keyword(SqlKeyword::Join).run(rest)?;
                return Some((join_type, rest));
            }
        }
        if let Some(((), rest)) = keyword(SqlKeyword::Cross).run(input.clone()) {
            let ((), rest) = keyword(SqlKeyword::Join).run(rest)?;
            return Some((JoinType::Cross, rest));
        }
        None
    }))
}
