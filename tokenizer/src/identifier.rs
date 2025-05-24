use kernel::parser::{Functor, Parser};

use crate::sql_token::Token;

/// SQL識別子トークンを生成
pub fn sql_identifier() -> Parser<Token> {
    identifier().map(Token::Identifier)
}

/// 入力文字列の先頭からSQL標準準拠の識別子を認識するパーサー
/// - 成功: Some (識別子, 残り)
/// - 失敗: None（空文字列、無効な開始文字などを含む文字列）
fn identifier() -> Parser<String> {
    Parser(Box::new(|input: String| {
        // UTF-8に対応した `(バイト位置, 文字)` を返す
        let mut chars = input.char_indices();

        // 最初の文字のバイト位置は0なので破棄
        // 空文字の場合は早期リターン
        let (_, first_char) = chars.next()?;
        if !is_identifier_start(first_char) {
            // identifierとして無効な文字列
            return None;
        }

        // 無効な識別子の位置をさがす
        let end_position = chars
            .find(|(_, c)| !is_identifier_continue(*c))
            .map(|(pos, _)| pos)
            .unwrap_or(input.len());

        Some((
            input[..end_position].to_string(),
            input[end_position..].to_string(),
        ))
    }))
}

/// Identifier の開始文字かを確認する関数
/// アルファベット、アンダースコアを許容する
fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

/// Identifier の開始文字かを確認する関数
/// アルファベット、アンダースコア、数値を許容する
fn is_identifier_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
