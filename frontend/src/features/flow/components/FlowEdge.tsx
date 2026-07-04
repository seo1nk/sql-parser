import {
  BaseEdge,
  EdgeLabelRenderer,
  getBezierPath,
  type EdgeProps,
} from '@xyflow/react'
import type { RfEdge } from '../types'

/**
 * データフローを表すエッジ
 * - ハイライト時は緑グロー
 * - エッジ上をパーティクルが流れる(Liam の RelationshipEdge の演出を踏襲)
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
          stroke: highlighted ? '#1ded83' : '#4a4f51',
          strokeWidth: 1.5,
          transition: 'stroke 300ms ease-out',
          filter: highlighted
            ? 'drop-shadow(0 0 4px rgba(29,237,131,0.4))'
            : undefined,
        }}
      />
      {[0, 1].map((i) => (
        <circle
          key={i}
          r={2.4}
          className="flow-particle"
          fill={highlighted ? '#1ded83' : 'rgba(255,255,255,0.35)'}
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
            className="nodrag nopan pointer-events-none absolute rounded border border-pane-border bg-canvas px-1.5 py-0.5 font-mono text-[11px] text-[#dfb8ff]"
          >
            {data.label}
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  )
}
