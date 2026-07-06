import {
  Background,
  BackgroundVariant,
  ReactFlow,
  useReactFlow,
  type EdgeTypes,
  type NodeTypes,
} from '@xyflow/react'
import { useCallback, useMemo } from 'react'
import type { FlowGraph } from '../../../types/flow'
import { useFlowLayout } from '../hooks/useFlowLayout'
import { useUpstreamHighlight } from '../hooks/useUpstreamHighlight'
import {
  collectUpstream,
  highlightEdges,
  highlightNodes,
} from '../utils/highlight'
import { FlowEdge } from './FlowEdge'
import { GroupBoxNode } from './GroupBoxNode'
import { StepNode } from './StepNode'
import { TableNode } from './TableNode'
import { Timeline } from './Timeline'

const nodeTypes: NodeTypes = {
  table: TableNode,
  step: StepNode,
  groupBox: GroupBoxNode,
}
const edgeTypes: EdgeTypes = {
  flow: FlowEdge,
}

type Props = {
  /** explain() が返したフローグラフ。null は初期ロード中 */
  graph: FlowGraph | null
}

/**
 * フローキャンバスと実行順タイムライン。ReactFlowProvider の内側で使うこと
 *
 * 状態は「レイアウト済みノード(useFlowLayout)」と
 * 「ハイライト起点(useUpstreamHighlight)」の2つだけで、
 * 実際に描画するノード/エッジはそこからの純粋な導出(useMemo)にしている
 */
export function FlowArea({ graph }: Props) {
  const { nodes, edges, onNodesChange, onEdgesChange } = useFlowLayout(graph)
  const { activeIds, highlight } = useUpstreamHighlight(graph)
  const { fitView } = useReactFlow()

  // UI = f(レイアウト済みノード, ハイライト状態)
  const displayNodes = useMemo(
    () => highlightNodes(nodes, activeIds),
    [nodes, activeIds],
  )
  const displayEdges = useMemo(
    () => highlightEdges(edges, activeIds),
    [edges, activeIds],
  )

  const focusStep = useCallback(
    (nodeIds: string[]) => {
      if (!graph) return
      const targets = collectUpstream(graph.edges, nodeIds)
      fitView({
        nodes: [...targets].map((id) => ({ id })),
        padding: 0.3,
        duration: 400,
      })
    },
    [graph, fitView],
  )

  return (
    <div className="flex min-w-0 flex-1 flex-col">
      <div className="min-h-0 flex-1">
        <ReactFlow
          colorMode="light"
          nodes={displayNodes}
          edges={displayEdges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          nodeTypes={nodeTypes}
          edgeTypes={edgeTypes}
          minZoom={0.2}
          maxZoom={2}
          panOnScroll
          nodesConnectable={false}
          onNodeMouseEnter={(_, node) => {
            if (node.type !== 'groupBox') highlight([node.id])
          }}
          onNodeMouseLeave={() => highlight(null)}
        >
          <Background
            variant={BackgroundVariant.Dots}
            gap={22}
            size={1.5}
            bgColor="#fff7f9"
            color="#f3d9e1"
          />
        </ReactFlow>
      </div>
      <Timeline
        steps={graph?.timeline ?? []}
        onHover={highlight}
        onSelect={focusStep}
      />
    </div>
  )
}
