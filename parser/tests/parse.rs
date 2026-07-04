use parser::ast::*;
use parser::parse;
use tokenizer::sql_token::{SqlNumber, SqlValue};

fn col(name: &str) -> Expr {
    Expr::Column(ObjectName(vec![name.to_string()]))
}

fn qcol(table: &str, name: &str) -> Expr {
    Expr::Column(ObjectName(vec![table.to_string(), name.to_string()]))
}

fn int(n: i64) -> Expr {
    Expr::Value(SqlValue::Number(SqlNumber::Integer(n)))
}

fn binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
    Expr::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }
}

#[test]
fn parses_standard_select() {
    let query = parse("SELECT id, name FROM users WHERE age >= 20;").unwrap();
    assert_eq!(
        query.body.select,
        SelectList::Items(vec![
            SelectItem {
                expr: col("id"),
                alias: None
            },
            SelectItem {
                expr: col("name"),
                alias: None
            },
        ])
    );
    assert_eq!(
        query.body.from,
        vec![TableExpr {
            primary: TablePrimary::Table {
                name: ObjectName(vec!["users".to_string()]),
                alias: None
            },
            joins: vec![]
        }]
    );
    assert_eq!(
        query.body.where_clause,
        Some(binary(col("age"), BinaryOp::GtEq, int(20)))
    );
}

#[test]
fn from_first_gives_same_ast_as_standard() {
    // 句順序が違っても同じ正規形(論理評価順)の AST になる
    let standard = parse("SELECT id, name FROM users WHERE age >= 20").unwrap();
    let from_first = parse("FROM users WHERE age >= 20 SELECT id, name").unwrap();
    let where_last = parse("FROM users SELECT id, name WHERE age >= 20").unwrap();
    assert_eq!(standard, from_first);
    assert_eq!(standard, where_last);
}

#[test]
fn omitted_select_means_wildcard() {
    let query = parse("FROM users").unwrap();
    assert_eq!(query.body.select, SelectList::Wildcard);

    let explicit = parse("SELECT * FROM users").unwrap();
    assert_eq!(query, explicit);
}

#[test]
fn duplicate_clause_fails() {
    assert!(parse("FROM users WHERE a = 1 WHERE b = 2").is_none());
    assert!(parse("SELECT a FROM t SELECT b").is_none());
}

#[test]
fn missing_from_fails() {
    assert!(parse("SELECT 1").is_none());
    assert!(parse("WHERE a = 1").is_none());
}

#[test]
fn parses_joins() {
    let query = parse(
        "FROM users u JOIN orders o ON u.id = o.user_id LEFT OUTER JOIN items i ON o.item_id = i.id CROSS JOIN t",
    )
    .unwrap();
    let table = &query.body.from[0];
    assert_eq!(table.joins.len(), 3);
    assert_eq!(table.joins[0].join_type, JoinType::Inner);
    assert_eq!(
        table.joins[0].on,
        Some(binary(qcol("u", "id"), BinaryOp::Eq, qcol("o", "user_id")))
    );
    assert_eq!(table.joins[1].join_type, JoinType::Left);
    assert_eq!(table.joins[2].join_type, JoinType::Cross);
    assert_eq!(table.joins[2].on, None);
}

#[test]
fn parses_table_alias() {
    let query = parse("FROM users AS u, orders o").unwrap();
    assert_eq!(query.body.from.len(), 2);
    assert_eq!(
        query.body.from[0].primary,
        TablePrimary::Table {
            name: ObjectName(vec!["users".to_string()]),
            alias: Some("u".to_string())
        }
    );
    assert_eq!(
        query.body.from[1].primary,
        TablePrimary::Table {
            name: ObjectName(vec!["orders".to_string()]),
            alias: Some("o".to_string())
        }
    );
}

#[test]
fn parses_with_and_subquery() {
    let query = parse(
        "WITH adults AS (FROM users WHERE age >= 20 SELECT id, name) \
         SELECT a.name FROM adults a JOIN (SELECT * FROM orders) o ON a.id = o.user_id",
    )
    .unwrap();
    assert_eq!(query.with.len(), 1);
    assert_eq!(query.with[0].name, "adults");
    assert_eq!(
        query.with[0].query.body.where_clause,
        Some(binary(col("age"), BinaryOp::GtEq, int(20)))
    );
    // JOIN 先が FROM 内サブクエリ
    let join = &query.body.from[0].joins[0];
    assert!(matches!(
        &join.table,
        TablePrimary::Subquery { alias: Some(a), .. } if a == "o"
    ));
}

#[test]
fn parses_select_item_alias_and_function() {
    let query = parse("SELECT a.name, count(o.id) AS order_count FROM t a").unwrap();
    let SelectList::Items(items) = &query.body.select else {
        panic!("expected items");
    };
    assert_eq!(items[0].expr, qcol("a", "name"));
    assert_eq!(items[1].alias, Some("order_count".to_string()));
    assert_eq!(
        items[1].expr,
        Expr::Function {
            name: "count".to_string(),
            args: FunctionArgs::List(vec![qcol("o", "id")]),
        }
    );

    // count(*) のワイルドカード引数
    let query = parse("SELECT count(*) FROM t").unwrap();
    let SelectList::Items(items) = &query.body.select else {
        panic!("expected items");
    };
    assert_eq!(
        items[0].expr,
        Expr::Function {
            name: "count".to_string(),
            args: FunctionArgs::Wildcard,
        }
    );
}

