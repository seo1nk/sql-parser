use crate::parser_type::Parser;

/// 与えられた述語関数を用いて、入力の先頭文字がその関数を満たすかをチェックするパーサーを返す
pub fn satisfy<P>(predicate: P) -> Parser<char>
where
    P: Fn(char) -> bool + 'static,
{
    // predicate をキャプチャするから move する
    Parser(Box::new(move |input: String| {
        // 先頭文字を取得
        let first = input.chars().next();
        match first {
            // ガード式で、先頭文字が述語関数を満たすかをチェック
            Some(c) if predicate(c) => {
                // 先頭文字のバイト数を取得
                let first_len = c.len_utf8();
                // 残りの文字列を取得
                let remaining = input[first_len..].to_string();
                Some((c, remaining))
            }
            _ => None,
        }
    }))
}
