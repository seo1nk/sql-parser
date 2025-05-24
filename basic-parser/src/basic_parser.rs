use kernel::parser::{Applicative, Functor, Parser};
use kernel::satisfy::satisfy;

/// satisfy を用いて、期待する文字と等しいかを確認する述語関数を部分適用した Char パーサーを返す
pub fn char(expected: char) -> Parser<char> {
    satisfy(move |c| c == expected)
}

/// satisfy を用いて、先頭文字が数字かどうかをチェックし、
/// 数字であれば digitToInt 的な処理（ここでは to_digit(10)）で整数に変換するパーサー
pub fn digit() -> Parser<i32> {
    satisfy(|c| c.is_digit(10)).map(|c| c.to_digit(10).unwrap() as i32)
}

/// 指定された文字列と完全一致するかチェックするパーサー
pub fn string(expected: &str) -> Parser<String> {
    if expected.is_empty() {
        // <Parser<String> as Applicative>::pure("".to_string()) と推論され、
        // Parser<String> に対して実装された pure が呼び出される
        return Applicative::pure(String::new());
    }

    let mut chars = expected.chars();
    // 先頭の文字
    let c = chars.next().unwrap();
    // 残りの文字列
    let cs = chars.as_str();

    // 先頭文字に対するパーサー
    // Haskell での `(:) <$> char c` （<*> string cs）に相当する実装
    // char(c) の結果 `ch` を受け取ったら、文字列の結合関数に変換する
    let cons = char(c).map(|ch: char| {
        Box::new(move |rest: String| {
            let mut result = String::new();
            result.push(ch);
            result.push_str(&rest);
            result
        }) as Box<dyn Fn(String) -> String>
    });

    // そして、cons <*> string cs を ap で表現
    let rest_parser = string(cs);
    rest_parser.ap(cons)
}
