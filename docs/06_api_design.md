# TypeScript 向け API(WASM)

Rust 実装(parser / explain)を wasm-bindgen で WASM 化し、ブラウザ内から呼び出す。
クレートは [wasm-api/](../wasm-api/src/lib.rs)。

## ビルド

```sh
# 前提: rustup target add wasm32-unknown-unknown / cargo install wasm-pack
cd frontend
pnpm build:wasm
# = wasm-pack build ../wasm-api --target web --out-dir ../frontend/src/wasm/pkg
```

生成物(`frontend/src/wasm/pkg/`、git 管理外):

| ファイル | 内容 |
| --- | --- |
| `wasm_api.js` | wasm-bindgen が生成するローダー + グルーコード |
| `wasm_api_bg.wasm` | 本体(tokenizer + parser + explain を含む) |
| `wasm_api.d.ts` | 公開関数の TypeScript 型定義(自動生成) |

## 公開 API

いずれも入力は SQL 文字列、出力は JSON 文字列。
成功なら `{"ok": true, "value": ...}`、失敗なら `{"ok": false, "error": "メッセージ"}`。

| 関数 | value の中身 | 用途 |
| --- | --- | --- |
| `explain(sql)` | **FlowGraph**(後述) | 可視化。フロントエンドの本命 API |
| `parse(sql)` | AST(`parser::ast::Query` の serde 出力) | デバッグ・検証用。TS 型定義は未提供 |

### TypeScript からの呼び出し

ラッパーは [frontend/src/wasm/api.ts](../frontend/src/wasm/api.ts)。
初回呼び出し時に一度だけ `init()`(.wasm のフェッチ)を行い、以降は同期的に実行される。

```typescript
import { explainSql } from './wasm/api'

const result = await explainSql('FROM users SELECT id')
if (result.ok) {
  result.graph // FlowGraph 型
} else {
  result.error // 日本語のエラーメッセージ
}
```

## FlowGraph 契約

`explain()` が返す可視化用グラフ。型は **Rust 側**([explain/src/flow.rs](../explain/src/flow.rs))と
**TS 側**([frontend/src/types/flow.ts](../frontend/src/types/flow.ts))で手動で同期している
(変更するときは両方を直すこと)。

```typescript
type FlowGraph = {
  nodes: FlowNode[]      // kind ごとに描画が変わる
  edges: FlowEdge[]      // { source, target, label? } label は JOIN の結合キー
  groups: FlowGroup[]    // WITH 句・サブクエリの枠 { id, label }
  timeline: TimelineStep[] // 論理実行順 { order, label: "① WITH", nodeIds }
}
```

### ノードの種類(FlowNode.kind)

| kind | 意味 | 主なフィールド |
| --- | --- | --- |
| `scan` | FROM の実テーブル | label, alias, columns, hasMore |
| `cte` | WITH の共通テーブル式・FROM 内サブクエリの結果 | 同上 |
| `joined` | JOIN の合流結果(`adults ⋈ orders`) | joinType, columns, hasMore |
| `filter` | WHERE / HAVING | phase, predicate(表示用文字列) |
| `group` | GROUP BY | keys |
| `project` | 途中の SELECT(CTE 内など) | items, distinct |
| `sort` | ORDER BY | keys |
| `slice` | LIMIT / OFFSET | limit, offset |
| `result` | 最終結果のテーブル | columns, hasMore |

### 列の系譜(Column)

```typescript
type Column = { name: string; role: 'output' | 'used' }
```

- `output` … 最終結果に値が届く列(集計関数経由・CTE チェーン経由を含む)
- `used` … 結合キー・WHERE / GROUP BY などの条件でのみ使われる列
- `hasMore: true` … SQL に現れていない列が存在しうる(フロントは `…` を描画)

列は **SQL に現れた事実のみ**。スキーマによる補完はしない。
どの供給源か特定できない列(修飾なしで複数ソース)は合流後の集合に表示される。

## 制限・今後

- UNION / INTERSECT / EXCEPT は explain 未対応(`ok: false` でエラーを返す)
- エラーに位置情報がない(「パースに失敗しました」のみ)
- npm パッケージとしての配布は未対応(リポジトリ内ビルドのみ)
- `parse()` の AST JSON は serde の既定形式で、TS 型定義がない
