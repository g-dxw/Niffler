import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, nextTick, ref, type App } from '@/test/vue'
import { createMemoryHistory, createRouter } from 'vue-router'

import PublicHeader from '../PublicHeader.vue'

const authStore = vi.hoisted(() => ({
  isAuthenticated: false,
  canAccessAdmin: false,
}))

vi.mock('@/stores/auth', () => ({
  useAuthStore: () => authStore,
}))

vi.mock('@/composables/useSiteInfo', () => ({
  useSiteInfo: () => ({ siteName: ref('Niffler'), siteSubtitle: ref('AI Gateway') }),
}))

vi.mock('@/components/HeaderLogo.vue', () => ({
  default: defineComponent({ name: 'HeaderLogoStub', setup: () => () => h('span') }),
}))

vi.mock('@/components/common/TopBarActions.vue', () => ({
  default: defineComponent({ name: 'TopBarActionsStub', setup: () => () => h('div') }),
}))

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

async function mountHeader(onLogin = vi.fn()) {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: { template: '<div />' } },
      { path: '/models', component: { template: '<div />' } },
      { path: '/guide', component: { template: '<div />' } },
      { path: '/dashboard', component: { template: '<div />' } },
      { path: '/dashboard/image-studio', component: { template: '<div />' } },
    ],
  })
  await router.push('/')
  await router.isReady()

  const Host = defineComponent({
    setup: () => () => h(PublicHeader, { onLogin }),
  })
  const app = createApp(Host)
  app.use(router)
  app.mount(root)
  mountedApps.push({ app, root })

  return { root, router, onLogin }
}

async function clickImageStudio(root: HTMLElement) {
  const link = root.querySelector<HTMLAnchorElement>('header a[href="/dashboard/image-studio"]')
  expect(link).not.toBeNull()
  const event = new MouseEvent('click', { bubbles: true, cancelable: true })
  link?.dispatchEvent(event)
  for (let index = 0; index < 8; index += 1) {
    await Promise.resolve()
    await nextTick()
  }
  return event
}

beforeEach(() => {
  authStore.isAuthenticated = false
  authStore.canAccessAdmin = false
})

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
  document.body.innerHTML = ''
})

describe('PublicHeader image studio navigation', () => {
  it('opens the login dialog instead of navigating for signed-out users', async () => {
    const { root, router, onLogin } = await mountHeader()

    await clickImageStudio(root)

    expect(onLogin).toHaveBeenCalledOnce()
    expect(router.currentRoute.value.fullPath).toBe('/')
  })
})
