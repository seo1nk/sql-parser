//! AST(parser::ast::Query)を可視化用のフローグラフ(FlowGraph)に変換する
//!
//! - JOIN はノードではなく「合流して1つの結合済みテーブルになる」形で描き、
//!   結合キーはそれぞれのエッジのラベルにする
//! - 列は SQL に現れた事実のみ。最終結果に届く列は output、条件のみは used
//! - タイムラインは論理実行順(WITH → FROM → JOIN → WHERE → GROUP BY →
//!   HAVING → SELECT → ORDER BY → LIMIT)

pub mod analyze;
pub mod flow;

use std::collections::HashMap;

use parser::ast::{
    Expr, JoinType, ObjectName, Query, SelectBody, SelectList, TablePrimary,
};

use crate::analyze::{
    BodyFacts, ProducedRoles, SourceFacts, analyze_body, collect_sources, identifier_eq,
    max_role, produced_name, source_key,
};
use parser::ast::SelectBody as AstSelectBody;
use crate::flow::{Column, FlowEdge, FlowGraph, FlowGroup, FlowNode, Role, TimelineStep};

/// SQL 文字列をパースしてフローグラフに変換する
pub fn explain_sql(sql: &str) -> Result<FlowGraph, String> {
    let query = parser::parse(sql).ok_or_else(|| "SQL のパースに失敗しました".to_string())?;
    explain(&query)
}

/// AST をフローグラフに変換する
pub fn explain(query: &Query) -> Result<FlowGraph, String> {
    let mut builder = Builder::default();
    builder.build_query(query, None, &ProducedRoles::AllOutput, true)?;
    Ok(builder.finish())
}

/// フローグラフの JSON 文字列を返す(WASM API・デバッグ用)
pub fn explain_sql_to_json(sql: &str) -> Result<String, String> {
    let graph = explain_sql(sql)?;
    serde_json::to_string_pretty(&graph).map_err(|e| e.to_string())
}

/// パイプラインを流れる「現在の集合」の情報
#[derive(Debug, Clone)]
struct Flowing {
    /// この集合を表すノードの ID(次のエッジの始点)
    node_id: String,
    /// 表示名(joined ノードのラベル `a ⋈ b` を組み立てるのに使う)
    label: String,
    /// この集合が持つと分かっている列
    columns: Vec<Column>,
    has_more: bool,
    /// 供給源ノードの参照名(結合時に列を `a.id` のように修飾する)
    qualifier: Option<String>,
}

/// 論理実行順タイムラインの収集箱(メインクエリの分だけ集める)
#[derive(Debug, Default)]
struct TimelineSlots {
    with: Vec<String>,
    from: Vec<String>,
    join: Vec<String>,
    where_: Vec<String>,
    group: Vec<String>,
    having: Vec<String>,
    select: Vec<String>,
    order: Vec<String>,
    limit: Vec<String>,
}

#[derive(Default)]
struct Builder {
    nodes: Vec<FlowNode>,
    edges: Vec<FlowEdge>,
    groups: Vec<FlowGroup>,
    counter: usize,
    /// CTE 名 → そのノード ID
    ctes: HashMap<String, String>,
    slots: TimelineSlots,
}

const CIRCLED: [&str; 10] = ["①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧", "⑨", "⑩"];

impl Builder {
    fn next_id(&mut self, kind: &str) -> String {
        self.counter += 1;
        format!("{kind}-{}", self.counter)
    }

    fn edge(&mut self, source: &str, target: &str, label: Option<String>) {
        self.edges.push(FlowEdge {
            source: source.to_string(),
            target: target.to_string(),
            label,
        });
    }

