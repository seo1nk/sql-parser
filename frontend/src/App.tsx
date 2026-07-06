import { Tooltip } from '@base-ui-components/react/tooltip'
import { ReactFlowProvider } from '@xyflow/react'
import { useState } from 'react'
import { SqlPane } from './components/SqlPane'
import { FlowArea } from './features/flow/components/FlowArea'
import { useExplainedGraph } from './hooks/useExplainedGraph'

const INITIAL_SQL = `WITH adults AS (
  FROM users
  WHERE age >= 20
  SELECT id, name
)
FROM adults a
JOIN orders o
  ON a.id = o.user_id
WHERE o.price > 100
GROUP BY a.name
SELECT a.name,
  count(o.id) AS order_count
-- SELECT が先頭でなくても書ける`

/**
 * アプリの根本状態は「SQL 文字列」だけ。
 * グラフとエラーは useExplainedGraph が SQL から導出する
 */
export default function App() {
  const [sql, setSql] = useState(INITIAL_SQL)
  const { graph, error } = useExplainedGraph(sql)

  return (
    <Tooltip.Provider>
      <div className="flex h-screen flex-col bg-canvas font-sans text-ink">
        <header className="flex shrink-0 items-center gap-3 border-b-2 border-pane-border bg-pane px-5 py-3">
          <span className="size-3 rounded-full bg-accent" aria-hidden />
          <h1 className="text-[15px] font-extrabold tracking-wide">
            SQL Visualizer
          </h1>
          <span className="ml-auto text-xs font-medium text-ink-dim">
            SQL を編集するとフローが更新されます
          </span>
        </header>
        <div className="flex min-h-0 flex-1">
          <SqlPane sql={sql} onChange={setSql} error={error} />
          <ReactFlowProvider>
            <FlowArea graph={graph} />
          </ReactFlowProvider>
        </div>
      </div>
    </Tooltip.Provider>
  )
}
