import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { nextTick } from 'vue'
import { useResizableTableColumns } from '../useResizableTableColumns'

type TestColumnKey = 'name' | 'status'

function createResizeTarget(width: number): HTMLButtonElement {
  const table = document.createElement('table')
  const row = document.createElement('tr')
  const header = document.createElement('th')
  const handle = document.createElement('button')

  header.getBoundingClientRect = () => ({
    width,
    height: 20,
    x: 0,
    y: 0,
    top: 0,
    right: width,
    bottom: 20,
    left: 0,
    toJSON: () => ({}),
  })

  header.appendChild(handle)
  row.appendChild(header)
  table.appendChild(row)
  document.body.appendChild(table)

  return handle
}

function buildPointerEvent(type: string, target: HTMLElement, clientX: number): PointerEvent {
  const event = new MouseEvent(type, { clientX, bubbles: true }) as PointerEvent
  Object.defineProperty(event, 'currentTarget', {
    configurable: true,
    value: target,
  })
  Object.defineProperty(event, 'target', {
    configurable: true,
    value: target,
  })
  return event
}

beforeEach(() => {
  window.localStorage.clear()
})

afterEach(() => {
  document.body.innerHTML = ''
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  window.localStorage.clear()
})

describe('useResizableTableColumns', () => {
  it('uses configured widths before the user resizes a column', () => {
    const tableColumns = useResizableTableColumns<TestColumnKey>({
      storageKey: 'test-table-columns',
      columns: [
        { key: 'name', width: '240px' },
        { key: 'status', width: '12%' },
      ],
    })

    expect(tableColumns.getColumnWidth('name')).toBe('240px')
    expect(tableColumns.getColumnStyle('status')).toEqual({ width: '12%' })
  })

  it('stores resized widths and respects the minimum width', async () => {
    const tableColumns = useResizableTableColumns<TestColumnKey>({
      storageKey: 'test-table-columns',
      defaultMinWidth: 80,
      columns: [
        { key: 'name', width: '240px' },
        { key: 'status', width: '12%', minWidth: 96 },
      ],
    })
    const handle = createResizeTarget(160)

    tableColumns.startResize({
      key: 'name',
      event: buildPointerEvent('pointerdown', handle, 100),
    })
    window.dispatchEvent(new MouseEvent('pointermove', { clientX: 180 }) as PointerEvent)
    window.dispatchEvent(new MouseEvent('pointerup') as PointerEvent)
    await nextTick()

    expect(tableColumns.getColumnWidth('name')).toBe('240px')

    tableColumns.startResize({
      key: 'status',
      event: buildPointerEvent('pointerdown', handle, 100),
    })
    window.dispatchEvent(new MouseEvent('pointermove', { clientX: 0 }) as PointerEvent)
    window.dispatchEvent(new MouseEvent('pointerup') as PointerEvent)
    await nextTick()

    expect(tableColumns.getColumnWidth('status')).toBe('96px')
  })

  it('can reset stored widths', async () => {
    const tableColumns = useResizableTableColumns<TestColumnKey>({
      storageKey: 'test-table-columns',
      columns: [
        { key: 'name', width: '240px' },
        { key: 'status', width: '12%' },
      ],
    })
    const handle = createResizeTarget(160)

    tableColumns.startResize({
      key: 'name',
      event: buildPointerEvent('pointerdown', handle, 100),
    })
    window.dispatchEvent(new MouseEvent('pointermove', { clientX: 140 }) as PointerEvent)
    window.dispatchEvent(new MouseEvent('pointerup') as PointerEvent)
    await nextTick()

    expect(tableColumns.getColumnWidth('name')).toBe('200px')

    tableColumns.resetColumnWidths()
    await nextTick()

    expect(tableColumns.getColumnWidth('name')).toBe('240px')
  })
})