    /// クエリ(WITH + body)を変換する。返り値は終端ノードの情報
    fn build_query(
        &mut self,
        query: &Query,
        group: Option<String>,
        produced: &ProducedRoles,
        is_main: bool,
    ) -> Result<Flowing, String> {
        if !query.set_ops.is_empty() {
            return Err("UNION / INTERSECT / EXCEPT の可視化は未対応です".to_string());
        }

        let main_facts = analyze_body(&query.body, produced);

        // CTE の列の役割は「それを参照する側」で決まる。CTE は自分より前の CTE しか
        // 参照できないため、後ろの CTE から前へ役割を伝播させる
        // (例: main が b.x を出力し、b が a.x を SELECT していれば a.x も output)
        let mut analyzed: Vec<(&AstSelectBody, BodyFacts)> = vec![(&query.body, main_facts)];
        let mut cte_roles: Vec<HashMap<String, Role>> = vec![HashMap::new(); query.with.len()];
        for (i, cte) in query.with.iter().enumerate().rev() {
            let mut roles = HashMap::new();
            for (body, facts) in &analyzed {
                merge_roles(&mut roles, roles_for_source_named(body, facts, &cte.name));
            }
            let produced_cte = ProducedRoles::by_name(roles.clone());
            let facts_cte = analyze_body(&cte.query.body, &produced_cte);
            analyzed.push((&cte.query.body, facts_cte));
            cte_roles[i] = roles;
        }

        // CTE を先に構築する(本体からノード ID で参照される)
        for (cte, roles) in query.with.iter().zip(cte_roles) {
            self.build_cte(cte, roles, is_main)?;
        }

        let main_facts = &analyzed[0].1;
        self.build_body(&query.body, group, produced, main_facts, is_main)
    }

    /// WITH の1要素をグループ枠つきのサブフローとして構築する
    fn build_cte(
        &mut self,
        cte: &parser::ast::Cte,
        outer_roles: HashMap<String, Role>,
        record_timeline: bool,
    ) -> Result<(), String> {
        let group_id = format!("group-cte-{}", cte.name);
        self.groups.push(FlowGroup {
            id: group_id.clone(),
            label: format!("① WITH {} AS ( … )", cte.name),
        });

        let produced = ProducedRoles::by_name(outer_roles);
        let end = self.build_query(&cte.query, Some(group_id.clone()), &produced, false)?;

        // CTE の列 = SELECT が作る列(事実)。SELECT * なら上流の既知列を引き継ぐ
        let (columns, has_more) = produced_columns(&cte.query.body, &produced, &end);
        let node_id = format!("cte-{}", cte.name);
        self.nodes.push(FlowNode::Cte {
            id: node_id.clone(),
            label: cte.name.clone(),
            alias: None,
            columns,
            has_more,
            group_id: Some(group_id),
        });
        self.edge(&end.node_id, &node_id, None);

        // 参照解決は大文字小文字を区別しない
        self.ctes.insert(cte.name.to_lowercase(), node_id.clone());
        if record_timeline {
            self.slots.with.push(node_id);
        }
        Ok(())
    }

