/// Token
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Select,
    From,
    Where,
    And,
    Or,
    Asterisk,
    Comma,
    Semicolon,
    Identifier(String), // テーブル名, カラム名
    Operator(String),
    Value(Value),
}

/// リテラル
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Number(i64),
    String(String),
}
