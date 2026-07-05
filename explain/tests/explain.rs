use explain::explain_sql;
use explain::flow::{Column, FlowNode, Role};

/// フロントエンドのモックデータと同じサンプルクエリ
const SAMPLE: &str = "WITH adults AS (
  FROM users
  WHERE age >= 20
  SELECT id, name
)
FROM adults a
JOIN orders o
  ON a.id = o.user_id
WHERE o.price > 100
GROUP BY a.name
SELECT a.name,
  count(o.id) AS order_count";

fn column(name: &str, role: Role) -> Column {
    Column {
        name: name.to_string(),
        role,
    }
}

fn find<'a>(graph: &'a explain::flow::FlowGraph, id: &str) -> &'a FlowNode {
    graph
        .nodes
        .iter()
        .find(|n| n.id() == id)
        .unwrap_or_else(|| panic!("node {id} not found"))
}

#[test]
fn sample_query_builds_expected_flow() {
    let graph = explain_sql(SAMPLE).unwrap();

    // ノード: users(scan), filter, project, adults(cte), orders(scan),
    //         joined, filter, group, result
    assert_eq!(graph.nodes.len(), 9);
    assert_eq!(graph.groups.len(), 1);
    assert_eq!(graph.groups[0].label, "① WITH adults AS ( … )");

    // users: id/age は条件のみ(used)、name は最終結果へ(output)、… あり
    let users = graph
        .nodes
        .iter()
        .find(|n| matches!(n, FlowNode::Scan { label, .. } if label == "users"))
        .unwrap();
    let FlowNode::Scan {
        columns, has_more, group_id, ..
    } = users
    else {
        panic!()
    };
    assert!(*has_more);
    assert!(group_id.is_some());
    assert_eq!(
        columns,
        &vec![
            column("id", Role::Used),
            column("name", Role::Output),
            column("age", Role::Used),
        ]
    );

    // adults CTE: 列は確定(has_more=false)、id=used(結合キー), name=output
    let FlowNode::Cte {
        columns,
        has_more,
        alias,
        ..
    } = find(&graph, "cte-adults")
    else {
        panic!()
    };
    assert!(!*has_more);
    assert_eq!(alias.as_deref(), Some("a"));
    assert_eq!(
        columns,
        &vec![column("id", Role::Used), column("name", Role::Output)]
    );

    // 結合済みテーブル: 修飾された列 + has_more(orders 側が不確定)
    let joined = graph
        .nodes
        .iter()
        .find(|n| matches!(n, FlowNode::Joined { .. }))
        .unwrap();
    let FlowNode::Joined {
        id: joined_id,
        label,
        join_type,
        columns,
        has_more,
        ..
    } = joined
    else {
        panic!()
    };
    assert_eq!(label, "adults ⋈ orders");
    assert_eq!(join_type, "INNER");
    assert!(*has_more);
    assert_eq!(
        columns,
        &vec![
            column("a.id", Role::Used),
            column("a.name", Role::Output),
            column("o.id", Role::Output),
            column("o.user_id", Role::Used),
            column("o.price", Role::Used),
        ]
    );

    // 結合キーがそれぞれのエッジのラベルに載る
    let into_joined: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| &e.target == joined_id)
        .collect();
    assert_eq!(into_joined.len(), 2);
    assert_eq!(into_joined[0].label.as_deref(), Some("a.id")); // adults 側
    assert_eq!(into_joined[1].label.as_deref(), Some("o.user_id")); // orders 側
    assert_eq!(into_joined[0].source, "cte-adults");

    // 結果: name / order_count がともに output、列は確定
    let FlowNode::Result {
        columns, has_more, ..
    } = graph
        .nodes
        .iter()
        .find(|n| matches!(n, FlowNode::Result { .. }))
        .unwrap()
    else {
        panic!()
    };
    assert!(!*has_more);
    assert_eq!(
        columns,
        &vec![
            column("name", Role::Output),
            column("order_count", Role::Output),
        ]
    );

    // タイムライン: WITH → FROM → JOIN → WHERE → GROUP BY → SELECT
    let labels: Vec<&str> = graph.timeline.iter().map(|s| s.label.as_str()).collect();
    assert_eq!(
        labels,
        vec![
            "① WITH",
            "② FROM",
            "③ JOIN",
            "④ WHERE",
            "⑤ GROUP BY",
            "⑥ SELECT"
        ]
    );
    // ② FROM は adults(CTE 再利用)と orders
    assert_eq!(graph.timeline[1].node_ids.len(), 2);
    assert!(graph.timeline[1].node_ids.contains(&"cte-adults".to_string()));
}

#[test]
fn wildcard_carries_known_columns() {
    // SELECT 省略(= *)では、条件で使った列も結果に届く(output)
    let graph = explain_sql("FROM users WHERE age >= 20").unwrap();
    let FlowNode::Scan { columns, .. } = graph
        .nodes
        .iter()
        .find(|n| matches!(n, FlowNode::Scan { .. }))
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(columns, &vec![column("age", Role::Output)]);

    // 結果ノードは既知列 + 未知列あり
    let FlowNode::Result {
        columns, has_more, ..
    } = graph
        .nodes
        .iter()
        .find(|n| matches!(n, FlowNode::Result { .. }))
        .unwrap()
    else {
        panic!()
    };
    assert!(*has_more);
    assert_eq!(columns, &vec![column("age", Role::Output)]);
}

#[test]
fn order_by_and_limit_become_sort_and_slice() {
    let graph =
        explain_sql("FROM users SELECT name ORDER BY name DESC LIMIT 10 OFFSET 5").unwrap();
    let kinds: Vec<&str> = graph.timeline.iter().map(|s| s.label.as_str()).collect();
    assert_eq!(
        kinds,
        vec!["① FROM", "② SELECT", "③ ORDER BY", "④ LIMIT"]
    );
    assert!(graph
        .nodes
        .iter()
        .any(|n| matches!(n, FlowNode::Sort { keys, .. } if keys == &vec!["name DESC".to_string()])));
    assert!(graph
        .nodes
        .iter()
        .any(|n| matches!(n, FlowNode::Slice { limit: Some(10), offset: Some(5), .. })));
}

#[test]
fn subquery_in_from_gets_own_group() {
    let graph = explain_sql("FROM (SELECT id FROM orders) o SELECT o.id").unwrap();
    assert_eq!(graph.groups.len(), 1);
    assert!(graph.groups[0].label.contains("AS o"));
    // サブクエリ内: orders(scan) → project → derived(cte 扱い) → result
    assert!(graph
        .nodes
        .iter()
        .any(|n| matches!(n, FlowNode::Cte { label, .. } if label == "o")));
}

#[test]
fn set_operations_are_not_supported_yet() {
    let error = explain_sql("SELECT id FROM a UNION SELECT id FROM b").unwrap_err();
    assert!(error.contains("未対応"));
}

#[test]
fn parse_failure_is_reported() {
    assert!(explain_sql("SELECT FROM WHERE").is_err());
}
