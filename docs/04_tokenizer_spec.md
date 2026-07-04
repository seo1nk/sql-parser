# 字句解析（tokenizer）仕様

入力: SQL 文字列 / 出力: `Vec<Token>`

## トークンの種類

```rust
pub enum Token {
    Keyword(SqlKeyword),   // SELECT, FROM, WHERE, ...
    Identifier(String),    // テーブル名・列名など
    Value(SqlValue),       // 123, 12.5, 'text', TRUE, FALSE, NULL
    Operator(String),      // =, <>, !=, <, >, <=, >=, +, -, *, /
    Delimiter(char),       // ( ) , ; .
    Comment(String),       // -- 行コメント
}
```

## 各トークンの規則

### 識別子（Identifier）

- 開始文字: アルファベット（Unicode 対応）または `_`
- 継続文字: 英数字または `_`
- キーワード表に一致する語は `Keyword`、`TRUE`/`FALSE`/`NULL` は `Value` として優先される

### キーワード（Keyword）

- 大文字・小文字を区別しない（`select` = `SELECT`）
- 一覧: SELECT, FROM, WHERE, AND, OR, NOT, JOIN, LEFT, RIGHT, INNER, OUTER,
  FULL, CROSS, ON, GROUP, BY, HAVING, ORDER, DISTINCT, AS, IN, LIKE, BETWEEN,
  IS, EXISTS, LIMIT, OFFSET, UNION, INTERSECT, EXCEPT, ASC, DESC, WITH

### 値リテラル（Value）

| 種類 | 例 | 備考 |
| --- | --- | --- |
| 整数 | `123` | `i64` |
| 小数 | `12.5` | `f64`。`1.` のように小数部がない場合は整数 `1` + 区切り文字 `.` として読む |
| 文字列 | `'hello'` | シングルクォート。エスケープ（`''`）は未対応（TODO） |
| 真偽値 | `TRUE` / `FALSE` | 大文字・小文字を区別しない |
| NULL | `NULL` | 〃 |

- 負数は字句解析ではなく構文解析で単項マイナスとして扱う（`-1` は `Operator("-")` + `Integer(1)`）

### 演算子（Operator）

- 2 文字演算子（`<>`, `!=`, `<=`, `>=`）を 1 文字演算子より先に試す
- `*` は字句解析では常に演算子。`SELECT *` のワイルドカードとしての解釈は構文解析が行う

### 区切り文字（Delimiter）

- `(` `)` `,` `;` `.`
- `.` は修飾名（`users.id`）のために必要

### コメント（Comment）

- `--` から行末（`\n` の直前）まで。トークンとしては保持し、構文解析で読み飛ばす

## tokenize の動作

1. 先頭の空白（スペース・タブ・改行）を読み飛ばす
2. 以下の順でトークンパーサーを試す（`alt` による優先順位）
   1. コメント（`--` が演算子 `-` に誤認されないよう最優先）
   2. 値リテラル（数値・文字列・TRUE/FALSE/NULL）
   3. キーワード
   4. 識別子
   5. 演算子
   6. 区切り文字
3. 入力が尽きるまで 1〜2 を繰り返す
4. どのパーサーも受理できない文字が残った場合は失敗（`None`）

## 例

```
FROM users WHERE age >= 20 SELECT id, name -- comment
```

```
Keyword(From), Identifier("users"),
Keyword(Where), Identifier("age"), Operator(">="), Value(Integer(20)),
Keyword(Select), Identifier("id"), Delimiter(','), Identifier("name"),
Comment(" comment")
```
