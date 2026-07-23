import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, type App } from '@/test/vue'
import TimeRangePicker from '../TimeRangePicker.vue'

vi.mock('@/components/ui', async () => {
  const { defineComponent, h } = await import('vue')

  const passthrough = (name: string) => defineComponent({
    name,
    props: { modelValue: [String, Number, Boolean] },
    emits: ['update:modelValue'],
    setup(_, { slots }) {
      return () => h('div', slots.default?.())
    },
  })

  return {
    Select: passthrough('SelectStub'),
    SelectContent: passthrough('SelectContentStub'),
    SelectItem: passthrough('SelectItemStub'),
    SelectTrigger: passthrough('SelectTriggerStub'),
    SelectValue: passthrough('SelectValueStub'),
    Input: defineComponent({
      name: 'InputStub',
      props: { modelValue: String },
      emits: ['update:modelValue'],
      setup(props, { attrs, emit }) {
        return () => h('input', {
          ...attrs,
          value: props.modelValue ?? '',
          onInput: (event: Event) => emit('update:modelValue', (event.target as HTMLInputElement).value),
        })
      },
    }),
  }
})

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
})

function mountPicker(onUpdate: (value: unknown) => void) {
  const root = document.createElement('div')
  document.body.appendChild(root)

  const app = createApp(TimeRangePicker, {
    modelValue: {
      start_time: '2026-06-12T10:00',
      end_time: '2026-06-12T11:00',
      tz_offset_minutes: 480,
    },
    showTime: true,
    showGranularity: false,
    'onUpdate:modelValue': onUpdate,
  })
  app.mount(root)
  mountedApps.push({ app, root })
  return root
}

describe('TimeRangePicker', () => {
  it('emits concrete custom times when time mode is enabled', async () => {
    const updates: unknown[] = []
    const root = mountPicker((value) => updates.push(value))
    const dateInputs = root.querySelectorAll<HTMLInputElement>('input[type="date"]')
    const timeInputs = root.querySelectorAll<HTMLInputElement>('input[type="time"]')

    expect(root.querySelector('input[type="datetime-local"]')).toBeNull()
    expect(dateInputs).toHaveLength(2)
    expect(timeInputs).toHaveLength(2)
    expect(dateInputs[0].getAttribute('aria-label')).toBe('开始日期')
    expect(timeInputs[0].getAttribute('aria-label')).toBe('开始时间')
    dateInputs[1].value = '2026-06-12'
    dateInputs[1].dispatchEvent(new Event('input'))
    timeInputs[1].value = '12:30'
    timeInputs[1].dispatchEvent(new Event('input'))
    await Promise.resolve()

    expect(updates.at(-1)).toMatchObject({
      start_date: '2026-06-12',
      end_date: '2026-06-12',
      start_time: '2026-06-12T10:00',
      end_time: '2026-06-12T12:30',
    })
  })
})
