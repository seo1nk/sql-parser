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
            className={`absolute -top-[11px] -right-[11px] flex size-[22px] items-center justify-center rounded-full border bg-pane-muted text-xs transition-colors duration-300 ${
              highlighted
                ? 'border-accent text-accent'
                : 'border-pane-border text-fg-muted'
            }`}
          >
            {no}
          </span>
        }
      />
      <Tooltip.Portal>
        <Tooltip.Positioner sideOffset={8}>
          <Tooltip.Popup className="rounded-md border border-pane-border bg-pane-muted px-2.5 py-1 text-xs text-fg-muted shadow-lg">
            論理実行順 {no}
          </Tooltip.Popup>
        </Tooltip.Positioner>
      </Tooltip.Portal>
    </Tooltip.Root>
  )
}
