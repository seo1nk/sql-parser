//! TypeScript から呼び出す WASM API
//!
//! wasm-pack build --target web で npm パッケージとしてビルドする。
//! 結果は JSON 文字列で返し、TypeScript 側で
//! frontend/src/types/flow.ts の型として解釈する。

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

/// 成功なら `{"ok":true,"value":...}`、失敗なら `{"ok":false,"error":"..."}` を返す
fn to_json<T: Serialize>(result: Result<T, String>) -> String {
    let wrapped = match result {
        Ok(value) => serde_json::json!({ "ok": true, "value": value }),
        Err(error) => serde_json::json!({ "ok": false, "error": error }),
    };
    wrapped.to_string()
}

/// SQL をパースして AST の JSON を返す
#[wasm_bindgen]
pub fn parse(sql: &str) -> String {
    let result = parser::parse(sql).ok_or_else(|| "SQL のパースに失敗しました".to_string());
    to_json(result)
}

/// SQL をパースして可視化用フローグラフの JSON を返す
/// 返り値の value は frontend/src/types/flow.ts の FlowGraph 型
#[wasm_bindgen]
pub fn explain(sql: &str) -> String {
    to_json(explain::explain_sql(sql))
}
