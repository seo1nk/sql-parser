/// SQLのトークン
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Keyword(SqlKeyword),
    Identifier(String),
    Value(SqlValue),

    // =, <>, !=, <, >, <=, >=, +, -, *, /
    Operator(String),
    // `(`, `)`, `,`, `;`
    Delimiter(char),

    Comment(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum SqlKeyword {
    // 基本
    Select,
    From,
    Where,

    // 論理演算
    And,
    Or,
    Not,

    // 結合
    Join,
    Left,
    Right,
    Inner,
    Outer,
    On,

    // 集約・ソート
    Group,
    By,
    Having,
    Order,
    Distinct,

    // その他
    As,
    In,
    Like,
    Between,
    Is,
    Exists,

    // 制限・オフセット
    Limit,
    Offset,

    // 集合演算
    Union,
    Intersect,
    Except,

    // ソート方向
    Asc,
    Desc,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SqlNumber {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, PartialEq, Clone)]
pub enum SqlValue {
    Number(SqlNumber),
    String(String), // 'hello world'
    Boolean(bool),  // TRUE, FALSE
    Null,           // NULL
}

/// 文字列からSQLキーワードに変換する
/// 大文字・小文字を区別しない
pub fn get_sql_keyword(word: &str) -> Option<SqlKeyword> {
    match word.to_uppercase().as_str() {
        "SELECT" => Some(SqlKeyword::Select),
        "FROM" => Some(SqlKeyword::From),
        "WHERE" => Some(SqlKeyword::Where),
        "AND" => Some(SqlKeyword::And),
        "OR" => Some(SqlKeyword::Or),
        "NOT" => Some(SqlKeyword::Not),
        "JOIN" => Some(SqlKeyword::Join),
        "LEFT" => Some(SqlKeyword::Left),
        "RIGHT" => Some(SqlKeyword::Right),
        "INNER" => Some(SqlKeyword::Inner),
        "OUTER" => Some(SqlKeyword::Outer),
        "ON" => Some(SqlKeyword::On),
        "GROUP" => Some(SqlKeyword::Group),
        "BY" => Some(SqlKeyword::By),
        "HAVING" => Some(SqlKeyword::Having),
        "ORDER" => Some(SqlKeyword::Order),
        "DISTINCT" => Some(SqlKeyword::Distinct),
        "AS" => Some(SqlKeyword::As),
        "IN" => Some(SqlKeyword::In),
        "LIKE" => Some(SqlKeyword::Like),
        "BETWEEN" => Some(SqlKeyword::Between),
        "IS" => Some(SqlKeyword::Is),
        "EXISTS" => Some(SqlKeyword::Exists),
        "LIMIT" => Some(SqlKeyword::Limit),
        "OFFSET" => Some(SqlKeyword::Offset),
        "UNION" => Some(SqlKeyword::Union),
        "INTERSECT" => Some(SqlKeyword::Intersect),
        "EXCEPT" => Some(SqlKeyword::Except),
        "ASC" => Some(SqlKeyword::Asc),
        "DESC" => Some(SqlKeyword::Desc),
        _ => None,
    }
}
