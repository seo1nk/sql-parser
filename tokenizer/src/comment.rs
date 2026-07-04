use kernel::parser::{Parser, StrParser};

use crate::sql_token::Token;

/// 行コメント（`--` から行末まで）を `Token::Comment` として認識するパーサー
/// コメントの中身（`--` を除く）を保持し、改行文字は消費しない
pub fn sql_comment() -> StrParser<Token> {
    Parser(Box::new(|input: String| {
        let body = input.strip_prefix("--")?;
        // 行末（なければ入力の終わり）までがコメント本文
        let end = body.find('\n').unwrap_or(body.len());
        Some((
            Token::Comment(body[..end].to_string()),
            body[end..].to_string(),
        ))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_line_comment() {
        let (token, rest) = sql_comment()
            .run("-- hello\nSELECT".to_string())
            .unwrap();
        assert_eq!(token, Token::Comment(" hello".to_string()));
        assert_eq!(rest, "\nSELECT");
    }

    #[test]
    fn parses_comment_at_end_of_input() {
        let (token, rest) = sql_comment().run("-- tail".to_string()).unwrap();
        assert_eq!(token, Token::Comment(" tail".to_string()));
        assert_eq!(rest, "");
    }

    #[test]
    fn fails_on_single_minus() {
        assert!(sql_comment().run("- 1".to_string()).is_none());
    }
}
