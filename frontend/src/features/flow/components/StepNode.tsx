import { Handle, Position, type NodeProps } from '@xyflow/react'
import type { StepRfNode } from '../types'
import { StepBadge } from './StepBadge'

const KIND_STYLE = {
  filter: { badge: 'FILTER', badgeBg: 'bg-step-filter', head: 'text-[#ffd77e]' },
  group: { badge: 'GROUP', badgeBg: 'bg-step-group', head: 'text-[#a9bcf5]' },
  project: {
    badge: 'PROJECT',
    badgeBg: 'bg-step-project',
    head: 'text-[#8ef7c0]',
  },
} as const

/** WHERE / HAVING / GROUP BY / SELECT のステップノード */
export function StepNode({ data }: NodeProps<StepRfNode>) {
  const style = KIND_STYLE[data.kind]
  return (
    <div
      className={`relative min-w-[190px] rounded-md border bg-node shadow-[0_0_20px_rgba(0,0,0,0.4)] transition-[border-color,box-shadow] duration-300 ease-out ${
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
        className={`flex items-center gap-2 px-3 py-2 text-[13px] font-semibold ${style.head}`}
      >
        <span
          className={`rounded-[3px] px-1.5 py-0.5 text-[9.5px] font-bold tracking-[0.1em] text-[#0c0d0d] ${style.badgeBg}`}
        >
          {style.badge}
        </span>
        <span>{data.title}</span>
      </div>
      <div className="px-3 pt-0 pb-2.5 font-mono text-xs text-fg-muted">
        {data.body}
      </div>
      <Handle type="source" position={Position.Right} />
    </div>
  )
}
