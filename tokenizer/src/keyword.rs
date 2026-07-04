use kernel::parser::{Alternative, Applicative, Monad, StrParser};

use crate::identifier::identifier;
use crate::sql_token::{Token, get_sql_keyword};

/// 識別子として読み取った語がSQLキーワード表に一致すれば `Token::Keyword` として認識するパーサー
/// 一致しなければ失敗する（識別子としての解釈は sql_identifier に任せる）
pub fn sql_keyword() -> StrParser<Token> {
    identifier().and_then(|word| match get_sql_keyword(&word) {
        // pure でパーサーの文脈に持ち上げる
        Some(keyword) => Applicative::pure(Token::Keyword(keyword)),
        // キーワードでなければ、このパーサー全体を失敗させる
        None => Alternative::empty(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql_token::SqlKeyword;

    #[test]
    fn parses_keyword_case_insensitively() {
        let (token, rest) = sql_keyword().run("select id".to_string()).unwrap();
        assert_eq!(token, Token::Keyword(SqlKeyword::Select));
        assert_eq!(rest, " id");
    }

    #[test]
    fn fails_on_non_keyword() {
        assert!(sql_keyword().run("users".to_string()).is_none());
    }

    #[test]
    fn does_not_match_keyword_prefix_of_identifier() {
        // "selection" は SELECT で始まるが識別子なので失敗する
        assert!(sql_keyword().run("selection".to_string()).is_none());
    }
}
