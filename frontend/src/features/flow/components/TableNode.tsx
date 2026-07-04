import { Handle, Position, type NodeProps } from '@xyflow/react'
import type { TableRfNode } from '../types'
import { StepBadge } from './StepBadge'

const KIND_STYLE = {
  scan: { badge: 'SCAN', badgeBg: 'bg-step-scan', head: 'text-[#9ce8ee]' },
  cte: { badge: 'CTE', badgeBg: 'bg-step-scan', head: 'text-[#9ce8ee]' },
  joined: { badge: 'JOINED', badgeBg: 'bg-step-join', head: 'text-[#dfb8ff]' },
  result: {
    badge: 'PROJECT',
    badgeBg: 'bg-step-project',
    head: 'text-[#8ef7c0]',
  },
} as const

/** テーブル・CTE・結合済みテーブル・結果を表すノード */
export function TableNode({ data }: NodeProps<TableRfNode>) {
  const style = KIND_STYLE[data.kind]
  return (
    <div
      className={`relative min-w-[208px] rounded-md border bg-node shadow-[0_0_20px_rgba(0,0,0,0.4)] transition-[border-color,box-shadow] duration-300 ease-out ${
        data.isHighlighted
          ? 'border-accent shadow-[0_0_20px_rgba(29,237,131,0.4)]'
          : 'border-node-border'
      }`}
    >
      {data.stepNo && (
        <StepBadge no={data.stepNo} highlighted={data.isHighlighted} />
      )}
      <Handle type="target" position={Position.Left} />
      <div
        className={`flex items-center gap-2 border-b border-white/10 px-3 py-2 text-[13px] font-semibold ${style.head}`}
      >
        <span
          className={`rounded-[3px] px-1.5 py-0.5 text-[9.5px] font-bold tracking-[0.1em] text-[#0c0d0d] ${style.badgeBg}`}
        >
          {style.badge}
        </span>
        <span>{data.title}</span>
        {data.joinType && (
          <span className="ml-auto pl-2 text-[10px] font-normal text-fg-dim">
            {data.joinType}
          </span>
        )}
      </div>
      <ul className="py-1.5 font-mono text-xs">
        {data.columns.map((column) => (
          <li
            key={column.name}
            className={`flex items-center gap-2 px-3 py-[3px] ${
              column.role === 'output' ? 'text-accent' : 'text-fg-muted'
            }`}
          >
            <span
              className={`size-[5px] shrink-0 rounded-full ${
                column.role === 'output' ? 'bg-accent' : 'bg-fg-dim'
              }`}
            />
            {column.name}
          </li>
        ))}
        {data.hasMore && (
          <li className="flex items-center gap-2 px-3 py-[3px] tracking-[0.15em] text-fg-dim opacity-55">
            <span className="size-[5px] shrink-0" />…
          </li>
        )}
      </ul>
      <Handle type="source" position={Position.Right} />
    </div>
  )
}
