import { Tooltip } from '@base-ui-components/react/tooltip'
import { ReactFlowProvider } from '@xyflow/react'
import { SqlPane } from './components/SqlPane'
import { FlowArea } from './features/flow/components/FlowArea'

export default function App() {
  return (
    <Tooltip.Provider>
      <div className="flex h-screen flex-col bg-canvas font-sans text-fg">
        <header className="flex shrink-0 items-baseline gap-3 border-b border-pane-border bg-pane px-5 py-3.5">
          <h1 className="text-[15px] font-semibold tracking-wide">SQL Flow</h1>
          <span className="rounded-full border border-accent-glow px-2.5 py-0.5 text-[11px] tracking-[0.08em] text-accent">
            MOCK DATA
          </span>
          <span className="ml-auto text-xs text-fg-dim">
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
