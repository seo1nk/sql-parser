import { MarkerType } from '@xyflow/react'
import type { FlowEdgeData } from '../../../types/flow'
import type { RfEdge, RfNode } from '../types'

/**
 * 指定ノードに流れ込む上流経路(自分自身を含む)のノード ID 集合を返す
 * (Liam の highlightNodesAndEdges と同じ発想の純粋関数)
 */
export function collectUpstream(
  edges: Pick<FlowEdgeData, 'source' | 'target'>[],
  seedIds: string[],
): Set<string> {
  const upstream = new Map<string, string[]>()
  for (const edge of edges) {
    const sources = upstream.get(edge.target)
    if (sources) {
      sources.push(edge.source)
    } else {
      upstream.set(edge.target, [edge.source])
    }
  }

  const acc = new Set<string>()
  const visit = (id: string) => {
    if (acc.has(id)) return
    acc.add(id)
    for (const source of upstream.get(id) ?? []) visit(source)
  }
  for (const id of seedIds) visit(id)
  return acc
}

/** ハイライト集合(null で解除)をノードに反映する */
export function highlightNodes(
  nodes: RfNode[],
  active: Set<string> | null,
): RfNode[] {
  return nodes.map((n) => {
    if (n.type === 'groupBox') return n
    return {
      ...n,
      data: { ...n.data, isHighlighted: active?.has(n.id) ?? false },
    } as RfNode
  })
}

/** ハイライト集合(null で解除)をエッジに反映する(両端が対象のときだけ光る) */
export function highlightEdges(
  edges: RfEdge[],
  active: Set<string> | null,
): RfEdge[] {
  return edges.map((e) => {
    const on = (active?.has(e.source) && active?.has(e.target)) ?? false
    return {
      ...e,
      data: { label: e.data?.label, isHighlighted: on },
      zIndex: on ? 1 : 0,
      markerEnd: {
        type: MarkerType.ArrowClosed,
        width: 12,
        height: 12,
        color: on ? '#ff7dae' : '#dbc7d2',
      },
    }
  })
}
