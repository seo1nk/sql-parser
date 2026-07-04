# TypeScript 向け API と可視化データモデル（設計案）

## 公開方法

- `wasm-api` クレート（新規）で wasm-bindgen を使い WASM 化
- `wasm-pack build --target web` で npm パッケージとして配布
- AST は serde_json で JSON 化して受け渡し、TypeScript 側に型定義を提供

## API（案）

```typescript
// パースして AST を返す
export function parse(sql: string): ParseResult;

type ParseResult =
  | { ok: true; query: Query }
  | { ok: false; error: { message: string; position?: number } };

// 可視化用: 論理評価順のステップ列に変換して返す
export function explain(sql: string): ExplainResult;
```

## 可視化データモデル（ExplainResult）

クエリを「集合がどう形作られていくか」のステップ列として表現する。
フロントエンドはこの配列を順に描画するだけでよい。

```typescript
type ExplainResult =
  | { ok: true; steps: Step[] }
  | { ok: false; error: { message: string; position?: number } };

type Step =
  | { kind: "cte";     name: string; query: Query }        // WITH: 名前付き集合の事前定義
  | { kind: "scan";    table: string; alias?: string }      // FROM: 元になる集合
  | { kind: "join";    joinType: JoinType; right: TableRef; on: Expr } // 集合の合成
  | { kind: "filter";  predicate: Expr; phase: "where" | "having" }    // 絞り込み
  | { kind: "group";   keys: Expr[] }                       // グループ化
  | { kind: "project"; columns: SelectItem[]; distinct: boolean }      // 抽出（射影）
  | { kind: "sort";    keys: OrderItem[] }                  // 並び替え
  | { kind: "slice";   limit?: number; offset?: number }    // 切り出し
  | { kind: "setop";   op: "union" | "intersect" | "except"; right: Query };
```

### 描画イメージ

```
[users] ──scan──▶ ◯ ──join(orders, on u.id=o.user_id)──▶ ◯
                                                          │ filter(age >= 20)
                                                          ▼
                                   ◯ ──project(id, name)──▶ 結果
```

## 実装メモ

- `Step` 列は Rust 側で `SelectBody`（論理評価順に正規化済み）から機械的に生成できる
- `Expr` は可視化側で文字列表示できるよう、`display: string` を各ノードに含める案もある
- エラー時の `position` は、フェーズ 3 の「位置付き入力型」リファクタリング後に提供
