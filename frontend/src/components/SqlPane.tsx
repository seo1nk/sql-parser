const LEGEND = [
  {
    swatch: 'bg-scan-soft border-scan-ink/40',
    label: 'SCAN / CTE — 集合の形成 (FROM / WITH)',
  },
  {
    swatch: 'bg-join-soft border-join-ink/40',
    label: 'JOINED — 合流して 1 つの集合になる',
  },
  {
    swatch: 'bg-filter-soft border-filter-ink/40',
    label: 'FILTER — 絞り込み (WHERE / HAVING)',
  },
  {
    swatch: 'bg-group-soft border-group-ink/40',
    label: 'GROUP — グループ化 (GROUP BY)',
  },
  {
    swatch: 'bg-project-soft border-project-ink/40',
    label: 'PROJECT — 抽出 (SELECT)',
  },
]

type Props = {
  sql: string
  onChange: (sql: string) => void
  /** パース失敗時のメッセージ(直前の正常なグラフは表示されたまま) */
  error: string | null
}

/** 左ペイン: SQL エディタと凡例 */
export function SqlPane({ sql, onChange, error }: Props) {
  return (
    <aside className="flex w-[340px] shrink-0 flex-col border-r-2 border-pane-border bg-pane">
      <h2 className="px-4 pt-3 pb-2 text-[11px] font-bold tracking-[0.12em] text-ink-dim">
        QUERY (FROM-FIRST もOK)
      </h2>
      <textarea
        value={sql}
        onChange={(e) => onChange(e.target.value)}
        spellCheck={false}
        aria-label="SQL エディタ"
        className="mx-3 h-72 resize-none rounded-xl border-2 border-pane-border bg-pane-muted p-4 font-mono text-[12.5px] leading-[1.75] text-ink transition-colors duration-200 focus:border-accent focus:outline-none"
      />
      {error && (
        <p className="mx-3 mt-2 rounded-xl border-2 border-[#f5b8c4] bg-[#ffe9ee] px-3 py-2 text-xs font-semibold text-[#c2405a]">
          {error}
        </p>
      )}
      <h2 className="px-4 pt-4 pb-2 text-[11px] font-bold tracking-[0.12em] text-ink-dim">
        STEPS
      </h2>
      <div className="grid gap-2 px-4">
        {LEGEND.map((item) => (
          <div
            key={item.label}
            className="flex items-center gap-2.5 text-xs font-medium text-ink-muted"
          >
            <span
              className={`size-3 shrink-0 rounded-md border ${item.swatch}`}
            />
            {item.label}
          </div>
        ))}
      </div>
      <p className="mt-auto mb-4 border-t-2 border-pane-border px-4 pt-3 text-[11.5px] leading-[1.8] text-ink-dim">
        列は SQL に現れた事実のみ表示し、それ以外の列の存在は
        <b className="font-bold text-ink-muted"> … </b>で示します。
        <span className="font-bold text-accent-ink">ピンク</span> =
        最終結果に値が届く列 / グレー = 条件にのみ使われる列。 フローの並び =
        論理実行順なので、読解だけでなくチューニングの検討にも使えます。
        パースは Rust 製パーサー(WASM)がブラウザ内で実行しています。
      </p>
    </aside>
  )
}
