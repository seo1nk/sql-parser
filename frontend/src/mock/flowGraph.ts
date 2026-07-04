import type { FlowGraph } from '../types/flow'

/** モックで可視化するサンプルクエリ(FROM-first 構文) */
export const mockSql = `WITH adults AS (
  FROM users
  WHERE age >= 20
  SELECT id, name
)
FROM adults a
JOIN orders o
  ON a.id = o.user_id
WHERE o.price > 100
GROUP BY a.name
SELECT a.name,
  count(o.id) AS order_count`

/**
 * 上記クエリを Rust パーサーの explain() にかけた場合に返る想定のフローグラフ。
 * パーサー(フェーズ3)と WASM API(フェーズ4)が完成したら実出力に差し替える。
 */
export const mockFlowGraph: FlowGraph = {
  groups: [{ id: 'cte-adults', label: '① WITH adults AS ( … )' }],
  nodes: [
    {
      id: 'users',
      kind: 'scan',
      label: 'users',
      columns: [
        { name: 'id', role: 'used' },
        { name: 'name', role: 'output' },
        { name: 'age', role: 'used' },
      ],
      hasMore: true,
      groupId: 'cte-adults',
    },
    {
      id: 'filter-age',
      kind: 'filter',
      phase: 'where',
      predicate: 'age >= 20',
      groupId: 'cte-adults',
    },
    {
      id: 'project-adults',
      kind: 'project',
      items: ['id', 'name'],
      distinct: false,
      groupId: 'cte-adults',
    },
    {
      id: 'adults',
      kind: 'cte',
      label: 'adults',
      alias: 'a',
      columns: [
        { name: 'id', role: 'used' },
        { name: 'name', role: 'output' },
      ],
      hasMore: false,
      groupId: 'cte-adults',
    },
    {
      id: 'orders',
      kind: 'scan',
      label: 'orders',
      alias: 'o',
      columns: [
        { name: 'id', role: 'output' },
        { name: 'user_id', role: 'used' },
        { name: 'price', role: 'used' },
      ],
      hasMore: true,
    },
    {
      id: 'joined',
      kind: 'joined',
      label: 'adults ⋈ orders',
      joinType: 'INNER',
      columns: [
        { name: 'a.id', role: 'used' },
        { name: 'a.name', role: 'output' },
        { name: 'o.id', role: 'output' },
        { name: 'o.user_id', role: 'used' },
        { name: 'o.price', role: 'used' },
      ],
      hasMore: true,
    },
    {
      id: 'filter-price',
      kind: 'filter',
      phase: 'where',
      predicate: 'o.price > 100',
    },
    { id: 'group-name', kind: 'group', keys: ['a.name'] },
    {
      id: 'result',
      kind: 'result',
      columns: [
        { name: 'name', role: 'output' },
        { name: 'order_count', role: 'output' },
      ],
    },
  ],
  edges: [
    { source: 'users', target: 'filter-age' },
    { source: 'filter-age', target: 'project-adults' },
    { source: 'project-adults', target: 'adults' },
    { source: 'adults', target: 'joined', label: 'a.id' },
    { source: 'orders', target: 'joined', label: 'o.user_id' },
    { source: 'joined', target: 'filter-price' },
    { source: 'filter-price', target: 'group-name' },
    { source: 'group-name', target: 'result' },
  ],
  timeline: [
    { order: 1, label: '① WITH', nodeIds: ['adults'] },
    { order: 2, label: '② FROM', nodeIds: ['adults', 'orders'] },
    { order: 3, label: '③ JOIN', nodeIds: ['joined'] },
    { order: 4, label: '④ WHERE', nodeIds: ['filter-price'] },
    { order: 5, label: '⑤ GROUP BY', nodeIds: ['group-name'] },
    { order: 6, label: '⑥ SELECT', nodeIds: ['result'] },
  ],
}
