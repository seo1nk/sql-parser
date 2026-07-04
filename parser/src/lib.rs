pub mod ast;
pub mod clause;
pub mod expr;
pub mod primitive;
pub mod query;
pub mod table;
pub mod token_stream;

use tokenizer::tokenize::tokenize;

use crate::ast::Query;
use crate::primitive::delimiter;
use crate::token_stream::TokenStream;

/// SQL 文字列をパースして AST を返す
/// - 字句解析 → 構文解析(句順序自由)の全体を実行する
/// - 末尾の `;` は任意
/// - 入力全体を消費できなければ失敗(None)
pub fn parse(sql: &str) -> Option<Query> {
    let tokens = tokenize(sql)?;
    let stream = TokenStream::new(tokens);
    let (parsed, rest) = query::query().run(stream)?;
    let rest = match delimiter(';').run(rest.clone()) {
        Some(((), after)) => after,
        None => rest,
    };
    if rest.is_empty() { Some(parsed) } else { None }
}
