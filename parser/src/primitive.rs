use kernel::parser::{Functor, Parser};
use tokenizer::sql_token::{SqlKeyword, SqlValue, Token};

use crate::token_stream::TokenStream;

/// トークン列を入力とするパーサーの型エイリアス
pub type TokenParser<T> = Parser<TokenStream, T>;

/// 述語を満たす先頭トークンを1つ消費するパーサー(トークン版の satisfy)
pub fn satisfy_token<P>(predicate: P) -> TokenParser<Token>
where
    P: Fn(&Token) -> bool + 'static,
{
    Parser(Box::new(move |input: TokenStream| {
        let token = input.peek()?;
        if predicate(token) {
            Some((token.clone(), input.advance()))
        } else {
            None
        }
    }))
}

/// 指定のキーワードを1つ消費するパーサー
pub fn keyword(expected: SqlKeyword) -> TokenParser<()> {
    satisfy_token(move |t| matches!(t, Token::Keyword(k) if *k == expected)).map(|_| ())
}

/// 識別子を1つ消費して名前を返すパーサー
pub fn identifier() -> TokenParser<String> {
    Parser(Box::new(|input: TokenStream| match input.peek()? {
        Token::Identifier(name) => Some((name.clone(), input.advance())),
        _ => None,
    }))
}

/// 指定の区切り文字(`(` `)` `,` `;` `.`)を1つ消費するパーサー
pub fn delimiter(expected: char) -> TokenParser<()> {
    satisfy_token(move |t| matches!(t, Token::Delimiter(c) if *c == expected)).map(|_| ())
}

/// 指定の演算子を1つ消費するパーサー
pub fn operator(expected: &str) -> TokenParser<()> {
    let expected = expected.to_string();
    satisfy_token(move |t| matches!(t, Token::Operator(op) if *op == expected)).map(|_| ())
}

/// 値リテラルを1つ消費するパーサー
pub fn value() -> TokenParser<SqlValue> {
    Parser(Box::new(|input: TokenStream| match input.peek()? {
        Token::Value(v) => Some((v.clone(), input.advance())),
        _ => None,
    }))
}

/// テーブルや選択列の別名 `[AS] identifier` をパースする。なければ None で成功する
pub fn alias() -> TokenParser<Option<String>> {
    Parser(Box::new(|input: TokenStream| {
        // AS があれば識別子が必須
        if let Some(((), rest)) = keyword(SqlKeyword::As).run(input.clone()) {
            let (name, rest) = identifier().run(rest)?;
            return Some((Some(name), rest));
        }
        // AS なしの裸の別名(`FROM users u` の u)
        match identifier().run(input.clone()) {
            Some((name, rest)) => Some((Some(name), rest)),
            None => Some((None, input)),
        }
    }))
}
