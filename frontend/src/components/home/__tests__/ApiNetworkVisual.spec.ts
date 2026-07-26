import { afterEach, describe, expect, it } from 'vitest'
import { createApp, nextTick, type App } from '@/test/vue'

import ApiNetworkVisual from '../ApiNetworkVisual.vue'

const upstreamNodes = [
  { id: 'gpt', label: 'GPT', icon: '/openai.svg' },
  { id: 'claude', label: 'Claude', icon: '/claude-color.svg' },
  { id: 'gemini', label: 'Gemini', icon: '/gemini-color.svg' },
  { id: 'deepseek', label: 'DeepSeek', icon: '/deepseek.svg' },
  { id: 'qwen', label: 'Qwen', icon: '/qwen.svg' },
  { id: 'more', label: '更多模型', glyph: '···' },
]

const downstreamNodes = [
  { id: 'codex', label: 'Codex', icon: '/openai.svg' },
  { id: 'claude-code', label: 'Claude Code', icon: '/claude-color.svg' },
  { id: 'canvas', label: '无限画布', glyph: '∞' },
  { id: 'image-studio', label: '生图工作台', glyph: '✦' },
]

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

function mountVisual() {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp(ApiNetworkVisual, {
    upstreamNodes,
    downstreamNodes,
    coreSubtitle: '统一 AI API',
    accessibleTitle: 'Niffler 模型接入示意',
    accessibleDescription: '模型经过 Niffler 接入工具。',
  })
  app.mount(root)
  mountedApps.push({ app, root })
  return root
}

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
})

describe('ApiNetworkVisual', () => {
  it('renders an accessible model-to-tool flow', () => {
    const root = mountVisual()

    expect(root.querySelector('title')?.textContent).toBe('Niffler 模型接入示意')
    expect(root.querySelector('desc')?.textContent).toBe('模型经过 Niffler 接入工具。')
    expect(root.querySelectorAll('[data-flow-side="upstream"][role="img"]')).toHaveLength(6)
    expect(root.querySelectorAll('[data-flow-side="downstream"][role="img"]')).toHaveLength(4)
    expect(root.textContent).toContain('Niffler')
    expect(root.textContent).toContain('统一 AI API')
  })

  it('highlights the full related path when a node receives focus', async () => {
    const root = mountVisual()
    const gptNode = root.querySelector<SVGGElement>('[data-flow-side="upstream"][data-node-id="gpt"][role="img"]')

    gptNode?.dispatchEvent(new FocusEvent('focus'))
    await nextTick()

    expect(root.querySelector('[data-flow-side="upstream"][data-node-id="claude"] .node-visual')?.classList).toContain('node-visual--dimmed')
    expect(root.querySelector('[data-flow-side="upstream"][data-node-id="gpt"].connection')?.classList).toContain('connection--related')
    expect(root.querySelector('[data-flow-side="downstream"][data-node-id="codex"].connection')?.classList).toContain('connection--related')

    gptNode?.dispatchEvent(new FocusEvent('blur'))
    await nextTick()

    expect(root.querySelector('.node-visual--dimmed')).toBeNull()
  })
})
