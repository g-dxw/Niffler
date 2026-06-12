import { computed, getCurrentInstance, onBeforeUnmount, ref, toValue, type CSSProperties, type MaybeRefOrGetter } from 'vue'
import { useLocalStorage } from '@vueuse/core'

export interface ResizableTableColumn<Key extends string> {
  key: Key
  width: string
  minWidth?: number
}

export interface TableColumnResizePayload {
  key: string
  event: PointerEvent
}

export interface UseResizableTableColumnsOptions<Key extends string> {
  storageKey: string
  columns: MaybeRefOrGetter<readonly ResizableTableColumn<Key>[]>
  defaultMinWidth?: number
}

const browserWindow = typeof window !== 'undefined' ? window : undefined

function isFiniteWidth(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value > 0
}

export function useResizableTableColumns<Key extends string>(
  options: UseResizableTableColumnsOptions<Key>,
) {
  const storedColumnWidths = useLocalStorage<Partial<Record<Key, number>>>(
    options.storageKey,
    {},
    { window: browserWindow },
  )
  const liveColumnWidths = ref<Partial<Record<Key, number>>>({ ...storedColumnWidths.value })
  let resizeCleanup: (() => void) | null = null

  function getColumnConfig(key: Key): ResizableTableColumn<Key> | undefined {
    return toValue(options.columns).find(column => column.key === key)
  }

  function getColumnWidth(key: Key): string {
    const storedWidth = liveColumnWidths.value[key]
    if (isFiniteWidth(storedWidth)) {
      return `${Math.round(storedWidth)}px`
    }
    return getColumnConfig(key)?.width ?? 'auto'
  }

  const columnWidths = computed(() => {
    return toValue(options.columns).reduce((result, column) => {
      result[column.key] = getColumnWidth(column.key)
      return result
    }, {} as Record<Key, string>)
  })

  function getColumnStyle(key: Key): CSSProperties {
    return {
      width: getColumnWidth(key),
    }
  }

  function stopResize() {
    resizeCleanup?.()
  }

  function resetColumnWidths() {
    liveColumnWidths.value = {}
    storedColumnWidths.value = {}
  }

  function startResize(payload: TableColumnResizePayload) {
    const key = payload.key as Key
    const config = getColumnConfig(key)
    if (!config) return

    const eventTarget = payload.event.currentTarget instanceof HTMLElement
      ? payload.event.currentTarget
      : payload.event.target instanceof HTMLElement
        ? payload.event.target
        : null
    const header = eventTarget?.closest('th') as HTMLElement | null
    if (!header) return

    stopResize()

    const startX = payload.event.clientX
    const startWidth = header.getBoundingClientRect().width
    const minWidth = config.minWidth ?? options.defaultMinWidth ?? 64
    const previousCursor = document.body.style.cursor
    const previousUserSelect = document.body.style.userSelect

    const handlePointerMove = (event: PointerEvent) => {
      const nextWidth = Math.max(minWidth, Math.round(startWidth + event.clientX - startX))
      liveColumnWidths.value = {
        ...liveColumnWidths.value,
        [key]: nextWidth,
      }
    }

    const cleanup = () => {
      window.removeEventListener('pointermove', handlePointerMove)
      window.removeEventListener('pointerup', cleanup)
      window.removeEventListener('pointercancel', cleanup)
      document.body.style.cursor = previousCursor
      document.body.style.userSelect = previousUserSelect
      storedColumnWidths.value = { ...liveColumnWidths.value }
      resizeCleanup = null
    }

    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
    window.addEventListener('pointermove', handlePointerMove)
    window.addEventListener('pointerup', cleanup)
    window.addEventListener('pointercancel', cleanup)
    resizeCleanup = cleanup
  }

  if (getCurrentInstance()) {
    onBeforeUnmount(stopResize)
  }

  return {
    columnWidths,
    getColumnWidth,
    getColumnStyle,
    resetColumnWidths,
    startResize,
    stopResize,
  }
}
