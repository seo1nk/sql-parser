use tokenizer::sql_token::SqlValue;

/// クエリ全体(WITH + 本体 + 集合演算)
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub with: Vec<Cte>,
    pub body: SelectBody,
    pub set_ops: Vec<(SetOperator, SelectBody)>,
}

/// WITH で定義される共通テーブル式
#[derive(Debug, Clone, PartialEq)]
pub struct Cte {
    pub name: String,
    pub query: Query,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetOperator {
    Union,
    Intersect,
    Except,
}

/// SELECT 文の本体
/// 句の出現順(FROM-first でも SELECT-first でも)に依存しない、
/// 論理評価順の正規形で保持する。この並びがそのまま可視化のステップ列になる
#[derive(Debug, Clone, PartialEq)]
pub struct SelectBody {
    /// 1. 集合の形成
    pub from: Vec<TableExpr>,
    /// 2. 行の絞り込み
    pub where_clause: Option<Expr>,
    /// 3. グループ化
    pub group_by: Vec<Expr>,
    /// 4. グループの絞り込み
    pub having: Option<Expr>,
    /// 5. 射影(SELECT 句省略時は Wildcard)
    pub select: SelectList,
    /// 6. 重複排除
    pub distinct: bool,
    /// 7. 並び替え
    pub order_by: Vec<OrderItem>,
    /// 8. 切り出し
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectList {
    /// `SELECT *`(SELECT 句の省略もこれ)
    Wildcard,
    Items(Vec<SelectItem>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectItem {
    pub expr: Expr,
    pub alias: Option<String>,
}

/// FROM の1要素(テーブル + そこに連なる JOIN の列)
#[derive(Debug, Clone, PartialEq)]
pub struct TableExpr {
    pub primary: TablePrimary,
    pub joins: Vec<Join>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TablePrimary {
    /// 実テーブルまたは CTE 名
    Table {
        name: ObjectName,
        alias: Option<String>,
    },
    /// FROM 内のサブクエリ
    Subquery {
        query: Box<Query>,
        alias: Option<String>,
    },
}

/// `users` や `public.users` のような(修飾されうる)名前
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectName(pub Vec<String>);

#[derive(Debug, Clone, PartialEq)]
pub struct Join {
    pub join_type: JoinType,
    pub table: TablePrimary,
    /// CROSS JOIN のときだけ None
    pub on: Option<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderItem {
    pub expr: Expr,
    /// ASC = Some(true) / DESC = Some(false) / 指定なし = None
    pub asc: Option<bool>,
}

/// 式。優先順位はパーサー(expr.rs)が解決し、AST は木構造で保持する
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Value(SqlValue),
    Column(ObjectName),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    IsNull {
        expr: Box<Expr>,
        negated: bool,
    },
    InList {
        expr: Box<Expr>,
        list: Vec<Expr>,
        negated: bool,
    },
    InSubquery {
        expr: Box<Expr>,
        query: Box<Query>,
        negated: bool,
    },
    Like {
        expr: Box<Expr>,
        pattern: Box<Expr>,
        negated: bool,
    },
    Between {
        expr: Box<Expr>,
        low: Box<Expr>,
        high: Box<Expr>,
        negated: bool,
    },
    Exists {
        query: Box<Query>,
    },
    Function {
        name: String,
        args: FunctionArgs,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionArgs {
    /// `count(*)`
    Wildcard,
    List(Vec<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Or,
    And,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Plus,
    Minus,
    Multiply,
    Divide,
}
