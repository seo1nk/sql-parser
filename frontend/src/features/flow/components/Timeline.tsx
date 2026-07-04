import { Fragment } from 'react'
import type { TimelineStep } from '../../../types/flow'

type Props = {
  steps: TimelineStep[]
  /** チップのホバー/フォーカスで該当ノードをハイライト。null で解除 */
  onHover: (nodeIds: string[] | null) => void
  /** チップのクリックで該当ノードにフォーカス(fitView) */
  onSelect: (nodeIds: string[]) => void
}

/** 論理実行順タイムライン。フローの並びと実行順の対応を示す */
export function Timeline({ steps, onHover, onSelect }: Props) {
  const ordered = [...steps].sort((a, b) => a.order - b.order)
  return (
    <nav
      aria-label="論理実行順"
      className="flex items-center gap-1.5 overflow-x-auto border-t border-pane-border bg-pane px-5 py-3"
    >
      <span className="mr-2 shrink-0 text-[11px] tracking-[0.1em] text-fg-dim">
        実行順
      </span>
      {ordered.map((step, i) => (
        <Fragment key={step.order}>
          {i > 0 && <span className="shrink-0 text-xs text-fg-dim">→</span>}
          <button
            type="button"
            className="shrink-0 cursor-pointer rounded-full border border-pane-border bg-pane-muted px-3.5 py-1 font-mono text-xs text-fg-muted transition-[color,border-color,box-shadow] duration-200 ease-out hover:border-accent hover:text-accent hover:shadow-[0_0_12px_rgba(29,237,131,0.4)] focus-visible:border-accent focus-visible:text-accent focus-visible:shadow-[0_0_12px_rgba(29,237,131,0.4)] focus-visible:outline-none"
            onMouseEnter={() => onHover(step.nodeIds)}
            onMouseLeave={() => onHover(null)}
            onFocus={() => onHover(step.nodeIds)}
            onBlur={() => onHover(null)}
            onClick={() => onSelect(step.nodeIds)}
          >
            {step.label}
          </button>
        </Fragment>
      ))}
    </nav>
  )
}
