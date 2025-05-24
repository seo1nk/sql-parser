use std::{collections::HashMap, sync::OnceLock};

/// SQLのトークン
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Keyword(SqlKeyword),
    Identifier(SqlIdentifier),
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

#[derive(Debug, PartialEq, Clone)]
pub enum SqlIdentifier {
    Unquoted(String), // table_name
    Quoted(String),   // `table_name`
}

/// 文字列とSQLのキーワードのMapを返す
pub fn get_select_keyword_map() -> &'static HashMap<String, SqlKeyword> {
    SELECT_KEYWORD_MAP.get_or_init(create_select_keyword_map)
}

fn create_select_keyword_map() -> HashMap<String, SqlKeyword> {
    let mut map = HashMap::new();

    // SELECT文の基本構造
    map.insert("SELECT".to_string(), SqlKeyword::Select);
    map.insert("FROM".to_string(), SqlKeyword::From);
    map.insert("WHERE".to_string(), SqlKeyword::Where);

    // 論理演算
    map.insert("AND".to_string(), SqlKeyword::And);
    map.insert("OR".to_string(), SqlKeyword::Or);
    map.insert("NOT".to_string(), SqlKeyword::Not);

    // 結合
    map.insert("JOIN".to_string(), SqlKeyword::Join);
    map.insert("LEFT".to_string(), SqlKeyword::Left);
    map.insert("RIGHT".to_string(), SqlKeyword::Right);
    map.insert("INNER".to_string(), SqlKeyword::Inner);
    map.insert("OUTER".to_string(), SqlKeyword::Outer);
    map.insert("ON".to_string(), SqlKeyword::On);

    // 集約・ソート
    map.insert("GROUP".to_string(), SqlKeyword::Group);
    map.insert("BY".to_string(), SqlKeyword::By);
    map.insert("HAVING".to_string(), SqlKeyword::Having);
    map.insert("ORDER".to_string(), SqlKeyword::Order);
    map.insert("DISTINCT".to_string(), SqlKeyword::Distinct);

    // その他
    map.insert("AS".to_string(), SqlKeyword::As);
    map.insert("IN".to_string(), SqlKeyword::In);
    map.insert("LIKE".to_string(), SqlKeyword::Like);
    map.insert("BETWEEN".to_string(), SqlKeyword::Between);
    map.insert("IS".to_string(), SqlKeyword::Is);
    map.insert("EXISTS".to_string(), SqlKeyword::Exists);

    // 制限・オフセット
    map.insert("LIMIT".to_string(), SqlKeyword::Limit);
    map.insert("OFFSET".to_string(), SqlKeyword::Offset);

    // 集合演算
    map.insert("UNION".to_string(), SqlKeyword::Union);
    map.insert("INTERSECT".to_string(), SqlKeyword::Intersect);
    map.insert("EXCEPT".to_string(), SqlKeyword::Except);

    // ソート方向
    map.insert("ASC".to_string(), SqlKeyword::Asc);
    map.insert("DESC".to_string(), SqlKeyword::Desc);

    map
}

static SELECT_KEYWORD_MAP: OnceLock<HashMap<String, SqlKeyword>> = OnceLock::new();
