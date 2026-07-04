import type { ReactNode } from 'react'
import { mockSql } from '../mock/flowGraph'

const LEGEND = [
  { color: 'bg-step-scan', label: 'SCAN / CTE — 集合の形成 (FROM / WITH)' },
  { color: 'bg-step-join', label: 'JOINED — 合流して 1 つの集合になる' },
  { color: 'bg-step-filter', label: 'FILTER — 絞り込み (WHERE / HAVING)' },
  { color: 'bg-step-group', label: 'GROUP — グループ化 (GROUP BY)' },
  { color: 'bg-step-project', label: 'PROJECT — 抽出 (SELECT)' },
]

const KEYWORD_PATTERN =
  /\b(WITH|AS|FROM|WHERE|GROUP BY|HAVING|ORDER BY|SELECT|JOIN|ON|AND|OR|NOT|LIMIT|OFFSET|DISTINCT)\b/g

/** SQL の簡易シンタックスハイライト(キーワードと数値のみ) */
function highlightSql(sql: string): ReactNode[] {
  return sql.split(KEYWORD_PATTERN).map((part, i) =>
    i % 2 === 1 ? (
      <span key={i} className="font-semibold text-[#7ee9ff]">
        {part}
      </span>
    ) : (
      <span key={i}>
        {part.split(/(\d+(?:\.\d+)?)/g).map((s, j) =>
          j % 2 === 1 ? (
            <span key={j} className="text-step-filter">
              {s}
            </span>
          ) : (
            s
          ),
        )}
      </span>
    ),
  )
}

/** 左ペイン: SQL 表示と凡例(将来はエディタ + パース結果表示になる) */
export function SqlPane() {
  return (
    <aside className="flex w-[340px] shrink-0 flex-col border-r border-pane-border bg-pane">
      <h2 className="px-4 pt-3 pb-2 text-[11px] font-semibold tracking-[0.12em] text-fg-dim">
        QUERY (FROM-FIRST)
      </h2>
      <pre className="mx-3 overflow-x-auto rounded-md border border-pane-border bg-canvas p-4 font-mono text-[12.5px] leading-[1.75] whitespace-pre text-fg-muted">
        {highlightSql(mockSql)}
      </pre>
      <h2 className="px-4 pt-4 pb-2 text-[11px] font-semibold tracking-[0.12em] text-fg-dim">
        STEPS
      </h2>
      <div className="grid gap-2 px-4">
        {LEGEND.map((item) => (
          <div
            key={item.label}
            className="flex items-center gap-2.5 text-xs text-fg-muted"
          >
            <span className={`size-2.5 shrink-0 rounded-[3px] ${item.color}`} />
            {item.label}
          </div>
        ))}
      </div>
      <p className="mt-auto mb-4 border-t border-pane-border px-4 pt-3 text-[11.5px] leading-[1.7] text-fg-dim">
        列は SQL に現れた事実のみ表示し、それ以外の列の存在は
        <b className="font-semibold text-fg-muted"> … </b>で示します。
        <span className="text-accent">緑</span> = 最終結果に値が届く列 / 白 =
        条件にのみ使われる列。フローの並び = 論理実行順なので、
        読解だけでなくチューニングの検討にも使えます。
        現在はモックデータで描画しており、Rust パーサー(WASM)との接続は
        フェーズ4で行います。
      </p>
    </aside>
  )
}
