import { Handle, Position, type NodeProps } from '@xyflow/react'
import type { TableRfNode } from '../types'
import { StepBadge } from './StepBadge'

const KIND_STYLE = {
  scan: { badge: 'SCAN', chip: 'bg-scan-soft text-scan-ink', head: 'text-scan-ink' },
  cte: { badge: 'CTE', chip: 'bg-scan-soft text-scan-ink', head: 'text-scan-ink' },
  joined: {
    badge: 'JOINED',
    chip: 'bg-join-soft text-join-ink',
    head: 'text-join-ink',
  },
  result: {
    badge: 'PROJECT',
    chip: 'bg-project-soft text-project-ink',
    head: 'text-project-ink',
  },
} as const

/** テーブル・CTE・結合済みテーブル・結果を表すノード */
export function TableNode({ data }: NodeProps<TableRfNode>) {
  const style = KIND_STYLE[data.kind]
  return (
    <div
      className={`relative min-w-[208px] rounded-2xl border-2 bg-node transition-[border-color,box-shadow] duration-300 ease-out ${
        data.isHighlighted
          ? 'border-accent shadow-[0_6px_20px_rgba(255,125,174,0.35)]'
          : 'border-node-border shadow-[0_4px_14px_rgba(74,59,68,0.08)]'
      }`}
    >
      {data.stepNo && (
        <StepBadge no={data.stepNo} highlighted={data.isHighlighted} />
      )}
      <Handle type="target" position={Position.Left} />
      <div
        className={`flex items-center gap-2 border-b border-pane-border px-3 py-2 text-[13px] font-bold ${style.head}`}
      >
        <span
          className={`rounded-full px-2 py-0.5 text-[9.5px] font-extrabold tracking-[0.1em] ${style.chip}`}
        >
          {style.badge}
        </span>
        <span>{data.title}</span>
        {data.joinType && (
          <span className="ml-auto pl-2 text-[10px] font-medium text-ink-dim">
            {data.joinType}
          </span>
        )}
      </div>
      <ul className="py-1.5 font-mono text-xs">
        {data.columns.map((column) => (
          <li
            key={column.name}
            className={`flex items-center gap-2 px-3 py-[3px] ${
              column.role === 'output'
                ? 'font-semibold text-accent-ink'
                : 'text-ink-muted'
            }`}
          >
            <span
              className={`size-[6px] shrink-0 rounded-full ${
                column.role === 'output' ? 'bg-accent' : 'bg-ink-dim'
              }`}
            />
            {column.name}
          </li>
        ))}
        {data.hasMore && (
          <li className="flex items-center gap-2 px-3 py-[3px] tracking-[0.15em] text-ink-dim">
            <span className="size-[6px] shrink-0" />…
          </li>
        )}
      </ul>
      <Handle type="source" position={Position.Right} />
    </div>
  )
}
