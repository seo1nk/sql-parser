# アーキテクチャ

## パーサーコンビネータの核（kernel）

### Parser 型

```rust
pub type ParseResult<T> = Option<(T, String)>;
pub struct Parser<T>(pub Box<dyn Fn(String) -> ParseResult<T>>);
```

- 入力文字列を受け取り、成功なら `(パース済みの値, 残りの文字列)` を返す。
- `Box<dyn Fn>` でラップすることで、環境をキャプチャするクロージャもパーサーにできる。

### 型クラス（Haskell 対応）

| trait | Haskell | 提供するもの |
| --- | --- | --- |
| `Functor` | `fmap` / `<$>` | `map` … 結果値への関数適用 |
| `Applicative` | `pure` / `<*>` | `pure` … 値の持ち上げ、`ap` … パーサーの合成 |
| `Alternative` | `empty` / `<|>` | `empty` … 常に失敗、`alt` … 左が失敗したら右 |
| `Monad` | `>>=` | `and_then` … 前の結果に応じて次のパーサーを決める |
| `RightFunctor` | `$>` | `replace_with` … 結果を固定値に差し替え |

`ap` は「関数を返すパーサー `p`」を先に実行し、続けて `self` を実行する。
`rest_parser.ap(cons)` は Haskell の `cons <*> rest_parser` に対応する
（レシーバと引数の役割が Haskell と逆であることに注意）。

### 基本要素・コンビネータ

- `satisfy(predicate)` … 先頭 1 文字が述語を満たせば消費する（全パーサーの原点）
- `many0(p)` / `many1(p)` … Haskell の `many` / `some`。0 回以上 / 1 回以上の繰り返し

## レイヤー構造

```
文字列
  │  tokenizer（字句解析）
  ▼
Vec<Token>
  │  parser（構文解析・今後実装）
  ▼
AST（SelectStatement）
  │  wasm-api（今後実装、serde で JSON 化）
  ▼
TypeScript / Web フロントエンド
```

- **tokenizer**: 空白・コメントの処理、キーワードの大文字小文字の吸収を担当。
  以降のレイヤーは「意味のある単位（トークン）」だけを扱えばよくなる。
- **parser**: トークン列に対するコンビネータで AST を構築。
  句順序の自由化は「句パーサーの繰り返し + 意味的な組み立て」で実現する
  （詳細は [05_grammar_spec.md](./05_grammar_spec.md)）。

## 設計上のメモ・既知の課題

1. **入力が `String`（所有権あり）である**
   `alt` などで入力を `clone()` しており、長い入力では非効率。
   学習目的の明快さを優先して現状維持とし、将来
   `&str` + 位置（オフセット）ベースの入力型へのリファクタリングを検討する
   （エラー位置報告にも必要になる）。

2. **エラーが `None` のみで理由・位置を持たない**
   可視化フロントエンドでは「どこでパースに失敗したか」の提示が重要なので、
   `Result<(T, Input), ParseError>` 化をフェーズ 3 以降で行う。

3. **トークンレベルのパーサー**
   現在の `Parser<T>` は入力が `String` 固定。構文解析では `Vec<Token>` を
   入力とするパーサーが必要になるため、入力型のジェネリック化
   （`Parser<I, T>`）またはトークン列専用の Parser 型の追加を行う。
