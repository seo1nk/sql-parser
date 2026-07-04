import {
  BaseEdge,
  EdgeLabelRenderer,
  getBezierPath,
  type EdgeProps,
} from '@xyflow/react'
import type { RfEdge } from '../types'

/**
 * データフローを表すエッジ
 * - ハイライト時はピンクにやわらかく発光
 * - エッジ上をパーティクルが流れる
 * - label には JOIN の結合キーなどを表示する
 */
export function FlowEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  data,
  markerEnd,
}: EdgeProps<RfEdge>) {
  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  })
  const highlighted = data?.isHighlighted ?? false

  return (
    <>
      <BaseEdge
        id={id}
        path={edgePath}
        markerEnd={markerEnd}
        style={{
          stroke: highlighted ? '#ff7dae' : '#dbc7d2',
          strokeWidth: 2,
          transition: 'stroke 300ms ease-out',
          filter: highlighted
            ? 'drop-shadow(0 2px 4px rgba(255,125,174,0.45))'
            : undefined,
        }}
      />
      {[0, 1].map((i) => (
        <circle
          key={i}
          r={3}
          className="flow-particle"
          fill={highlighted ? '#ff7dae' : '#e9c4d5'}
        >
          <animateMotion
            dur="3.2s"
            begin={`${i * 1.6}s`}
            repeatCount="indefinite"
            path={edgePath}
          />
        </circle>
      ))}
      {data?.label && (
        <EdgeLabelRenderer>
          <div
            style={{
              transform: `translate(-50%, -50%) translate(${labelX}px, ${labelY}px)`,
            }}
            className="nodrag nopan pointer-events-none absolute rounded-full border-2 border-join-soft bg-node px-2 py-0.5 font-mono text-[11px] font-semibold text-join-ink"
          >
            {data.label}
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  )
}
