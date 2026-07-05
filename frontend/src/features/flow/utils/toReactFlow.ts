import type { FlowGraph } from '../../../types/flow'
import type { RfEdge, RfNode } from '../types'

const CIRCLED = ['①', '②', '③', '④', '⑤', '⑥', '⑦', '⑧', '⑨', '⑩']

/** FlowGraph(explain() の出力契約)を React Flow のノード/エッジに変換する純粋関数 */
export function toReactFlow(graph: FlowGraph): {
  nodes: RfNode[]
  edges: RfEdge[]
} {
  // 各ノードの論理実行順バッジ。タイムラインで最初に登場するステップの番号を使う。
  // WITH 枠内のノードは枠ラベル側に番号があるため付けない
  const stepNoById = new Map<string, string>()
  for (const step of [...graph.timeline].sort((a, b) => a.order - b.order)) {
    for (const id of step.nodeIds) {
      if (!stepNoById.has(id)) {
        stepNoById.set(id, CIRCLED[step.order - 1] ?? String(step.order))
      }
    }
  }

  const nodes = graph.nodes.map((n): RfNode => {
    const stepNo = n.groupId ? undefined : stepNoById.get(n.id)
    const position = { x: 0, y: 0 } // レイアウトは ELK が決める
    switch (n.kind) {
      case 'filter':
        return {
          id: n.id,
          type: 'step',
          position,
          data: {
            kind: 'filter',
            title: n.phase === 'where' ? 'WHERE' : 'HAVING',
            body: n.predicate,
            stepNo,
            isHighlighted: false,
          },
        }
      case 'group':
        return {
          id: n.id,
          type: 'step',
          position,
          data: {
            kind: 'group',
            title: 'GROUP BY',
            body: n.keys.join(', '),
            stepNo,
            isHighlighted: false,
          },
        }
      case 'project':
        return {
          id: n.id,
          type: 'step',
          position,
          data: {
            kind: 'project',
            title: 'SELECT',
            body: `${n.distinct ? 'DISTINCT ' : ''}${n.items.join(', ')}`,
            stepNo,
            isHighlighted: false,
          },
        }
      case 'sort':
        return {
          id: n.id,
          type: 'step',
          position,
          data: {
            kind: 'sort',
            title: 'ORDER BY',
            body: n.keys.join(', '),
            stepNo,
            isHighlighted: false,
          },
        }
      case 'slice':
        return {
          id: n.id,
          type: 'step',
          position,
          data: {
            kind: 'slice',
            title: 'LIMIT / OFFSET',
            body: [
              n.limit != null ? `LIMIT ${n.limit}` : null,
              n.offset != null ? `OFFSET ${n.offset}` : null,
            ]
              .filter(Boolean)
              .join(' '),
            stepNo,
            isHighlighted: false,
          },
        }
      case 'result':
        return {
          id: n.id,
          type: 'table',
          position,
          data: {
            kind: 'result',
            title: '結果',
            columns: n.columns,
            hasMore: n.hasMore,
            stepNo,
            isHighlighted: false,
          },
        }
      case 'joined':
        return {
          id: n.id,
          type: 'table',
          position,
          data: {
            kind: 'joined',
            title: n.label,
            joinType: n.joinType,
            columns: n.columns,
            hasMore: n.hasMore,
            stepNo,
            isHighlighted: false,
          },
        }
      default:
        return {
          id: n.id,
          type: 'table',
          position,
          data: {
            kind: n.kind,
            title: n.alias != null ? `${n.label} (${n.alias})` : n.label,
            columns: n.columns,
            hasMore: n.hasMore,
            stepNo,
            isHighlighted: false,
          },
        }
    }
  })

  const edges = graph.edges.map(
    (e): RfEdge => ({
      id: `${e.source}->${e.target}`,
      source: e.source,
      target: e.target,
      type: 'flow',
      data: { label: e.label, isHighlighted: false },
    }),
  )

  return { nodes, edges }
}

/** ELK・グループ枠計算用のサイズ見積もり(実測に頼らないことで初期レイアウトを単純にする) */
export function estimateSize(node: RfNode): { width: number; height: number } {
  if (node.type === 'step') {
    return { width: 190, height: 64 }
  }
  if (node.type === 'groupBox') {
    return { width: node.width ?? 0, height: node.height ?? 0 }
  }
  const rows = node.data.columns.length + (node.data.hasMore ? 1 : 0)
  return { width: 208, height: 38 + rows * 23 + 12 }
}