    /// body を「供給源 → 合流 → 絞り込み → グループ化 → 射影」の流れに変換する
    fn build_body(
        &mut self,
        body: &SelectBody,
        group: Option<String>,
        produced: &ProducedRoles,
        facts: &BodyFacts,
        is_main: bool,
    ) -> Result<Flowing, String> {
        // ---- 供給源と JOIN の合流 ----
        let mut source_index = 0;
        let mut flows: Vec<Flowing> = Vec::new();
        for table_expr in &body.from {
            let mut current = self.build_source(
                &table_expr.primary,
                &facts.per_source[source_index],
                group.clone(),
                is_main,
            )?;
            source_index += 1;
            for join in &table_expr.joins {
                let right = self.build_source(
                    &join.table,
                    &facts.per_source[source_index],
                    group.clone(),
                    is_main,
                )?;
                source_index += 1;
                current = self.merge(
                    current,
                    right,
                    join_type_label(join.join_type),
                    join.on.as_ref(),
                    group.clone(),
                    is_main,
                );
            }
            flows.push(current);
        }
        // `FROM a, b` は CROSS 結合として合流させる
        let mut current = flows.remove(0);
        for right in flows {
            current = self.merge(current, right, "CROSS", None, group.clone(), is_main);
        }

        // どの供給源の列か特定できなかった参照も SQL に現れた事実なので、
        // 合流後(または唯一の供給源)の集合に表示する
        if !facts.unattributed.is_empty() {
            let extra: Vec<Column> = facts
                .unattributed
                .iter()
                .map(|fact| Column {
                    name: match &fact.qualifier {
                        Some(q) => format!("{q}.{}", fact.name),
                        None => fact.name.clone(),
                    },
                    role: fact.role,
                })
                .collect();
            self.append_columns(&current.node_id, &extra);
            for column in extra {
                merge_column(&mut current.columns, column);
            }
        }

        // ---- 絞り込み・グループ化 ----
        if let Some(e) = &body.where_clause {
            let id = self.next_id("filter");
            self.nodes.push(FlowNode::Filter {
                id: id.clone(),
                phase: "where".to_string(),
                predicate: e.to_string(),
                group_id: group.clone(),
            });
            self.edge(&current.node_id, &id, None);
            if is_main {
                self.slots.where_.push(id.clone());
            }
            current.node_id = id;
        }
        if !body.group_by.is_empty() {
            let id = self.next_id("group");
            self.nodes.push(FlowNode::Group {
                id: id.clone(),
                keys: body.group_by.iter().map(|e| e.to_string()).collect(),
                group_id: group.clone(),
            });
            self.edge(&current.node_id, &id, None);
            if is_main {
                self.slots.group.push(id.clone());
            }
            current.node_id = id;
        }
        if let Some(e) = &body.having {
            let id = self.next_id("filter");
            self.nodes.push(FlowNode::Filter {
                id: id.clone(),
                phase: "having".to_string(),
                predicate: e.to_string(),
                group_id: group.clone(),
            });
            self.edge(&current.node_id, &id, None);
            if is_main {
                self.slots.having.push(id.clone());
            }
            current.node_id = id;
        }

        // ---- 射影(SELECT)・並び替え・切り出し ----
        let has_tail = !body.order_by.is_empty() || body.limit.is_some() || body.offset.is_some();

        // メインで ORDER BY / LIMIT がないときは結果ノードが SELECT を兼ねる。
        // それ以外(CTE・サブクエリ・後続ステップあり)は SELECT を射影ステップとして挟む
        if !is_main || has_tail {
            let id = self.next_id("project");
            self.nodes.push(FlowNode::Project {
                id: id.clone(),
                items: select_items_display(&body.select),
                distinct: body.distinct,
                group_id: group.clone(),
            });
            self.edge(&current.node_id, &id, None);
            if is_main {
                self.slots.select.push(id.clone());
            }
            current.node_id = id;
        }
        if !body.order_by.is_empty() {
            let id = self.next_id("sort");
            let keys = body
                .order_by
                .iter()
                .map(|item| {
                    let direction = match item.asc {
                        Some(true) => " ASC",
                        Some(false) => " DESC",
                        None => "",
                    };
                    format!("{}{}", item.expr, direction)
                })
                .collect();
            self.nodes.push(FlowNode::Sort {
                id: id.clone(),
                keys,
                group_id: group.clone(),
            });
            self.edge(&current.node_id, &id, None);
            if is_main {
                self.slots.order.push(id.clone());
            }
            current.node_id = id;
        }
        if body.limit.is_some() || body.offset.is_some() {
            let id = self.next_id("slice");
            self.nodes.push(FlowNode::Slice {
                id: id.clone(),
                limit: body.limit,
                offset: body.offset,
                group_id: group.clone(),
            });
            self.edge(&current.node_id, &id, None);
            if is_main {
                self.slots.limit.push(id.clone());
            }
            current.node_id = id;
        }

        // ---- 結果ノード(メインのみ) ----
        if is_main {
            let (columns, has_more) = produced_columns_of(&body.select, produced, &current);
            let id = self.next_id("result");
            self.nodes.push(FlowNode::Result {
                id: id.clone(),
                columns: columns.clone(),
                has_more,
                group_id: group,
            });
            self.edge(&current.node_id, &id, None);
            if !has_tail {
                self.slots.select.push(id.clone());
            }
            current.node_id = id;
            current.columns = columns;
            current.has_more = has_more;
        }

        Ok(current)
    }

