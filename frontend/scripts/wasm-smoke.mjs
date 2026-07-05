// WASM バイナリ経由で explain() が動くことを確認するスモークテスト
// 実行: node scripts/wasm-smoke.mjs
import { readFileSync } from 'node:fs'
import init, { explain, parse } from '../src/wasm/pkg/wasm_api.js'

const wasmBytes = readFileSync(
  new URL('../src/wasm/pkg/wasm_api_bg.wasm', import.meta.url),
)
await init({ module_or_path: wasmBytes })

const sql = `WITH adults AS (FROM users WHERE age >= 20 SELECT id, name)
FROM adults a JOIN orders o ON a.id = o.user_id
WHERE o.price > 100 GROUP BY a.name
SELECT a.name, count(o.id) AS order_count`

const result = JSON.parse(explain(sql))
if (!result.ok) throw new Error(`explain failed: ${result.error}`)
const graph = result.value

console.log(`nodes: ${graph.nodes.length}, edges: ${graph.edges.length}, groups: ${graph.groups.length}`)
console.log('timeline:', graph.timeline.map((s) => s.label).join(' → '))
for (const node of graph.nodes) {
  const cols = node.columns
    ? ` [${node.columns.map((c) => `${c.name}:${c.role}`).join(', ')}]${node.hasMore ? ' …' : ''}`
    : ''
  console.log(`  ${node.kind}\t${node.id}${cols}`)
}
for (const edge of graph.edges.filter((e) => e.label)) {
  console.log(`  edge ${edge.source} -> ${edge.target} [${edge.label}]`)
}

// 失敗ケース: エラーメッセージが返る
const bad = JSON.parse(explain('SELECT FROM WHERE'))
if (bad.ok) throw new Error('expected failure')
console.log('error case:', bad.error)

// parse() も動く
const ast = JSON.parse(parse('FROM users SELECT id'))
if (!ast.ok) throw new Error('parse failed')
console.log('parse: ok (with =', JSON.stringify(ast.value.with), ')')

console.log('wasm smoke test: OK')
