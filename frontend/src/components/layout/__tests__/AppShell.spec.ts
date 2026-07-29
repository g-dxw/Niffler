import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApp, nextTick, type App } from '@/test/vue'

import AppShell from '../AppShell.vue'

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

function mountAppShell(onCloseMobileSidebar = vi.fn()) {
  const root = document.createElement('div')
  document.body.appendChild(root)

  const app = createApp(AppShell, {
    mobileSidebarOpen: true,
    onCloseMobileSidebar,
  })
  app.mount(root)
  mountedApps.push({ app, root })

  return { root, onCloseMobileSidebar }
}

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
})

describe('AppShell mobile sidebar backdrop', () => {
  it('renders the localized close control and emits the close event', async () => {
    const { root, onCloseMobileSidebar } = mountAppShell()
    const backdrop = root.querySelector<HTMLButtonElement>('button[aria-label="关闭侧边栏"]')

    expect(backdrop).not.toBeNull()

    backdrop?.click()
    await nextTick()

    expect(onCloseMobileSidebar).toHaveBeenCalledOnce()
  })
})
