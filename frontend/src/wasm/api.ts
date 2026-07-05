/**
 * Rust パーサー(WASM)の呼び出しラッパー
 * wasm-api クレートを `pnpm build:wasm` でビルドすると src/wasm/pkg が生成される
 */
import type { FlowGraph } from '../types/flow'
import init, { explain as wasmExplain } from './pkg/wasm_api'
import wasmUrl from './pkg/wasm_api_bg.wasm?url'

let ready: Promise<unknown> | null = null

/** WASM モジュールを初期化する(初回のみフェッチ、以降は同じ Promise を返す) */
function initWasm(): Promise<unknown> {
  ready ??= init({ module_or_path: wasmUrl })
  return ready
}

type ApiResponse<T> =
  | { ok: true; value: T }
  | { ok: false; error: string }

export type ExplainResult =
  | { ok: true; graph: FlowGraph }
  | { ok: false; error: string }

/** SQL を可視化用フローグラフに変換する */
export async function explainSql(sql: string): Promise<ExplainResult> {
  await initWasm()
  const response = JSON.parse(wasmExplain(sql)) as ApiResponse<FlowGraph>
  return response.ok
    ? { ok: true, graph: response.value }
    : { ok: false, error: response.error }
}
