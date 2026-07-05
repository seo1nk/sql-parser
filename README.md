# sql-parser

パーサーコンビネータ方式で実装する SELECT 文専用の SQL パーサー。
標準の `SELECT ... FROM ...` に加えて、`FROM users WHERE ... SELECT id` のような
論理評価順（FROM-first）の書き方も受理することを目指す。
最終的には、WITH / FROM / JOIN による集合の形成、WHERE による絞り込み、
SELECT による抽出を可視化する Web フロントエンドに接続する。

- 概要・設計・ロードマップは [docs/](./docs/01_overview.md) を参照
- デモ: `cargo run`（SQL をトークン列に変換して表示）
- テスト: `cargo test --workspace`

## 開発

```sh
cargo test --workspace          # Rust 側のテスト
cargo run -- "FROM users SELECT id"  # AST を表示

cd frontend
pnpm build:wasm                 # Rust → WASM (要 wasm-pack)
pnpm dev                        # 可視化フロントエンド
```
