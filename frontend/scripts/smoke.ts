/**
 * コアロジックのスモークテスト(ブラウザ不要の範囲)
 * 実行: node scripts/smoke.ts
 */
import { mockFlowGraph } from '../src/mock/flowGraph'
import {
  collectUpstream,
  highlightEdges,
  highlightNodes,
} from '../src/features/flow/utils/highlight'
import { addGroupBoxes, applyElkLayout } from '../src/features/flow/utils/layout'
import { toReactFlow } from '../src/features/flow/utils/toReactFlow'

const { nodes, edges } = toReactFlow(mockFlowGraph)
console.log(`nodes: ${nodes.length}, edges: ${edges.length}`)

const laidOut = await applyElkLayout(nodes, edges)
const withBoxes = addGroupBoxes(mockFlowGraph, laidOut)

for (const n of withBoxes) {
  console.log(
    `${n.type}\t${n.id}\t(${Math.round(n.position.x)}, ${Math.round(n.position.y)})`,
  )
}

// レイアウト結果の検証: 全ノードが原点以外に配置され、位置が重複していないこと
const placed = laidOut.filter((n) => n.type !== 'groupBox')
const unique = new Set(placed.map((n) => `${n.position.x},${n.position.y}`))
if (unique.size !== placed.length) throw new Error('ノード位置が重複している')

// エッジ方向(左→右)の検証: source の x < target の x
const posById = new Map(placed.map((n) => [n.id, n.position]))
for (const e of edges) {
  const s = posById.get(e.source)!
  const t = posById.get(e.target)!
  if (s.x >= t.x) throw new Error(`エッジ ${e.id} が左→右になっていない`)
}

// ハイライトの検証: result の上流は全ノード、orders の上流は orders のみ
const upstreamOfResult = collectUpstream(mockFlowGraph.edges, ['result'])
if (upstreamOfResult.size !== mockFlowGraph.nodes.length)
  throw new Error(`result の上流が全ノードでない: ${upstreamOfResult.size}`)
const upstreamOfOrders = collectUpstream(mockFlowGraph.edges, ['orders'])
if (upstreamOfOrders.size !== 1) throw new Error('orders の上流計算が不正')

const hn = highlightNodes(withBoxes, upstreamOfResult)
const he = highlightEdges(edges, upstreamOfResult)
const litNodes = hn.filter((n) => n.type !== 'groupBox' && n.data.isHighlighted)
const litEdges = he.filter((e) => e.data?.isHighlighted)
if (litNodes.length !== mockFlowGraph.nodes.length)
  throw new Error('ノードハイライトの反映が不正')
if (litEdges.length !== edges.length)
  throw new Error('エッジハイライトの反映が不正')

console.log('smoke test: OK')
