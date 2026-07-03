import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, nextTick, type App } from 'vue'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'

import MainLayout from '../MainLayout.vue'

const adminApiMocks = vi.hoisted(() => ({
  checkUpdate: vi.fn(),
}))

const authStore = vi.hoisted(() => ({
  user: { username: 'admin', role: 'admin' },
  token: 'token',
  canAccessAdmin: true,
  canOperateAdmin: true,
  syncToken: vi.fn(),
  logout: vi.fn(),
}))

const moduleStore = vi.hoisted(() => ({
  loaded: false,
  loading: false,
  modules: {},
  fetchModules: vi.fn(),
}))

vi.mock('@/stores/auth', () => ({
  useAuthStore: () => authStore,
}))

vi.mock('@/stores/modules', () => ({
  useModuleStore: () => moduleStore,
}))

vi.mock('@/api/admin', () => ({
  adminApi: adminApiMocks,
}))

vi.mock('@/api/announcements', () => ({
  announcementApi: {
    getRequiredUnreadAnnouncements: vi.fn().mockResolvedValue({ items: [] }),
    markAsRead: vi.fn().mockResolvedValue({}),
  },
}))

vi.mock('@/api/billing', () => ({
  billingApi: {
    listEntitlements: vi.fn().mockResolvedValue({ items: [] }),
  },
}))

vi.mock('@/api/wallet', () => ({
  walletApi: {
    getBalance: vi.fn().mockResolvedValue({ wallet_balance: 0, package_balance: 0 }),
  },
}))

vi.mock('@/composables/useDarkMode', () => ({
  useDarkMode: () => ({ themeMode: 'light', toggleDarkMode: vi.fn() }),
}))

vi.mock('@/composables/useSiteInfo', () => ({
  useSiteInfo: () => ({ siteName: 'Niffler', siteSubtitle: 'Test' }),
}))

vi.mock('@/config/demo', () => ({
  isDemoMode: () => false,
}))

vi.mock('@/components/layout/AppShell.vue', () => ({
  default: defineComponent({
    name: 'AppShellStub',
    setup(_, { slots }) {
      return () => h('div', [slots.notice?.(), slots.sidebar?.(), slots.header?.(), slots.default?.()])
    },
  }),
}))

vi.mock('@/components/layout/SidebarNav.vue', () => ({
  default: defineComponent({ name: 'SidebarNavStub', setup: () => () => h('nav') }),
}))

vi.mock('@/components/HeaderLogo.vue', () => ({
  default: defineComponent({ name: 'HeaderLogoStub', setup: () => () => h('span') }),
}))

vi.mock('@/components/common/UpdateDialog.vue', () => ({
  default: defineComponent({ name: 'UpdateDialogStub', setup: () => () => h('div') }),
}))

vi.mock('@/components/common/VersionButton.vue', () => ({
  default: defineComponent({ name: 'VersionButtonStub', setup: () => () => h('button') }),
}))

vi.mock('@/components/ui/button.vue', () => ({
  default: defineComponent({ name: 'ButtonStub', setup: (_, { slots }) => () => h('button', slots.default?.()) }),
}))

vi.mock('@/components/ui', () => ({
  Dialog: defineComponent({ name: 'DialogStub', setup: (_, { slots }) => () => h('div', slots.default?.()) }),
}))

vi.mock('@/components/icons/GithubIcon.vue', () => ({
  default: defineComponent({ name: 'GithubIconStub', setup: () => () => h('span') }),
}))

vi.mock('@/config/builtin-tools', () => ({
  BUILTIN_TOOL_BREADCRUMBS: {},
}))

vi.mock('@/utils/adminNavigationPrefetch', () => ({
  prefetchAdminNavigationTarget: vi.fn(),
}))

vi.mock('lucide-vue-next', () => {
  const Icon = defineComponent({ name: 'IconStub', setup: () => () => h('span') })
  return {
    Activity: Icon,
    AlertTriangle: Icon,
    BarChart3: Icon,
    Box: Icon,
    ChevronRight: Icon,
    Cog: Icon,
    CreditCard: Icon,
    Database: Icon,
    FileUp: Icon,
    FolderTree: Icon,
    Gift: Icon,
    Gauge: Icon,
    Home: Icon,
    Key: Icon,
    KeyRound: Icon,
    Layers: Icon,
    LogOut: Icon,
    Megaphone: Icon,
    Menu: Icon,
    Moon: Icon,
    Package: Icon,
    Puzzle: Icon,
    Server: Icon,
    Settings: Icon,
    Shield: Icon,
    SlidersHorizontal: Icon,
    SunMedium: Icon,
    SunMoon: Icon,
    Users: Icon,
    Wallet: Icon,
    X: Icon,
    Zap: Icon,
  }
})

const mountedApps: Array<{ app: App, root: HTMLElement, router: Router }> = []

async function mountMainLayout() {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: { template: '<div />' } },
      { path: '/dashboard/wallet', component: { template: '<div />' } },
      { path: '/dashboard/settings', component: { template: '<div />' } },
    ],
  })
  await router.push('/')
  await router.isReady()
  const app = createApp(MainLayout)
  app.use(router)
  app.mount(root)
  mountedApps.push({ app, root, router })
}

async function settle() {
  for (let index = 0; index < 8; index += 1) {
    await Promise.resolve()
    await nextTick()
  }
}

beforeEach(() => {
  vi.useFakeTimers()
  adminApiMocks.checkUpdate.mockReset()
  adminApiMocks.checkUpdate.mockResolvedValue({
    current_version: 'v1.0.0',
    latest_version: 'v1.0.0',
    has_update: false,
    release_url: null,
    release_notes: null,
    published_at: null,
    error: null,
  })
  authStore.syncToken.mockReset()
  moduleStore.fetchModules.mockReset()
  sessionStorage.clear()
  localStorage.clear()
})

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
  document.body.innerHTML = ''
  vi.useRealTimers()
})

describe('MainLayout version check', () => {
  it('does not call check-update automatically when the admin console mounts', async () => {
    await mountMainLayout()
    await settle()
    await vi.advanceTimersByTimeAsync(3000)
    await settle()

    expect(adminApiMocks.checkUpdate).not.toHaveBeenCalled()
  })
})
