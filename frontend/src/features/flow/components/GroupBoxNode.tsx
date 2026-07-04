import type { NodeProps } from '@xyflow/react'
import type { GroupRfNode } from '../types'

/** WITH 句などノードをまとめる背景枠(操作対象ではない) */
export function GroupBoxNode({ data }: NodeProps<GroupRfNode>) {
  return (
    <div className="relative h-full w-full rounded-3xl border-2 border-dashed border-accent-soft bg-accent/4">
      <span className="absolute -top-[11px] left-4 rounded-full bg-canvas px-2.5 font-mono text-[11px] font-semibold text-accent-ink/70">
        {data.label}
      </span>
    </div>
  )
}
