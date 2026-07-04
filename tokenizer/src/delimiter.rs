use kernel::parser::{Functor, Parser};
use kernel::satisfy::satisfy;

use crate::sql_token::Token;

/// 区切り文字（`(`, `)`, `,`, `;`, `.`）を `Token::Delimiter` として認識するパーサー
/// `.` は `users.id` のような修飾名のために必要
pub fn sql_delimiter() -> Parser<Token> {
    satisfy(|c| matches!(c, '(' | ')' | ',' | ';' | '.')).map(Token::Delimiter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_delimiter() {
        let (token, rest) = sql_delimiter().run(", name".to_string()).unwrap();
        assert_eq!(token, Token::Delimiter(','));
        assert_eq!(rest, " name");
    }

    #[test]
    fn fails_on_non_delimiter() {
        assert!(sql_delimiter().run("abc".to_string()).is_none());
    }
}