    /// 供給源(実テーブル / CTE 参照 / サブクエリ)を1つ構築する
    fn build_source(
        &mut self,
        primary: &TablePrimary,
        facts: &SourceFacts,
        group: Option<String>,
        is_main: bool,
    ) -> Result<Flowing, String> {
        let key = source_key(primary);
        match primary {
            TablePrimary::Table { name, alias } => {
                let simple_name = name.0.last().cloned().unwrap_or_default();

                // CTE 参照なら既存ノードを共有する(集合の再利用がそのまま図に出る)
                if name.0.len() == 1
                    && let Some(cte_id) = self.ctes.get(&simple_name.to_lowercase()).cloned() {
                        if let Some(alias_name) = alias {
                            self.set_cte_alias(&cte_id, alias_name);
                        }
                        let columns = self.node_columns(&cte_id);
                        let has_more = self.node_has_more(&cte_id);
                        if is_main {
                            self.slots.from.push(cte_id.clone());
                        }
                        return Ok(Flowing {
                            node_id: cte_id,
                            label: simple_name,
                            columns,
                            has_more,
                            qualifier: key,
                        });
                    }

                let columns: Vec<Column> = facts
                    .columns
                    .iter()
                    .map(|f| Column {
                        name: f.name.clone(),
                        role: f.role,
                    })
                    .collect();
                let id = self.next_id("scan");
                self.nodes.push(FlowNode::Scan {
                    id: id.clone(),
                    label: name.to_string(),
                    alias: alias.clone(),
                    columns: columns.clone(),
                    // 実テーブルの全列は SQL からは分からない
                    has_more: true,
                    group_id: group,
                });
                if is_main {
                    self.slots.from.push(id.clone());
                }
                Ok(Flowing {
                    node_id: id,
                    label: simple_name,
                    columns,
                    has_more: true,
                    qualifier: key,
                })
            }
            TablePrimary::Subquery { query, alias } => {
                // 外側での使われ方(facts)をサブクエリの産出列の役割にする
                let roles: HashMap<String, Role> = facts
                    .columns
                    .iter()
                    .map(|f| (f.name.clone(), f.role))
                    .collect();
                let produced = ProducedRoles::by_name(roles);

                let group_id = self.next_id("group-subquery");
                let label = alias.clone().unwrap_or_else(|| "サブクエリ".to_string());
                self.groups.push(FlowGroup {
                    id: group_id.clone(),
                    label: format!("( … ) AS {label}"),
                });
                let end = self.build_query(query, Some(group_id.clone()), &produced, false)?;

                let (columns, has_more) = produced_columns(&query.body, &produced, &end);
                let id = self.next_id("derived");
                self.nodes.push(FlowNode::Cte {
                    id: id.clone(),
                    label: label.clone(),
                    alias: None,
                    columns: columns.clone(),
                    has_more,
                    group_id: Some(group_id),
                });
                self.edge(&end.node_id, &id, None);
                if is_main {
                    self.slots.from.push(id.clone());
                }
                Ok(Flowing {
                    node_id: id,
                    label,
                    columns,
                    has_more,
                    qualifier: key,
                })
            }
        }
    }

    /// 2つの集合を結合済みテーブルノードに合流させる
    /// 結合キーはそれぞれの流入エッジのラベルに置く
    fn merge(
        &mut self,
        left: Flowing,
        right: Flowing,
        join_type: &str,
        on: Option<&Expr>,
        group: Option<String>,
        is_main: bool,
    ) -> Flowing {
        let (left_label, right_label) = join_key_labels(on, right.qualifier.as_deref());

        let mut columns = qualified_columns(&left);
        columns.extend(qualified_columns(&right));
        let has_more = left.has_more || right.has_more;
        let label = format!("{} ⋈ {}", left.label, right.label);

        let id = self.next_id("joined");
        self.nodes.push(FlowNode::Joined {
            id: id.clone(),
            label: label.clone(),
            join_type: join_type.to_string(),
            columns: columns.clone(),
            has_more,
            group_id: group,
        });
        self.edge(&left.node_id, &id, left_label);
        self.edge(&right.node_id, &id, right_label);
        if is_main {
            self.slots.join.push(id.clone());
        }

        Flowing {
            node_id: id,
            label,
            columns,
            has_more,
            qualifier: None, // 合流後の列は修飾済み
        }
    }

