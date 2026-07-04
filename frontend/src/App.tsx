import { Tooltip } from '@base-ui-components/react/tooltip'
import { ReactFlowProvider } from '@xyflow/react'
import { SqlPane } from './components/SqlPane'
import { FlowArea } from './features/flow/components/FlowArea'

export default function App() {
  return (
    <Tooltip.Provider>
      <div className="flex h-screen flex-col bg-canvas font-sans text-ink">
        <header className="flex shrink-0 items-center gap-3 border-b-2 border-pane-border bg-pane px-5 py-3">
          <span className="size-3 rounded-full bg-accent" aria-hidden />
          <h1 className="text-[15px] font-extrabold tracking-wide">
            SQL Flow
          </h1>
          <span className="rounded-full bg-accent-soft px-2.5 py-0.5 text-[11px] font-bold tracking-[0.08em] text-accent-ink">
            MOCK DATA
          </span>
          <span className="ml-auto text-xs font-medium text-ink-dim">
            ノードにホバーすると上流の経路がハイライト / バッジとタイムラインは論理実行順
          </span>
        </header>
        <div className="flex min-h-0 flex-1">
          <SqlPane />
          <ReactFlowProvider>
            <FlowArea />
          </ReactFlowProvider>
        </div>
      </div>
    </Tooltip.Provider>
  )
}
