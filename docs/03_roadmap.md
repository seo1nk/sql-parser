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

## フェーズ 3: 構文解析（parser クレート・新規）

仕様: [05_grammar_spec.md](./05_grammar_spec.md)

- [ ] トークン列を入力とするパーサー基盤（入力型のジェネリック化 or 専用型）
- [ ] AST の定義（`SelectStatement`, `TableExpr`, `Expr` など）
- [ ] 式パーサー（リテラル・列参照・二項演算・優先順位・括弧）
- [ ] 各句のパーサー（SELECT / FROM / JOIN / WHERE / GROUP BY / HAVING / ORDER BY / LIMIT / OFFSET）
- [ ] **句順序自由化**: 句の集合をパースして意味的に組み立てる（重複句はエラー）
- [ ] WITH 句（CTE）
- [ ] サブクエリ（FROM 内・IN / EXISTS）
- [ ] 集合演算（UNION / INTERSECT / EXCEPT）
- [ ] エラー報告（位置・期待していたトークン）

## フェーズ 4: TypeScript 向け API（wasm-api クレート・新規）

仕様: [06_api_design.md](./06_api_design.md)

- [ ] AST への serde 導入（JSON シリアライズ）
- [ ] wasm-bindgen による `parse(sql: string) -> ParseResultJson` の公開
- [ ] npm パッケージ化（wasm-pack）
- [ ] TypeScript 型定義（AST の型を .d.ts として提供）

## フェーズ 5: 可視化フロントエンド

- [ ] 論理評価順のステップ表示（WITH → FROM/JOIN → WHERE → GROUP BY → HAVING → SELECT → DISTINCT → ORDER BY → LIMIT）
- [ ] JOIN による集合の合成の図示（ベン図 / ノードグラフ）
- [ ] WHERE / HAVING による絞り込みの図示
- [ ] SELECT による射影（列の抽出）の図示
- [ ] エディタ連携（入力中の SQL をリアルタイムにパース・可視化）