    /// CTE ノードに参照側の別名を反映する
    fn set_cte_alias(&mut self, node_id: &str, alias_name: &str) {
        for node in &mut self.nodes {
            if let FlowNode::Cte { id, alias, .. } = node
                && id == node_id {
                    *alias = Some(alias_name.to_string());
                }
        }
    }

    fn node_columns(&self, node_id: &str) -> Vec<Column> {
        self.nodes
            .iter()
            .find(|n| n.id() == node_id)
            .map(|n| match n {
                FlowNode::Scan { columns, .. }
                | FlowNode::Cte { columns, .. }
                | FlowNode::Joined { columns, .. }
                | FlowNode::Result { columns, .. } => columns.clone(),
                _ => Vec::new(),
            })
            .unwrap_or_default()
    }

    /// 既存ノードの列リストに列を追加する(名前が重複したら役割をマージ)
    fn append_columns(&mut self, node_id: &str, extra: &[Column]) {
        for node in &mut self.nodes {
            if node.id() != node_id {
                continue;
            }
            if let FlowNode::Scan { columns, .. }
            | FlowNode::Cte { columns, .. }
            | FlowNode::Joined { columns, .. }
            | FlowNode::Result { columns, .. } = node
            {
                for column in extra {
                    merge_column(columns, column.clone());
                }
            }
        }
    }

    fn node_has_more(&self, node_id: &str) -> bool {
        self.nodes
            .iter()
            .find(|n| n.id() == node_id)
            .map(|n| match n {
                FlowNode::Scan { has_more, .. }
                | FlowNode::Cte { has_more, .. }
                | FlowNode::Joined { has_more, .. }
                | FlowNode::Result { has_more, .. } => *has_more,
                _ => false,
            })
            .unwrap_or(false)
    }

    /// タイムラインを論理実行順に組み立てる(存在するステップだけ番号を振る)
    fn finish(self) -> FlowGraph {
        let slots = [
            ("WITH", self.slots.with),
            ("FROM", self.slots.from),
            ("JOIN", self.slots.join),
            ("WHERE", self.slots.where_),
            ("GROUP BY", self.slots.group),
            ("HAVING", self.slots.having),
            ("SELECT", self.slots.select),
            ("ORDER BY", self.slots.order),
            ("LIMIT", self.slots.limit),
        ];
        let timeline = slots
            .into_iter()
            .filter(|(_, ids)| !ids.is_empty())
            .enumerate()
            .map(|(i, (name, node_ids))| TimelineStep {
                order: (i + 1) as u32,
                label: format!("{} {}", CIRCLED.get(i).copied().unwrap_or(""), name),
                node_ids,
            })
            .collect();

        FlowGraph {
            nodes: self.nodes,
            edges: self.edges,
            groups: self.groups,
            timeline,
        }
    }
}

fn join_type_label(join_type: JoinType) -> &'static str {
    match join_type {
        JoinType::Inner => "INNER",
        JoinType::Left => "LEFT",
        JoinType::Right => "RIGHT",
        JoinType::Full => "FULL",
        JoinType::Cross => "CROSS",
    }
}

