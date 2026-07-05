# プロジェクト概要

## ゴール

Rust でパーサーコンビネータ方式の **SELECT 文専用 SQL パーサー** を実装し、
最終的にクエリの「集合の形作られ方」を可視化する Web フロントエンドを提供する。

### 特徴的な要件

1. **句順序の自由化（FROM-first 構文）**
   標準 SQL のように `SELECT ... FROM ...` の順で書かなくてもよい。
   `FROM users WHERE age > 20 SELECT id, name` のように、
   論理的な評価順（FROM → WHERE → SELECT）で書き始められる。
   （PRQL や DuckDB の FROM-first 構文に近い発想）

2. **可視化のための AST**
   パース結果は「実行結果」ではなく「集合がどう形作られるか」を説明するためのデータ。
   - `WITH` / `FROM` / `JOIN` … 集合（テーブル）の形成
   - `WHERE` / `HAVING` … 行の絞り込み
   - `GROUP BY` … 集約によるグループ化
   - `SELECT` … 列の抽出（射影）
   - `ORDER BY` / `LIMIT` … 並び替えと切り出し

3. **TypeScript から呼び出せる API**
   Rust 実装を wasm-bindgen で WASM 化し、AST を JSON（serde）で受け渡す。
   フロントエンドは TypeScript で AST を受け取り可視化する。

## スコープ外（Non-goals）

- SELECT 以外の文（INSERT / UPDATE / DDL など）
- クエリの実行（実データに対する評価）
- 特定 DBMS 方言の完全互換

## クレート構成

```
sql-visualizer (workspace root / bin)
├── kernel        … パーサーコンビネータの核（Parser 型・Functor/Applicative/Alternative/Monad・satisfy・繰り返し系コンビネータ）
├── basic-parser  … 汎用の基本パーサー（char, digit, string, whitespace など）
└── tokenizer     … SQL 字句解析（Token 定義、identifier / keyword / value / operator / delimiter / comment）
    └── (今後) parser クレート … トークン列 → AST
    └── (今後) wasm-api クレート … TypeScript 向け公開 API
```

## ドキュメント一覧

| ファイル | 内容 |
| --- | --- |
| [02_architecture.md](./02_architecture.md) | アーキテクチャ・データの流れ(シーケンス図)・設計上のメモ |
| [03_roadmap.md](./03_roadmap.md) | 開発ロードマップ（フェーズ別チェックリスト） |
| [04_tokenizer_spec.md](./04_tokenizer_spec.md) | 字句解析（トークナイザー）の仕様 |
| [05_grammar_spec.md](./05_grammar_spec.md) | サポートする SELECT 文の文法（句順序自由化を含む） |
| [06_api_design.md](./06_api_design.md) | WASM API（parse / explain）と FlowGraph 契約 |
| [07_ui_design.md](./07_ui_design.md) | 可視化 UI の設計（描画ルール・デザイントークン） |
