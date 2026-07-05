import {
  Background,
  BackgroundVariant,
  ReactFlow,
  useEdgesState,
  useNodesState,
  useReactFlow,
  type EdgeTypes,
  type NodeTypes,
} from '@xyflow/react'
import { useCallback, useEffect, useRef } from 'react'
import type { FlowGraph } from '../../../types/flow'
import type { RfEdge, RfNode } from '../types'
import {
  collectUpstream,
  highlightEdges,
  highlightNodes,
} from '../utils/highlight'
import { addGroupBoxes, applyElkLayout } from '../utils/layout'
import { toReactFlow } from '../utils/toReactFlow'
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
  /** WASM の explain() が返したフローグラフ。null は初期ロード中 */
  graph: FlowGraph | null
}

/** フローキャンバスと実行順タイムライン。ReactFlowProvider の内側で使うこと */
export function FlowArea({ graph }: Props) {
  const [nodes, setNodes, onNodesChange] = useNodesState<RfNode>([])
  const [edges, setEdges, onEdgesChange] = useEdgesState<RfEdge>([])
  const { fitView } = useReactFlow()
  // ハイライト計算・古い非同期レイアウト結果の破棄に使う
  const graphRef = useRef<FlowGraph | null>(null)

  // FlowGraph → React Flow 変換 → ELK レイアウト → グループ枠追加
  useEffect(() => {
    if (!graph) return
    graphRef.current = graph
    ;(async () => {
      const { nodes: rawNodes, edges: rawEdges } = toReactFlow(graph)
      const laidOut = await applyElkLayout(rawNodes, rawEdges)
      // レイアウト中に新しいグラフが来ていたら捨てる
      if (graphRef.current !== graph) return
      const withBoxes = addGroupBoxes(graph, laidOut)
      setNodes(highlightNodes(withBoxes, null))
      setEdges(highlightEdges(rawEdges, null))
      requestAnimationFrame(() => fitView({ padding: 0.1 }))
    })()
  }, [graph, setNodes, setEdges, fitView])

  // 指定ノード(とその上流経路)をハイライトする。null で解除
  const highlight = useCallback(
    (seedIds: string[] | null) => {
      const current = graphRef.current
      if (!current) return
      const active = seedIds ? collectUpstream(current.edges, seedIds) : null
      setNodes((nodes) => highlightNodes(nodes, active))
      setEdges((edges) => highlightEdges(edges, active))
    },
    [setNodes, setEdges],
  )

  const focusStep = useCallback(
    (nodeIds: string[]) => {
      const current = graphRef.current
      if (!current) return
      const targets = collectUpstream(current.edges, nodeIds)
      fitView({
        nodes: [...targets].map((id) => ({ id })),
        padding: 0.3,
        duration: 400,
      })
    },
    [fitView],
  )

  return (
    <div className="flex min-w-0 flex-1 flex-col">
      <div className="min-h-0 flex-1">
        <ReactFlow
          colorMode="light"
          nodes={nodes}
          edges={edges}
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
