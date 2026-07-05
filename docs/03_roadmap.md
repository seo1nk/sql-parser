# 開発ロードマップ

論理的な評価順（FROM → WHERE → GROUP BY → HAVING → SELECT → ORDER BY → LIMIT）を
可視化する、という最終目標から逆算したフェーズ分け。

## フェーズ 1: パーサーコンビネータ基盤（kernel / basic-parser）

- [x] `Parser<T>` 型と `run`
- [x] `Functor` / `Applicative` / `Alternative` / `RightFunctor`
- [x] `satisfy`
- [x] 基本パーサー（`char` / `digit` / `string` / `whitespace` / `alphabet` / `alphanumeric`）
- [x] `Monad`（`and_then`）… 前の結果で次のパーサーを選ぶために必要
- [x] 繰り返し系コンビネータ（`many0` / `many1`）
- [ ] （必要になり次第）`sep_by` / `between` / `optional`

## フェーズ 2: 字句解析（tokenizer）

仕様: [04_tokenizer_spec.md](./04_tokenizer_spec.md)

- [x] `Token` / `SqlKeyword` / `SqlValue` の定義
- [x] `identifier` パーサー
- [x] `keyword` パーサー（識別子として読んでからキーワード表と照合）
- [x] 数値リテラル（整数・小数）
- [x] 文字列リテラル（`'...'`）
- [x] `TRUE` / `FALSE` / `NULL`
- [x] 演算子（`=`, `<>`, `!=`, `<`, `>`, `<=`, `>=`, `+`, `-`, `*`, `/`）
- [x] 区切り文字（`(`, `)`, `,`, `;`, `.`）
- [x] 行コメント（`-- ...`）
- [x] `tokenize`: 文字列全体をトークン列に変換
- [ ] エラー位置の報告（フェーズ 3 の入力型リファクタリングと合わせて）

## フェーズ 3: 構文解析（parser クレート）

仕様: [05_grammar_spec.md](./05_grammar_spec.md)

- [x] トークン列を入力とするパーサー基盤（kernel を `Parser<I, T>` にジェネリック化 + `TokenStream`）
- [x] AST の定義（`Query` / `SelectBody` / `TableExpr` / `Expr` など。論理評価順の正規形）
- [x] 式パーサー（リテラル・列参照・関数・二項演算・優先順位・括弧・単項 NOT / マイナス）
- [x] 各句のパーサー（SELECT / FROM / JOIN / WHERE / GROUP BY / HAVING / ORDER BY / LIMIT / OFFSET）
- [x] **句順序自由化**: `many1(clause)` でパースして意味的に組み立てる（重複句はエラー、SELECT 省略時は `*`、FROM 必須）
- [x] WITH 句（CTE）
- [x] サブクエリ（FROM 内・IN / EXISTS）
- [x] 集合演算（UNION / INTERSECT / EXCEPT）
- [ ] エラー報告（位置・期待していたトークン）

## フェーズ 4: TypeScript 向け API（explain / wasm-api クレート）

仕様: [06_api_design.md](./06_api_design.md)

- [x] AST への serde 導入（JSON シリアライズ）
- [x] **explain クレート**: AST → FlowGraph 変換（JOIN の合流、列の系譜 = output/used の計算、
  事実のみの列表示、論理実行順タイムライン生成）
- [x] wasm-bindgen による `parse(sql)` / `explain(sql)` の公開（JSON 文字列で返す）
- [x] wasm-pack ビルド（`frontend/src/wasm/pkg` に生成、`pnpm build:wasm`）
- [x] TypeScript 型定義（FlowGraph 契約は frontend/src/types/flow.ts に手書きで管理）
- [ ] UNION / INTERSECT / EXCEPT の可視化（explain 側が未対応）
- [ ] npm パッケージとしての配布（現状はリポジトリ内ビルドのみ）

## フェーズ 5: 可視化フロントエンド

設計: [07_ui_design.md](./07_ui_design.md)

- [x] デザインモックアップ（HTML 一枚・フィードバック反映済み）
- [x] `frontend/` 雛形（Vite + React + TS + Tailwind + Base UI + @xyflow/react + elkjs）
- [x] フローキャンバス（モック `Step[]` データ、ELK 層状レイアウト、左→右 = 論理実行順）
- [x] JOIN の合流描画（結合済みテーブルノード + 矢印上の結合キーラベル）
- [x] WHERE / HAVING / GROUP BY / SELECT のステップノードと列の系譜色分け（事実のみ表示・`…`）
- [x] 上流経路ハイライト + パーティクル、実行順タイムライン（ステップ番号バッジ連動）
- [x] SQL エディタ（入力中の SQL をデバウンスしてリアルタイムにパース・可視化、
  エラー時は直前のグラフを残してメッセージ表示）
- [x] WASM の `explain()` 実出力への差し替え（モックデータは廃止）
- [ ] シンタックスハイライト付きエディタ（現状はプレーンな textarea）
- [ ] チューニング支援（エッジへの行数・コスト表示、EXPLAIN 連携）
