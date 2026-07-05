import { Handle, Position, type NodeProps } from '@xyflow/react'
import type { StepRfNode } from '../types'
import { StepBadge } from './StepBadge'

const KIND_STYLE = {
  filter: {
    badge: 'FILTER',
    chip: 'bg-filter-soft text-filter-ink',
    head: 'text-filter-ink',
  },
  group: {
    badge: 'GROUP',
    chip: 'bg-group-soft text-group-ink',
    head: 'text-group-ink',
  },
  project: {
    badge: 'PROJECT',
    chip: 'bg-project-soft text-project-ink',
    head: 'text-project-ink',
  },
  // 並び替えはグループ化と同系色、切り出しは絞り込みと同系色で表現する
  sort: {
    badge: 'SORT',
    chip: 'bg-group-soft text-group-ink',
    head: 'text-group-ink',
  },
  slice: {
    badge: 'SLICE',
    chip: 'bg-filter-soft text-filter-ink',
    head: 'text-filter-ink',
  },
} as const

/** WHERE / HAVING / GROUP BY / SELECT のステップノード */
export function StepNode({ data }: NodeProps<StepRfNode>) {
  const style = KIND_STYLE[data.kind]
  return (
    <div
      className={`relative min-w-[190px] rounded-2xl border-2 bg-node transition-[border-color,box-shadow] duration-300 ease-out ${
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
        className={`flex items-center gap-2 px-3 py-2 text-[13px] font-bold ${style.head}`}
      >
        <span
          className={`rounded-full px-2 py-0.5 text-[9.5px] font-extrabold tracking-[0.1em] ${style.chip}`}
        >
          {style.badge}
        </span>
        <span>{data.title}</span>
      </div>
      <div className="px-3 pt-0 pb-2.5 font-mono text-xs text-ink-muted">
        {data.body}
      </div>
      <Handle type="source" position={Position.Right} />
    </div>
  )
}
