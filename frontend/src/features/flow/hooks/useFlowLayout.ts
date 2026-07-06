import {
  applyEdgeChanges,
  applyNodeChanges,
  useReactFlow,
  type OnEdgesChange,
  type OnNodesChange,
} from '@xyflow/react'
import { useCallback, useEffect, useState } from 'react'
import type { FlowGraph } from '../../../types/flow'
import type { RfEdge, RfNode } from '../types'
import { addGroupBoxes, applyElkLayout } from '../utils/layout'
import { toReactFlow } from '../utils/toReactFlow'

type FlowLayout = {
  /** レイアウト済みのノード(ハイライトなどの見た目の導出は含まない) */
  nodes: RfNode[]
  edges: RfEdge[]
  onNodesChange: OnNodesChange<RfNode>
  onEdgesChange: OnEdgesChange<RfEdge>
}

/**
 * フローグラフを ELK でレイアウトし、React Flow のノード/エッジ状態として保持するフック
 * グラフが変わるたびに「変換 → 非同期レイアウト → グループ枠計算 → fitView」を行う
 */
export function useFlowLayout(graph: FlowGraph | null): FlowLayout {
  const [nodes, setNodes] = useState<RfNode[]>([])
  const [edges, setEdges] = useState<RfEdge[]>([])
  const { fitView } = useReactFlow()

  useEffect(() => {
    if (!graph) return
    let cancelled = false
    ;(async () => {
      const { nodes: rawNodes, edges: rawEdges } = toReactFlow(graph)
      const laidOut = await applyElkLayout(rawNodes, rawEdges)
      // レイアウト中に新しいグラフが来ていたら捨てる
      if (cancelled) return
      setNodes(addGroupBoxes(graph, laidOut))
      setEdges(rawEdges)
      requestAnimationFrame(() => fitView({ padding: 0.1 }))
    })()
    return () => {
      cancelled = true
    }
  }, [graph, fitView])

  // ドラッグなど React Flow 側の操作をベースの状態に反映する
  const onNodesChange: OnNodesChange<RfNode> = useCallback(
    (changes) => setNodes((current) => applyNodeChanges(changes, current)),
    [],
  )
  const onEdgesChange: OnEdgesChange<RfEdge> = useCallback(
    (changes) => setEdges((current) => applyEdgeChanges(changes, current)),
    [],
  )

  return { nodes, edges, onNodesChange, onEdgesChange }
}
