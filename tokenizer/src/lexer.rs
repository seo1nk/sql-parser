use std::str::Chars;

/// 字句解析器
///
/// `'a` のライフタイムはイテレーターがどこかを参照している  
/// このケースだと `&'a str`
pub struct Lexer<'a> {
    /// ソースのテキスト
    pub source: &'a str,

    /// 残りの文字
    pub chars: Chars<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars(),
        }
    }
}
