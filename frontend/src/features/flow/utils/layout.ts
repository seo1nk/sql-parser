import type { LayoutOptions } from 'elkjs'
import ELK from 'elkjs/lib/elk.bundled.js'
import type { FlowGraph } from '../../../types/flow'
import type { GroupRfNode, RfEdge, RfNode } from '../types'
import { estimateSize } from './toReactFlow'

// Liam の getElkLayout に倣った層状レイアウト(左→右 = 論理実行順)
const layoutOptions: LayoutOptions = {
  'elk.algorithm': 'layered',
  'elk.direction': 'RIGHT',
  'elk.layered.spacing.baseValue': '48',
  'elk.spacing.nodeNode': '56',
  'elk.layered.spacing.nodeNodeBetweenLayers': '104',
  'elk.layered.mergeEdges': 'true',
  'elk.layered.considerModelOrder.strategy': 'PREFER_EDGES',
}

const elk = new ELK()

/** ELK でノード位置を計算して返す(groupBox はレイアウト対象外) */
export async function applyElkLayout(
  nodes: RfNode[],
  edges: RfEdge[],
): Promise<RfNode[]> {
  const targets = nodes.filter((n) => n.type !== 'groupBox')
  const graph = {
    id: 'root',
    layoutOptions,
    children: targets.map((n) => ({ id: n.id, ...estimateSize(n) })),
    edges: edges.map((e) => ({
      id: e.id,
      sources: [e.source],
      targets: [e.target],
    })),
  }

  const result = await elk.layout(graph)
  const positions = new Map(
    (result.children ?? []).map((c) => [c.id, { x: c.x ?? 0, y: c.y ?? 0 }]),
  )

  return nodes.map((n) => {
    const position = positions.get(n.id)
    return position ? { ...n, position } : n
  })
}

const GROUP_PADDING = { top: 44, right: 28, bottom: 24, left: 28 }

/** レイアウト済みノードから WITH 句などのグループ枠を計算して先頭に追加する */
export function addGroupBoxes(graph: FlowGraph, nodes: RfNode[]): RfNode[] {
  const boxes = graph.groups.flatMap((group): GroupRfNode[] => {
    const memberIds = new Set(
      graph.nodes.filter((n) => n.groupId === group.id).map((n) => n.id),
    )
    const members = nodes.filter((n) => memberIds.has(n.id))
    if (members.length === 0) return []

    const minX = Math.min(...members.map((n) => n.position.x))
    const minY = Math.min(...members.map((n) => n.position.y))
    const maxX = Math.max(
      ...members.map((n) => n.position.x + estimateSize(n).width),
    )
    const maxY = Math.max(
      ...members.map((n) => n.position.y + estimateSize(n).height),
    )

    return [
      {
        id: `group:${group.id}`,
        type: 'groupBox',
        position: {
          x: minX - GROUP_PADDING.left,
          y: minY - GROUP_PADDING.top,
        },
        width: maxX - minX + GROUP_PADDING.left + GROUP_PADDING.right,
        height: maxY - minY + GROUP_PADDING.top + GROUP_PADDING.bottom,
        data: { label: group.label },
        draggable: false,
        selectable: false,
        focusable: false,
        zIndex: -1,
      },
    ]
  })

  return [...boxes, ...nodes]
}
