import type { NodeProps } from '@xyflow/react'
import type { GroupRfNode } from '../types'

/** WITH 句などノードをまとめる背景枠(操作対象ではない) */
export function GroupBoxNode({ data }: NodeProps<GroupRfNode>) {
  return (
    <div className="relative h-full w-full rounded-[10px] border border-dashed border-white/18 bg-white/2">
      <span className="absolute -top-[9px] left-3.5 bg-canvas px-2 font-mono text-[11px] text-fg-dim">
        {data.label}
      </span>
    </div>
  )
}
