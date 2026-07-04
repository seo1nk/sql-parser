use basic_parser::basic::string;
use kernel::parser::{Alternative, Functor, StrParser};
use kernel::satisfy::satisfy;

use crate::sql_token::Token;

/// SQL演算子を `Token::Operator` として認識するパーサー
/// `<=` が `<` として認識されないよう、2文字の演算子を先に試す
pub fn sql_operator() -> StrParser<Token> {
    let two_chars = string("<>")
        .alt(string("!="))
        .alt(string("<="))
        .alt(string(">="));

    let one_char = satisfy(|c| matches!(c, '=' | '<' | '>' | '+' | '-' | '*' | '/'))
        .map(|c| c.to_string());

    two_chars.alt(one_char).map(Token::Operator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_char_operator_wins_over_one_char() {
        let (token, rest) = sql_operator().run("<= 20".to_string()).unwrap();
        assert_eq!(token, Token::Operator("<=".to_string()));
        assert_eq!(rest, " 20");
    }

    #[test]
    fn parses_one_char_operator() {
        let (token, rest) = sql_operator().run("= 1".to_string()).unwrap();
        assert_eq!(token, Token::Operator("=".to_string()));
        assert_eq!(rest, " 1");
    }

    #[test]
    fn fails_on_non_operator() {
        assert!(sql_operator().run("abc".to_string()).is_none());
    }
}
