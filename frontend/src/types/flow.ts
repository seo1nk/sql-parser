/**
 * フローグラフの型定義
 *
 * 将来 Rust パーサー(WASM)の explain() が返す JSON の契約。
 * 現状はモックデータ(src/mock/flowGraph.ts)がこの型を満たす。
 * docs/06_api_design.md / docs/07_ui_design.md を参照。
 */

/** 列の系譜(lineage)における役割 */
export type ColumnRole =
  /** 最終結果に値が届く列(集計関数経由を含む) */
  | 'output'
  /** 結合キー・WHERE / GROUP BY などの条件でのみ使われる列 */
  | 'used'

export type Column = {
  name: string
  role: ColumnRole
}

export type JoinType = 'INNER' | 'LEFT' | 'RIGHT' | 'FULL' | 'CROSS'

/** グラフのノード。kind ごとに描画コンポーネントが変わる */
export type FlowNode =
  /** FROM で参照される実テーブル */
  | {
      id: string
      kind: 'scan'
      label: string
      alias?: string
      columns: Column[]
      /** SQL に現れていない列が存在しうる場合 true(列リスト末尾に … を描く) */
      hasMore: boolean
      groupId?: string
    }
  /** WITH で定義された共通テーブル式 */
  | {
      id: string
      kind: 'cte'
      label: string
      alias?: string
      columns: Column[]
      hasMore: boolean
      groupId?: string
    }
  /** JOIN の合流結果(JOIN はノードではなく合流として描く) */
  | {
      id: string
      kind: 'joined'
      label: string
      joinType: JoinType
      columns: Column[]
      hasMore: boolean
      groupId?: string
    }
  /** WHERE / HAVING による絞り込み */
  | {
      id: string
      kind: 'filter'
      phase: 'where' | 'having'
      predicate: string
      groupId?: string
    }
  /** GROUP BY によるグループ化 */
  | { id: string; kind: 'group'; keys: string[]; groupId?: string }
  /** 途中の SELECT(サブクエリ・CTE 内の射影) */
  | {
      id: string
      kind: 'project'
      items: string[]
      distinct: boolean
      groupId?: string
    }
  /** ORDER BY による並び替え */
  | { id: string; kind: 'sort'; keys: string[]; groupId?: string }
  /** LIMIT / OFFSET による切り出し */
  | {
      id: string
      kind: 'slice'
      limit?: number
      offset?: number
      groupId?: string
    }
  /** 最終結果のテーブル */
  | {
      id: string
      kind: 'result'
      columns: Column[]
      hasMore: boolean
      groupId?: string
    }

export type FlowNodeKind = FlowNode['kind']

/** ノード間のエッジ。label は JOIN の結合キーなど矢印上に表示する文字列 */
export type FlowEdgeData = {
  source: string
  target: string
  label?: string
}

/** WITH 句などノードをまとめる枠 */
export type FlowGroup = {
  id: string
  label: string
}

/** 論理実行順タイムラインの 1 ステップ */
export type TimelineStep = {
  order: number
  label: string
  nodeIds: string[]
}

export type FlowGraph = {
  nodes: FlowNode[]
  edges: FlowEdgeData[]
  groups: FlowGroup[]
  timeline: TimelineStep[]
}
