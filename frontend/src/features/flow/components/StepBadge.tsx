import { Tooltip } from '@base-ui-components/react/tooltip'

/** ノード右上の論理実行順バッジ。ホバーで説明をツールチップ表示する(Base UI) */
export function StepBadge({
  no,
  highlighted,
}: {
  no: string
  highlighted: boolean
}) {
  return (
    <Tooltip.Root>
      <Tooltip.Trigger
        render={
          <span
            className={`absolute -top-[12px] -right-[12px] flex size-[24px] items-center justify-center rounded-full border-2 bg-node text-xs font-bold transition-colors duration-300 ${
              highlighted
                ? 'border-accent text-accent-ink'
                : 'border-pane-border text-ink-muted'
            }`}
          >
            {no}
          </span>
        }
      />
      <Tooltip.Portal>
        <Tooltip.Positioner sideOffset={8}>
          <Tooltip.Popup className="rounded-xl border-2 border-pane-border bg-node px-2.5 py-1 text-xs font-medium text-ink-muted shadow-[0_4px_14px_rgba(74,59,68,0.12)]">
            論理実行順 {no}
          </Tooltip.Popup>
        </Tooltip.Positioner>
      </Tooltip.Portal>
    </Tooltip.Root>
  )
}
