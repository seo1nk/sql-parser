use kernel::parser::{Alternative, Parser};

use crate::comment::sql_comment;
use crate::delimiter::sql_delimiter;
use crate::identifier::sql_identifier;
use crate::keyword::sql_keyword;
use crate::operator::sql_operator;
use crate::sql_token::Token;
use crate::value::sql_value;

/// 1つのSQLトークンを認識するパーサー
/// alt の順序が優先順位になる:
/// 1. コメント（`--` が演算子 `-` に誤認されないよう最優先）
/// 2. 値リテラル（TRUE/FALSE/NULL が識別子に誤認されないようキーワード・識別子より先）
/// 3. キーワード（識別子より先）
/// 4. 識別子
/// 5. 演算子・区切り文字
pub fn sql_token() -> Parser<Token> {
    sql_comment()
        .alt(sql_value())
        .alt(sql_keyword())
        .alt(sql_identifier())
        .alt(sql_operator())
        .alt(sql_delimiter())
}

/// SQL文字列全体をトークン列に変換する
/// - 空白（スペース・タブ・改行）はトークンの区切りとして読み飛ばす
/// - どのトークンとしても認識できない文字が残った場合は None
pub fn tokenize(input: &str) -> Option<Vec<Token>> {
    let token_parser = sql_token();
    let mut tokens = Vec::new();
    let mut rest = input.trim_start().to_string();

    while !rest.is_empty() {
        let (token, next) = token_parser.run(rest)?;
        tokens.push(token);
        rest = next.trim_start().to_string();
    }

    Some(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql_token::{SqlKeyword, SqlNumber, SqlValue};

    #[test]
    fn tokenizes_standard_select() {
        let tokens = tokenize("SELECT id, name FROM users;").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Keyword(SqlKeyword::Select),
                Token::Identifier("id".to_string()),
                Token::Delimiter(','),
                Token::Identifier("name".to_string()),
                Token::Keyword(SqlKeyword::From),
                Token::Identifier("users".to_string()),
                Token::Delimiter(';'),
            ]
        );
    }

    #[test]
    fn tokenizes_from_first_query() {
        // FROM から書き始めるスタイルも字句解析としては同じ
        let tokens = tokenize("FROM users WHERE age >= 20 SELECT id").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Keyword(SqlKeyword::From),
                Token::Identifier("users".to_string()),
                Token::Keyword(SqlKeyword::Where),
                Token::Identifier("age".to_string()),
                Token::Operator(">=".to_string()),
                Token::Value(SqlValue::Number(SqlNumber::Integer(20))),
                Token::Keyword(SqlKeyword::Select),
                Token::Identifier("id".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_qualified_name_and_join() {
        let tokens = tokenize("FROM users u JOIN orders o ON u.id = o.user_id").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Keyword(SqlKeyword::From),
                Token::Identifier("users".to_string()),
                Token::Identifier("u".to_string()),
                Token::Keyword(SqlKeyword::Join),
                Token::Identifier("orders".to_string()),
                Token::Identifier("o".to_string()),
                Token::Keyword(SqlKeyword::On),
                Token::Identifier("u".to_string()),
                Token::Delimiter('.'),
                Token::Identifier("id".to_string()),
                Token::Operator("=".to_string()),
                Token::Identifier("o".to_string()),
                Token::Delimiter('.'),
                Token::Identifier("user_id".to_string()),
            ]
        );
    }

    #[test]
    fn tokenizes_literals_and_comment() {
        let tokens = tokenize("WHERE name = 'foo' AND price > 1.5 -- filter").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Keyword(SqlKeyword::Where),
                Token::Identifier("name".to_string()),
                Token::Operator("=".to_string()),
                Token::Value(SqlValue::String("foo".to_string())),
                Token::Keyword(SqlKeyword::And),
                Token::Identifier("price".to_string()),
                Token::Operator(">".to_string()),
                Token::Value(SqlValue::Number(SqlNumber::Float(1.5))),
                Token::Comment(" filter".to_string()),
            ]
        );
    }

    #[test]
    fn empty_input_gives_empty_tokens() {
        assert_eq!(tokenize("   "), Some(vec![]));
    }

    #[test]
    fn fails_on_invalid_character() {
        assert!(tokenize("SELECT #").is_none());
    }
}
