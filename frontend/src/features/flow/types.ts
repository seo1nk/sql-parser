import type { Edge, Node } from '@xyflow/react'
import type { Column, JoinType } from '../../types/flow'

/** テーブル系ノード(scan / cte / joined / result)の描画データ */
export type TableNodeData = {
  kind: 'scan' | 'cte' | 'joined' | 'result'
  title: string
  joinType?: JoinType
  columns: Column[]
  hasMore: boolean
  /** 論理実行順バッジ(例: '③')。WITH 枠内のノードは持たない */
  stepNo?: string
  isHighlighted: boolean
}

/** ステップ系ノード(filter / group / project)の描画データ */
export type StepNodeData = {
  kind: 'filter' | 'group' | 'project'
  title: string
  body: string
  stepNo?: string
  isHighlighted: boolean
}

/** WITH 句などの背景枠 */
export type GroupBoxData = {
  label: string
}

export type EdgeData = {
  label?: string
  isHighlighted: boolean
}

export type TableRfNode = Node<TableNodeData, 'table'>
export type StepRfNode = Node<StepNodeData, 'step'>
export type GroupRfNode = Node<GroupBoxData, 'groupBox'>
export type RfNode = TableRfNode | StepRfNode | GroupRfNode
export type RfEdge = Edge<EdgeData, 'flow'>
