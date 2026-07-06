import { useEffect, useState } from 'react'
import type { FlowGraph } from '../types/flow'
import { explainSql } from '../wasm/api'

type ExplainedGraph = {
  /** 最後に成功したフローグラフ(初期ロード中は null) */
  graph: FlowGraph | null
  /** パース失敗時のメッセージ。graph は直前の成功値を保持したままになる */
  error: string | null
}

/**
 * SQL 文字列をデバウンスしてフローグラフに変換するフック
 * 「SQL(入力状態)→ グラフ(導出された状態)」の変換をここに閉じ込め、
 * コンポーネントは返り値を描画するだけにする
 */
export function useExplainedGraph(sql: string, delayMs = 300): ExplainedGraph {
  const [graph, setGraph] = useState<FlowGraph | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    const timer = setTimeout(async () => {
      try {
        const result = await explainSql(sql)
        if (cancelled) return
        if (result.ok) {
          setGraph(result.graph)
          setError(null)
        } else {
          // 失敗時は直前の正常なグラフを残したままエラーを表示する
          setError(result.error)
        }
      } catch (cause) {
        // WASM の読み込み失敗など、パース以前の例外もエラー表示に落とす
        console.error(cause)
        if (!cancelled) {
          setError('うまく動きませんでした。ページを再読み込みしてみてください。')
        }
      }
    }, delayMs)
    return () => {
      cancelled = true
      clearTimeout(timer)
    }
  }, [sql, delayMs])

  return { graph, error }
}