#[test]
fn expression_precedence() {
    // a + b * 2 = c AND NOT d OR e
    // => ((a + (b * 2)) = c AND (NOT d)) OR e
    let query = parse("FROM t WHERE a + b * 2 = c AND NOT d OR e").unwrap();
    let expected = binary(
        binary(
            binary(
                binary(col("a"), BinaryOp::Plus, binary(col("b"), BinaryOp::Multiply, int(2))),
                BinaryOp::Eq,
                col("c"),
            ),
            BinaryOp::And,
            Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(col("d")),
            },
        ),
        BinaryOp::Or,
        col("e"),
    );
    assert_eq!(query.body.where_clause, Some(expected));
}

#[test]
fn parens_override_precedence() {
    let query = parse("FROM t WHERE a * (b + 2)").unwrap();
    assert_eq!(
        query.body.where_clause,
        Some(binary(
            col("a"),
            BinaryOp::Multiply,
            binary(col("b"), BinaryOp::Plus, int(2))
        ))
    );
}

#[test]
fn parses_predicates() {
    let query = parse("FROM t WHERE a IS NOT NULL AND b NOT IN (1, 2) AND c LIKE 'x%' AND d BETWEEN 1 AND 10").unwrap();
    let Some(Expr::Binary { left, op: BinaryOp::And, right }) = query.body.where_clause else {
        panic!("expected AND chain");
    };
    // 右端: d BETWEEN 1 AND 10
    assert_eq!(
        *right,
        Expr::Between {
            expr: Box::new(col("d")),
            low: Box::new(int(1)),
            high: Box::new(int(10)),
            negated: false,
        }
    );
    // 左側の AND 連鎖に IS NOT NULL / NOT IN / LIKE が含まれる
    let Expr::Binary { left: ll, right: lr, .. } = *left else {
        panic!("expected nested AND");
    };
    assert_eq!(
        *lr,
        Expr::Like {
            expr: Box::new(col("c")),
            pattern: Box::new(Expr::Value(SqlValue::String("x%".to_string()))),
            negated: false,
        }
    );
    let Expr::Binary { left: lll, right: llr, .. } = *ll else {
        panic!("expected nested AND");
    };
    assert_eq!(
        *lll,
        Expr::IsNull {
            expr: Box::new(col("a")),
            negated: true,
        }
    );
    assert_eq!(
        *llr,
        Expr::InList {
            expr: Box::new(col("b")),
            list: vec![int(1), int(2)],
            negated: true,
        }
    );
}

#[test]
fn parses_in_subquery_and_exists() {
    let query = parse("FROM t WHERE id IN (SELECT user_id FROM orders)").unwrap();
    assert!(matches!(
        query.body.where_clause,
        Some(Expr::InSubquery { negated: false, .. })
    ));

    let query = parse("FROM t WHERE NOT EXISTS (SELECT * FROM orders WHERE orders.user_id = t.id)").unwrap();
    let Some(Expr::Unary { op: UnaryOp::Not, expr }) = query.body.where_clause else {
        panic!("expected NOT");
    };
    assert!(matches!(*expr, Expr::Exists { .. }));
}

#[test]
fn parses_group_having_order_limit_offset() {
    let query = parse(
        "FROM orders GROUP BY user_id HAVING count(*) > 3 SELECT user_id, count(*) ORDER BY user_id DESC, count(*) LIMIT 10 OFFSET 5",
    )
    .unwrap();
    assert_eq!(query.body.group_by, vec![col("user_id")]);
    assert!(query.body.having.is_some());
    assert_eq!(query.body.order_by.len(), 2);
    assert_eq!(query.body.order_by[0].asc, Some(false));
    assert_eq!(query.body.order_by[1].asc, None);
    assert_eq!(query.body.limit, Some(10));
    assert_eq!(query.body.offset, Some(5));
}

#[test]
fn parses_distinct() {
    let query = parse("SELECT DISTINCT name FROM users").unwrap();
    assert!(query.body.distinct);
}

#[test]
fn parses_set_operations() {
    let query = parse("SELECT id FROM a UNION SELECT id FROM b EXCEPT SELECT id FROM c").unwrap();
    assert_eq!(query.set_ops.len(), 2);
    assert_eq!(query.set_ops[0].0, SetOperator::Union);
    assert_eq!(query.set_ops[1].0, SetOperator::Except);
}

#[test]
fn fails_on_leftover_input() {
    assert!(parse("FROM users garbage garbage").is_none());
    // ただし別名としての識別子1つは許される(users garbage は alias)ので、明確に壊れた入力で確認
    assert!(parse("FROM users WHERE").is_none());
    assert!(parse("SELECT FROM users").is_none());
}

#[test]
fn comment_is_ignored_by_parser() {
    let query = parse("FROM users -- コメント\nSELECT id").unwrap();
    let plain = parse("FROM users SELECT id").unwrap();
    assert_eq!(query, plain);
}
