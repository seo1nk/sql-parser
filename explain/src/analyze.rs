//! SQL に現れた列参照(事実)を収集し、供給源ごとの列と役割を求める
//!
//! 「列は SQL に現れた事実のみ表示する」という方針のため、
//! スキーマ情報による補完は行わない(docs/07_ui_design.md)

use std::collections::HashMap;

use parser::ast::{Expr, FunctionArgs, SelectBody, SelectItem, SelectList, TablePrimary};

use crate::flow::Role;

/// この body の SELECT が作る列が、外側でどう使われるかのコンテキスト
pub enum ProducedRoles {
    /// メインクエリ: すべて最終結果に届く
    AllOutput,
    /// CTE / サブクエリ: 外側での使われ方から列ごとに決まる
    ByName(HashMap<String, Role>),
}

impl ProducedRoles {
    pub fn role_for(&self, name: &str) -> Role {
        match self {
            ProducedRoles::AllOutput => Role::Output,
            ProducedRoles::ByName(map) => map.get(name).copied().unwrap_or(Role::Used),
        }
    }
}

/// 1つの列参照の事実
#[derive(Debug, Clone)]
pub struct ColumnFact {
    /// 参照時の修飾子(`u.id` の `u`)
    pub qualifier: Option<String>,
    /// 列名(`u.id` の `id`)
    pub name: String,
    pub role: Role,
}

/// 供給源1つ分の列事実(出現順・列名の重複はマージ済み)
#[derive(Debug, Default)]
pub struct SourceFacts {
    pub columns: Vec<ColumnFact>,
}

impl SourceFacts {
    fn merge(&mut self, fact: ColumnFact) {
        match self.columns.iter_mut().find(|c| c.name == fact.name) {
            Some(existing) => existing.role = max_role(existing.role, fact.role),
            None => self.columns.push(fact),
        }
    }
}

/// body 全体の分析結果
#[derive(Debug)]
pub struct BodyFacts {
    /// 供給源ごとの列事実。並び順は collect_sources と同じ
    pub per_source: Vec<SourceFacts>,
}

pub fn max_role(a: Role, b: Role) -> Role {
    if a == Role::Output || b == Role::Output {
        Role::Output
    } else {
        Role::Used
    }
}

/// body 内の供給源を出現順(primary, その joins, 次の primary, ...)に並べる
pub fn collect_sources(body: &SelectBody) -> Vec<&TablePrimary> {
    let mut sources = Vec::new();
    for table_expr in &body.from {
        sources.push(&table_expr.primary);
        for join in &table_expr.joins {
            sources.push(&join.table);
        }
    }
    sources
}

/// 供給源を参照するときの名前(別名 > テーブル名の末尾 > なし)
pub fn source_key(primary: &TablePrimary) -> Option<String> {
    match primary {
        TablePrimary::Table { name, alias } => alias
            .clone()
            .or_else(|| name.0.last().cloned()),
        TablePrimary::Subquery { alias, .. } => alias.clone(),
    }
}

/// SELECT 項目が作る列の名前(別名 > 列名の末尾 > 式の表示)
pub fn produced_name(item: &SelectItem) -> String {
    if let Some(alias) = &item.alias {
        return alias.clone();
    }
    if let Expr::Column(name) = &item.expr
        && let Some(last) = name.0.last() {
            return last.clone();
        }
    item.expr.to_string()
}

/// body の全句を走査して、供給源ごとの列事実を集める
pub fn analyze_body(body: &SelectBody, produced: &ProducedRoles) -> BodyFacts {
    let keys: Vec<Option<String>> = collect_sources(body).iter().map(|s| source_key(s)).collect();
    let mut per_source: Vec<SourceFacts> = keys.iter().map(|_| SourceFacts::default()).collect();

    {
        let mut sink = |fact: ColumnFact| attribute(&mut per_source, &keys, fact);

        // SELECT 句: 作られる列の役割(output/used)を、参照している列に引き継ぐ
        if let SelectList::Items(items) = &body.select {
            for item in items {
                let role = produced.role_for(&produced_name(item));
                walk_expr(&item.expr, role, &mut sink);
            }
        }

        // 条件系はすべて used(表示順が自然になるよう ON → WHERE → ... の順に走査)
        for table_expr in &body.from {
            for join in &table_expr.joins {
                if let Some(on) = &join.on {
                    walk_expr(on, Role::Used, &mut sink);
                }
            }
        }
        if let Some(e) = &body.where_clause {
            walk_expr(e, Role::Used, &mut sink);
        }
        for e in &body.group_by {
            walk_expr(e, Role::Used, &mut sink);
        }
        if let Some(e) = &body.having {
            walk_expr(e, Role::Used, &mut sink);
        }
        for item in &body.order_by {
            walk_expr(&item.expr, Role::Used, &mut sink);
        }
    }

    // SELECT * はすべての列を通過させるので、参照済みの列の役割を引き上げる
    if matches!(body.select, SelectList::Wildcard) {
        for source in &mut per_source {
            for column in &mut source.columns {
                column.role = max_role(column.role, produced.role_for(&column.name));
            }
        }
    }

    BodyFacts { per_source }
}

/// 列参照を修飾子で供給源に振り分ける
/// - 修飾子が供給源の名前に一致すればその供給源へ
/// - 修飾子なしで供給源が1つならその供給源へ
/// - それ以外(どの集合の列か特定できない)は事実として扱えないため捨てる
fn attribute(per_source: &mut [SourceFacts], keys: &[Option<String>], fact: ColumnFact) {
    let index = match &fact.qualifier {
        Some(q) => keys
            .iter()
            .position(|k| k.as_deref() == Some(q.as_str())),
        None if keys.len() == 1 => Some(0),
        None => None,
    };
    if let Some(i) = index {
        per_source[i].merge(fact);
    }
}

/// 式の中の列参照をすべて sink に流す
/// サブクエリの内部は別スコープなので走査しない
fn walk_expr<F: FnMut(ColumnFact)>(expr: &Expr, role: Role, sink: &mut F) {
    match expr {
        Expr::Value(_) | Expr::Exists { .. } => {}
        Expr::Column(name) => {
            let (qualifier, column) = match name.0.as_slice() {
                [single] => (None, single.clone()),
                parts => (
                    Some(parts[..parts.len() - 1].join(".")),
                    parts[parts.len() - 1].clone(),
                ),
            };
            sink(ColumnFact {
                qualifier,
                name: column,
                role,
            });
        }
        Expr::Unary { expr, .. } | Expr::IsNull { expr, .. } => walk_expr(expr, role, sink),
        Expr::Binary { left, right, .. } => {
            walk_expr(left, role, sink);
            walk_expr(right, role, sink);
        }
        Expr::InList { expr, list, .. } => {
            walk_expr(expr, role, sink);
            for e in list {
                walk_expr(e, role, sink);
            }
        }
        Expr::InSubquery { expr, .. } => walk_expr(expr, role, sink),
        Expr::Like { expr, pattern, .. } => {
            walk_expr(expr, role, sink);
            walk_expr(pattern, role, sink);
        }
        Expr::Between {
            expr, low, high, ..
        } => {
            walk_expr(expr, role, sink);
            walk_expr(low, role, sink);
            walk_expr(high, role, sink);
        }
        Expr::Function { args, .. } => {
            if let FunctionArgs::List(list) = args {
                for e in list {
                    walk_expr(e, role, sink);
                }
            }
        }
    }
}
