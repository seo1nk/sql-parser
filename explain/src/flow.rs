//! フロントエンドに渡すフローグラフの型定義
//! frontend/src/types/flow.ts の `FlowGraph` と同じ構造に JSON 化される契約

use serde::Serialize;

/// 列の系譜(lineage)における役割
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// 最終結果に値が届く列(集計関数経由を含む)
    Output,
    /// 結合キー・WHERE / GROUP BY などの条件でのみ使われる列
    Used,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Column {
    pub name: String,
    pub role: Role,
}

/// グラフのノード。kind ごとにフロントエンドの描画コンポーネントが変わる
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum FlowNode {
    /// FROM で参照される実テーブル
    Scan {
        id: String,
        label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        alias: Option<String>,
        columns: Vec<Column>,
        #[serde(rename = "hasMore")]
        has_more: bool,
        #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
        group_id: Option<String>,
    },
    /// WITH で定義された共通テーブル式(FROM 内サブクエリの結果もこれ)
    Cte {
        id: String,
        label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        alias: Option<String>,
        columns: Vec<Column>,
        #[serde(rename = "hasMore")]
        has_more: bool,
        #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
        group_id: Option<String>,
    },
    /// JOIN の合流結果(JOIN はノードではなく合流として描く)
    Joined {
        id: String,
        label: String,
        #[serde(rename = "joinType")]
        join_type: String,
        columns: Vec<Column>,
        #[serde(rename = "hasMore")]
        has_more: bool,
        #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
        group_id: Option<String>,
    },
    /// WHERE / HAVING による絞り込み
    Filter {
        id: String,
        phase: String,
        predicate: String,
        #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
        group_id: Option<String>,
    },
    /// GROUP BY によるグループ化
    Group {
        id: String,
        keys: Vec<String>,
        #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
        group_id: Option<String>,
    },
    /// 途中の SELECT(CTE・サブクエリ内の射影)
    Project {
        id: String,
        items: Vec<String>,
        distinct: bool,
        #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
        group_id: Option<String>,
    },
    /// ORDER BY による並び替え
    Sort {
        id: String,
        keys: Vec<String>,
        #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
        group_id: Option<String>,
    },
    /// LIMIT / OFFSET による切り出し
    Slice {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        offset: Option<u64>,
        #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
        group_id: Option<String>,
    },
    /// 最終結果のテーブル
    Result {
        id: String,
        columns: Vec<Column>,
        #[serde(rename = "hasMore")]
        has_more: bool,
        #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
        group_id: Option<String>,
    },
}

impl FlowNode {
    pub fn id(&self) -> &str {
        match self {
            FlowNode::Scan { id, .. }
            | FlowNode::Cte { id, .. }
            | FlowNode::Joined { id, .. }
            | FlowNode::Filter { id, .. }
            | FlowNode::Group { id, .. }
            | FlowNode::Project { id, .. }
            | FlowNode::Sort { id, .. }
            | FlowNode::Slice { id, .. }
            | FlowNode::Result { id, .. } => id,
        }
    }
}

/// ノード間のエッジ。label は JOIN の結合キーなど矢印上に表示する文字列
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FlowEdge {
    pub source: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// WITH 句などノードをまとめる枠
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FlowGroup {
    pub id: String,
    pub label: String,
}

/// 論理実行順タイムラインの 1 ステップ
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimelineStep {
    pub order: u32,
    pub label: String,
    #[serde(rename = "nodeIds")]
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FlowGraph {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    pub groups: Vec<FlowGroup>,
    pub timeline: Vec<TimelineStep>,
}
