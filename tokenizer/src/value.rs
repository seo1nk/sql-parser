use kernel::parser::{Alternative, Applicative, Functor, Monad, Parser};

use crate::identifier::identifier;
use crate::sql_token::{SqlNumber, SqlValue, Token};

/// 値リテラル（数値・文字列・TRUE/FALSE/NULL）を `Token::Value` として認識するパーサー
pub fn sql_value() -> Parser<Token> {
    number()
        .map(SqlValue::Number)
        .alt(string_literal().map(SqlValue::String))
        .alt(boolean_or_null())
        .map(Token::Value)
}

/// 数値リテラルのパーサー
/// - `123` → Integer(123)
/// - `12.5` → Float(12.5)
/// - `1.` のように小数点の後に数字が続かない場合は Integer(1) として `.` を残す
///   （`users.id` のような修飾名の `.` と区別がつかないため）
fn number() -> Parser<SqlNumber> {
    Parser(Box::new(|input: String| {
        // 整数部の終わり（最初の非数字）を探す
        let int_end = input
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(input.len());
        if int_end == 0 {
            // 数字で始まっていない
            return None;
        }

        // 小数部（`.` + 1文字以上の数字）があれば Float として読む
        let rest = &input[int_end..];
        if let Some(frac) = rest.strip_prefix('.') {
            let frac_len = frac
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(frac.len());
            if frac_len > 0 {
                let end = int_end + 1 + frac_len;
                let value: f64 = input[..end].parse().ok()?;
                return Some((SqlNumber::Float(value), input[end..].to_string()));
            }
        }

        let value: i64 = input[..int_end].parse().ok()?;
        Some((SqlNumber::Integer(value), input[int_end..].to_string()))
    }))
}

/// 文字列リテラル `'...'` のパーサー
/// クォートを除いた中身を返す。エスケープ（`''`）は未対応
fn string_literal() -> Parser<String> {
    Parser(Box::new(|input: String| {
        let mut chars = input.char_indices();
        let (_, first) = chars.next()?;
        if first != '\'' {
            return None;
        }

        // 閉じクォートを探す。見つからなければ（閉じられていなければ）失敗
        let (close, _) = chars.find(|(_, c)| *c == '\'')?;
        Some((input[1..close].to_string(), input[close + 1..].to_string()))
    }))
}

/// TRUE / FALSE / NULL のパーサー（大文字・小文字を区別しない）
/// 識別子として読んでから照合することで、`TRUEマン` のような
/// 識別子の前方一致を誤って値として認識しないようにする
fn boolean_or_null() -> Parser<SqlValue> {
    identifier().and_then(|word| match word.to_uppercase().as_str() {
        "TRUE" => Applicative::pure(SqlValue::Boolean(true)),
        "FALSE" => Applicative::pure(SqlValue::Boolean(false)),
        "NULL" => Applicative::pure(SqlValue::Null),
        _ => Alternative::empty(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_integer() {
        let (token, rest) = sql_value().run("123 ".to_string()).unwrap();
        assert_eq!(token, Token::Value(SqlValue::Number(SqlNumber::Integer(123))));
        assert_eq!(rest, " ");
    }

    #[test]
    fn parses_float() {
        let (token, rest) = sql_value().run("12.5,".to_string()).unwrap();
        assert_eq!(token, Token::Value(SqlValue::Number(SqlNumber::Float(12.5))));
        assert_eq!(rest, ",");
    }

    #[test]
    fn integer_followed_by_dot_leaves_the_dot() {
        // `1.` は Integer(1) として `.` を残す
        let (token, rest) = sql_value().run("1.".to_string()).unwrap();
        assert_eq!(token, Token::Value(SqlValue::Number(SqlNumber::Integer(1))));
        assert_eq!(rest, ".");
    }

    #[test]
    fn parses_string_literal() {
        let (token, rest) = sql_value().run("'hello' world".to_string()).unwrap();
        assert_eq!(token, Token::Value(SqlValue::String("hello".to_string())));
        assert_eq!(rest, " world");
    }

    #[test]
    fn fails_on_unclosed_string_literal() {
        assert!(sql_value().run("'hello".to_string()).is_none());
    }

    #[test]
    fn parses_boolean_and_null() {
        let (token, _) = sql_value().run("true".to_string()).unwrap();
        assert_eq!(token, Token::Value(SqlValue::Boolean(true)));

        let (token, _) = sql_value().run("NULL".to_string()).unwrap();
        assert_eq!(token, Token::Value(SqlValue::Null));
    }

    #[test]
    fn fails_on_identifier_starting_with_true() {
        // "truthy" は TRUE の前方一致だが識別子
        assert!(sql_value().run("truthy".to_string()).is_none());
    }
}
