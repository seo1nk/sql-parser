use kernel::combinator::{optional, sep_by1};
use kernel::parser::{Alternative, Parser, RightFunctor};
use tokenizer::sql_token::SqlKeyword;

use crate::ast::{Cte, Query, SetOperator};
use crate::clause::select_body;
use crate::primitive::{TokenParser, delimiter, identifier, keyword};
use crate::token_stream::TokenStream;

/// クエリ全体: [WITH ...] select_body (UNION|INTERSECT|EXCEPT select_body)*
pub fn query() -> TokenParser<Query> {
    Parser(Box::new(|input: TokenStream| {
        let (with, rest) = optional(with_clause()).run(input)?;
        let (body, mut rest) = select_body().run(rest)?;

        // 集合演算は「演算子 + 本体」の左結合の並び
        let mut set_ops = Vec::new();
        while let Some((op, next)) = set_operator().run(rest.clone()) {
            let (right, next) = select_body().run(next)?;
            set_ops.push((op, right));
            rest = next;
        }

        Some((
            Query {
                with: with.unwrap_or_default(),
                body,
                set_ops,
            },
            rest,
        ))
    }))
}

/// UNION | INTERSECT | EXCEPT
fn set_operator() -> TokenParser<SetOperator> {
    keyword(SqlKeyword::Union)
        .replace_with(SetOperator::Union)
        .alt(keyword(SqlKeyword::Intersect).replace_with(SetOperator::Intersect))
        .alt(keyword(SqlKeyword::Except).replace_with(SetOperator::Except))
}

/// WITH cte, cte, ...
fn with_clause() -> TokenParser<Vec<Cte>> {
    Parser(Box::new(|input: TokenStream| {
        let ((), rest) = keyword(SqlKeyword::With).run(input)?;
        sep_by1(cte(), delimiter(',')).run(rest)
    }))
}

/// name AS ( query )
fn cte() -> TokenParser<Cte> {
    Parser(Box::new(|input: TokenStream| {
        let (name, rest) = identifier().run(input)?;
        let ((), rest) = keyword(SqlKeyword::As).run(rest)?;
        let ((), rest) = delimiter('(').run(rest)?;
        let (q, rest) = query().run(rest)?;
        let ((), rest) = delimiter(')').run(rest)?;
        Some((Cte { name, query: q }, rest))
    }))
}