/// 結合キーのエッジラベルを求める
/// `a.id = o.user_id` のような単純な等結合なら左右に振り分け、
/// それ以外の条件は右側のエッジに条件全体を表示する
fn join_key_labels(
    on: Option<&Expr>,
    right_qualifier: Option<&str>,
) -> (Option<String>, Option<String>) {
    let Some(on) = on else {
        return (None, None);
    };
    if let Expr::Binary {
        left,
        op: parser::ast::BinaryOp::Eq,
        right,
    } = on
        && let (Expr::Column(a), Expr::Column(b)) = (left.as_ref(), right.as_ref()) {
            let matches_right = |name: &ObjectName| {
                name.0.len() >= 2
                    && right_qualifier
                        .is_some_and(|q| identifier_eq(&name.0[0], q))
            };
            // 右側の集合の修飾子に一致する方を右エッジへ
            if matches_right(a) {
                return (Some(b.to_string()), Some(a.to_string()));
            }
            if matches_right(b) {
                return (Some(a.to_string()), Some(b.to_string()));
            }
            return (Some(a.to_string()), Some(b.to_string()));
        }
    (None, Some(on.to_string()))
}

/// 合流時に列名を `a.id` のように参照名で修飾する
fn qualified_columns(flowing: &Flowing) -> Vec<Column> {
    match &flowing.qualifier {
        Some(q) => flowing
            .columns
            .iter()
            .map(|c| Column {
                name: format!("{q}.{}", c.name),
                role: c.role,
            })
            .collect(),
        None => flowing.columns.clone(),
    }
}

/// SELECT リストの表示用文字列
fn select_items_display(select: &SelectList) -> Vec<String> {
    match select {
        SelectList::Wildcard => vec!["*".to_string()],
        SelectList::Items(items) => items
            .iter()
            .map(|item| match &item.alias {
                Some(alias) => format!("{} AS {alias}", item.expr),
                None => item.expr.to_string(),
            })
            .collect(),
    }
}

/// body の SELECT が作る列(事実)と、未知の列がありうるか
fn produced_columns(
    body: &SelectBody,
    produced: &ProducedRoles,
    end: &Flowing,
) -> (Vec<Column>, bool) {
    produced_columns_of(&body.select, produced, end)
}

fn produced_columns_of(
    select: &SelectList,
    produced: &ProducedRoles,
    end: &Flowing,
) -> (Vec<Column>, bool) {
    match select {
        SelectList::Items(items) => {
            let columns = items
                .iter()
                .map(|item| {
                    let name = produced_name(item);
                    let role = produced.role_for(&name);
                    Column { name, role }
                })
                .collect();
            (columns, false)
        }
        // SELECT * は上流の既知列をそのまま通す(全列は分からないままなら has_more)
        SelectList::Wildcard => (end.columns.clone(), end.has_more),
    }
}

/// 列を名前(大文字小文字を区別しない)でマージする
fn merge_column(columns: &mut Vec<Column>, column: Column) {
    match columns
        .iter_mut()
        .find(|c| identifier_eq(&c.name, &column.name))
    {
        Some(existing) => existing.role = max_role(existing.role, column.role),
        None => columns.push(column),
    }
}

/// 役割マップをマージする(output が used に勝つ)
fn merge_roles(into: &mut HashMap<String, Role>, from: HashMap<String, Role>) {
    for (name, role) in from {
        into.entry(name)
            .and_modify(|r| *r = max_role(*r, role))
            .or_insert(role);
    }
}

/// body の分析結果から、指定した名前(CTE)を参照している列ごとの役割を集める
/// キーは小文字に正規化して返す
fn roles_for_source_named(
    body: &SelectBody,
    facts: &BodyFacts,
    cte_name: &str,
) -> HashMap<String, Role> {
    let mut roles: HashMap<String, Role> = HashMap::new();
    for (source, source_facts) in collect_sources(body).iter().zip(&facts.per_source) {
        let is_this_cte = matches!(
            source,
            TablePrimary::Table { name, .. }
                if name.0.len() == 1 && identifier_eq(&name.0[0], cte_name)
        );
        if is_this_cte {
            for fact in &source_facts.columns {
                let entry = roles
                    .entry(fact.name.to_lowercase())
                    .or_insert(fact.role);
                *entry = max_role(*entry, fact.role);
            }
        }
    }
    roles
}
