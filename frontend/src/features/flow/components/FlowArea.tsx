import {
  Background,
  BackgroundVariant,
  ReactFlow,
  useEdgesState,
  useNodesState,
  useReactFlow,
  type NodeTypes,
  type EdgeTypes,
} from '@xyflow/react'
import { useCallback, useEffect } from 'react'
import { mockFlowGraph } from '../../../mock/flowGraph'
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

/** フローキャンバスと実行順タイムライン。ReactFlowProvider の内側で使うこと */
export function FlowArea() {
  const [nodes, setNodes, onNodesChange] = useNodesState<RfNode>([])
  const [edges, setEdges, onEdgesChange] = useEdgesState<RfEdge>([])
  const { fitView } = useReactFlow()

  // FlowGraph → React Flow 変換 → ELK レイアウト → グループ枠追加
  useEffect(() => {
    let cancelled = false
    ;(async () => {
      const { nodes: rawNodes, edges: rawEdges } = toReactFlow(mockFlowGraph)
      const laidOut = await applyElkLayout(rawNodes, rawEdges)
      const withBoxes = addGroupBoxes(mockFlowGraph, laidOut)
      if (cancelled) return
      setNodes(highlightNodes(withBoxes, null))
      setEdges(highlightEdges(rawEdges, null))
      requestAnimationFrame(() => fitView({ padding: 0.1 }))
    })()
    return () => {
      cancelled = true
    }
  }, [setNodes, setEdges, fitView])

  // 指定ノード(とその上流経路)をハイライトする。null で解除
  const highlight = useCallback(
    (seedIds: string[] | null) => {
      const active = seedIds
        ? collectUpstream(mockFlowGraph.edges, seedIds)
        : null
      setNodes((current) => highlightNodes(current, active))
      setEdges((current) => highlightEdges(current, active))
    },
    [setNodes, setEdges],
  )

  const focusStep = useCallback(
    (nodeIds: string[]) => {
      const targets = collectUpstream(mockFlowGraph.edges, nodeIds)
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
          colorMode="dark"
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
            size={1}
            bgColor="#0d0f0f"
            color="rgba(255,255,255,0.09)"
          />
        </ReactFlow>
      </div>
      <Timeline
        steps={mockFlowGraph.timeline}
        onHover={highlight}
        onSelect={focusStep}
      />
    </div>
  )
}
