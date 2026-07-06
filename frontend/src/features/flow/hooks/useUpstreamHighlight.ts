import { useCallback, useEffect, useState } from 'react'
import type { FlowGraph } from '../../../types/flow'
import { collectUpstream } from '../utils/highlight'

type UpstreamHighlight = {
  /** ハイライト中のノード ID 集合(なければ null)。描画側はここから見た目を導出する */
  activeIds: Set<string> | null
  /** 指定ノードとその上流経路をハイライトする。null で解除 */
  highlight: (seedIds: string[] | null) => void
}

/**
 * 「どのノードを起点にハイライトしているか」だけを状態として持つフック
 * ノードの isHighlighted は状態ではなく、activeIds からの純粋な導出(useMemo)にする
 */
export function useUpstreamHighlight(graph: FlowGraph | null): UpstreamHighlight {
  const [activeIds, setActiveIds] = useState<Set<string> | null>(null)

  // グラフが変わったら古いハイライトは意味を失うので解除する
  useEffect(() => {
    setActiveIds(null)
  }, [graph])

  const highlight = useCallback(
    (seedIds: string[] | null) => {
      setActiveIds(
        seedIds && graph ? collectUpstream(graph.edges, seedIds) : null,
      )
    },
    [graph],
  )

  return { activeIds, highlight }
}
