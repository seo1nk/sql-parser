# 文法仕様（SELECT 文・句順序自由化）

## 基本方針

このパーサーがサポートするのは **SELECT 文（クエリ式）のみ**。
ただし標準 SQL と異なり、**句（clause）は任意の順序で書ける**。

```sql
-- 標準的な書き方
SELECT id, name FROM users WHERE age >= 20;

-- FROM-first（論理評価順）の書き方も同じ意味として受理する
FROM users WHERE age >= 20 SELECT id, name;
```

### 句順序自由化の仕組み

1. 文を「句の並び」としてパースする: `statement = clause+`
2. 各句パーサー（select_clause / from_clause / where_clause / …）を `alt` で束ね、
   `many1` で繰り返す
3. 得られた句のリストを **意味的に組み立てて** `SelectStatement` にする
   - 同じ種類の句が 2 回以上現れたらエラー（例: WHERE が 2 つ）
   - `FROM` は必須、`SELECT` は省略時 `SELECT *` とみなす（FROM-first の書き心地優先）
4. AST は句の出現順に依存しない正規形（論理評価順）で保持する

## 文法（EBNF 風）

```ebnf
query           = [ with_clause ] select_body { set_operator select_body } [ ";" ] ;
set_operator    = "UNION" | "INTERSECT" | "EXCEPT" ;

with_clause     = "WITH" cte { "," cte } ;
cte             = identifier "AS" "(" query ")" ;

(* 句は任意の順序・各 1 回まで。from_clause は必須 *)
select_body     = clause , { clause } ;
clause          = select_clause | from_clause | where_clause
                | group_by_clause | having_clause
                | order_by_clause | limit_clause | offset_clause ;

select_clause   = "SELECT" [ "DISTINCT" ] select_list ;
select_list     = "*" | select_item { "," select_item } ;
select_item     = expr [ [ "AS" ] identifier ] ;

from_clause     = "FROM" table_expr { "," table_expr } ;
table_expr      = table_primary { join } ;
table_primary   = table_name [ [ "AS" ] identifier ]
                | "(" query ")" [ [ "AS" ] identifier ] ;
table_name      = identifier [ "." identifier ] ;
join            = join_type "JOIN" table_primary "ON" expr ;
join_type       = [ "INNER" ] | ( "LEFT" | "RIGHT" | "FULL" ) [ "OUTER" ] | "CROSS" ;

where_clause    = "WHERE" expr ;
group_by_clause = "GROUP" "BY" expr { "," expr } ;
having_clause   = "HAVING" expr ;
order_by_clause = "ORDER" "BY" order_item { "," order_item } ;
order_item      = expr [ "ASC" | "DESC" ] ;
limit_clause    = "LIMIT" integer ;
offset_clause   = "OFFSET" integer ;
```

## 識別子の扱い

- キーワードと同様、**識別子の照合は大文字小文字を区別しない**
  （CTE 参照・テーブル別名の修飾子・列名のマージ）。表示は書かれたままを保持する
- 引用識別子（`"name"`）は未対応

## 式（expr）の優先順位

低いほうから:

| 優先度 | 演算子 |
| --- | --- |
| 1 | `OR` |
| 2 | `AND` |
| 3 | `NOT`（単項） |
| 4 | 比較: `=`, `<>`, `!=`, `<`, `>`, `<=`, `>=`, `IS [NOT] NULL`, `[NOT] IN`, `[NOT] LIKE`, `[NOT] BETWEEN`, `EXISTS` |
| 5 | 加減: `+`, `-` |
| 6 | 乗除: `*`, `/` |
| 7 | 単項マイナス、関数呼び出し `f(args)`, 修飾名 `t.col`, リテラル, `(expr)` |

## AST（構想）

```rust
pub struct Query {
    pub with: Vec<Cte>,                  // WITH（集合の事前定義）
    pub body: SelectBody,
    pub set_ops: Vec<(SetOperator, SelectBody)>, // UNION など
}

pub struct SelectBody {
    // 論理評価順に正規化して保持する（可視化の順序と一致させる）
    pub from: Vec<TableExpr>,            // 1. 集合の形成
    pub where_clause: Option<Expr>,      // 2. 行の絞り込み
    pub group_by: Vec<Expr>,             // 3. グループ化
    pub having: Option<Expr>,            // 4. グループの絞り込み
    pub select: SelectList,              // 5. 射影（省略時は *）
    pub distinct: bool,                  // 6. 重複排除
    pub order_by: Vec<OrderItem>,        // 7. 並び替え
    pub limit: Option<u64>,              // 8. 切り出し
    pub offset: Option<u64>,
}
```

この「論理評価順で持つ」構造がそのまま可視化のステップ列になる。
