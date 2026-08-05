/* eslint-disable vue/one-component-per-file, vue/require-default-prop */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, nextTick, type App } from '@/test/vue'
import OAuthAccountDialog from '@/features/providers/components/OAuthAccountDialog.vue'

const endpointMocks = vi.hoisted(() => ({
  startProviderLevelOAuth: vi.fn(),
  completeProviderLevelOAuth: vi.fn(),
  importProviderRefreshToken: vi.fn(),
  startBatchImportOAuthTask: vi.fn(),
  getBatchImportOAuthTaskStatus: vi.fn(),
  startDeviceAuthorize: vi.fn(),
  pollDeviceAuthorize: vi.fn(),
  getAwsRegions: vi.fn(),
}))

vi.mock('@/api/endpoints', () => endpointMocks)

vi.mock('@/components/ui', async () => {
  const { defineComponent, h } = await import('vue')

  const passthrough = (name: string, tag = 'div') => defineComponent({
    name,
    setup(_, { slots }) {
      return () => h(tag, slots.default?.())
    },
  })

  const Dialog = defineComponent({
    name: 'DialogStub',
    props: {
      modelValue: Boolean,
    },
    setup(props, { slots }) {
      return () => props.modelValue
        ? h('section', [slots.headerActions?.(), slots.default?.(), slots.footer?.()])
        : null
    },
  })

  const Button = defineComponent({
    name: 'ButtonStub',
    inheritAttrs: false,
    props: {
      disabled: Boolean,
      variant: String,
      size: String,
    },
    setup(props, { attrs, slots }) {
      return () => h('button', {
        ...attrs,
        disabled: props.disabled,
        type: attrs.type ?? 'button',
      }, slots.default?.())
    },
  })

  const Textarea = defineComponent({
    name: 'TextareaStub',
    inheritAttrs: false,
    props: {
      modelValue: {
        type: String,
        default: '',
      },
    },
    emits: ['update:modelValue'],
    setup(props, { attrs, emit }) {
      return () => h('textarea', {
        ...attrs,
        value: props.modelValue,
        onInput: (event: Event) => emit('update:modelValue', (event.target as HTMLTextAreaElement).value),
      })
    },
  })

  return {
    Dialog,
    Button,
    Textarea,
    Popover: passthrough('PopoverStub'),
    PopoverTrigger: passthrough('PopoverTriggerStub'),
    PopoverContent: passthrough('PopoverContentStub'),
  }
})

vi.mock('radix-vue', async () => {
  const { defineComponent, h } = await import('vue')
  const passthrough = (name: string) => defineComponent({
    name,
    setup(_, { slots }) {
      return () => h('div', slots.default?.())
    },
  })

  return {
    ComboboxAnchor: passthrough('ComboboxAnchorStub'),
    ComboboxContent: passthrough('ComboboxContentStub'),
    ComboboxEmpty: passthrough('ComboboxEmptyStub'),
    ComboboxInput: passthrough('ComboboxInputStub'),
    ComboboxItem: passthrough('ComboboxItemStub'),
    ComboboxRoot: passthrough('ComboboxRootStub'),
    ComboboxTrigger: passthrough('ComboboxTriggerStub'),
    ComboboxViewport: passthrough('ComboboxViewportStub'),
  }
})

vi.mock('@/components/common/JsonImportInput.vue', async () => {
  const { defineComponent, h } = await import('vue')

  return {
    default: defineComponent({
      name: 'JsonImportInputStub',
      props: {
        modelValue: {
          type: String,
          default: '',
        },
        dropTitle: {
          type: String,
          default: '',
        },
        dropHint: {
          type: String,
          default: '',
        },
        manualPlaceholder: {
          type: String,
          default: '',
        },
        manualDescription: {
          type: String,
          default: '',
        },
        pasteToggleText: {
          type: String,
          default: '',
        },
        fileToggleText: {
          type: String,
          default: '',
        },
      },
      emits: ['update:modelValue'],
      setup(props, { emit }) {
        return () => h('div', [
          h('p', { 'data-testid': 'drop-title' }, props.dropTitle),
          h('p', { 'data-testid': 'drop-hint' }, props.dropHint),
          h('p', { 'data-testid': 'manual-description' }, props.manualDescription),
          h('p', props.pasteToggleText),
          h('p', props.fileToggleText),
          h('textarea', {
            placeholder: props.manualPlaceholder,
            value: props.modelValue,
            onInput: (event: Event) => emit('update:modelValue', (event.target as HTMLTextAreaElement).value),
          }),
        ])
      },
    }),
  }
})

vi.mock('@/components/ui/Label.vue', () => ({}))
vi.mock('./ProxyNodeSelect.vue', () => ({}))
vi.mock('@/features/providers/components/ProxyNodeSelect.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return {
    default: defineComponent({
      name: 'ProxyNodeSelectStub',
      setup() {
        return () => h('div')
      },
    }),
  }
})

vi.mock('@/stores/proxy-nodes', () => ({
  useProxyNodesStore: () => ({
    nodes: [],
    onlineNodes: [],
    loading: false,
    ensureLoaded: vi.fn(),
  }),
}))

vi.mock('@/composables/useToast', () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
  }),
}))

vi.mock('@/composables/useClipboard', () => ({
  useClipboard: () => ({
    copyToClipboard: vi.fn(),
  }),
}))

vi.mock('@/composables/useTotp', () => ({
  useTotp: () => ({
    code: { value: '' },
    remaining: { value: 0 },
    start: vi.fn(),
    stop: vi.fn(),
  }),
}))

vi.mock('lucide-vue-next', async () => {
  const { defineComponent, h } = await import('vue')
  const Icon = defineComponent({
    name: 'IconStub',
    setup() {
      return () => h('span')
    },
  })

  return {
    UserPlus: Icon,
    Copy: Icon,
    ExternalLink: Icon,
    Globe: Icon,
    AlertCircle: Icon,
    ShieldCheck: Icon,
    ChevronsUpDown: Icon,
    Check: Icon,
  }
})

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

function mountDialog(providerType = 'grok') {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp(OAuthAccountDialog, {
    open: true,
    providerId: 'provider-1',
    providerType,
  })
  app.mount(root)
  mountedApps.push({ app, root })
  return root
}

async function settle() {
  await nextTick()
  await Promise.resolve()
}

function getButton(root: HTMLElement, text: string) {
  return Array.from(root.querySelectorAll('button'))
    .find(button => button.textContent?.includes(text))
}

function getButtonExact(root: HTMLElement, text: string) {
  return Array.from(root.querySelectorAll('button'))
    .find(button => button.textContent?.trim() === text)
}

function getImportTextarea(root: HTMLElement) {
  const textarea = root.querySelector('textarea')
  if (!(textarea instanceof HTMLTextAreaElement)) {
    throw new Error('Expected import textarea to exist')
  }
  return textarea
}

describe('OAuthAccountDialog imports', () => {
  beforeEach(() => {
    endpointMocks.startProviderLevelOAuth.mockReset()
    endpointMocks.completeProviderLevelOAuth.mockReset()
    endpointMocks.importProviderRefreshToken.mockReset()
    endpointMocks.startBatchImportOAuthTask.mockReset()
    endpointMocks.getBatchImportOAuthTaskStatus.mockReset()
    endpointMocks.startDeviceAuthorize.mockReset()
    endpointMocks.pollDeviceAuthorize.mockReset()
    endpointMocks.getAwsRegions.mockReset()

    endpointMocks.startProviderLevelOAuth.mockResolvedValue({
      authorization_url: 'https://example.com/oauth',
      redirect_uri: 'http://localhost/callback',
      provider_type: 'codex',
      instructions: '',
    })
    endpointMocks.importProviderRefreshToken.mockResolvedValue({
      provider_type: 'grok',
      has_refresh_token: false,
      email: 'grok@example.com',
      replaced: false,
    })
    endpointMocks.startBatchImportOAuthTask.mockResolvedValue({
      task_id: 'task-1',
      status: 'submitted',
      total: 2,
      processed: 0,
      success: 0,
      failed: 0,
      progress_percent: 0,
    })
  })

  afterEach(() => {
    for (const { app, root } of mountedApps.splice(0)) {
      app.unmount()
      root.remove()
    }
  })

  it('opens Grok in import mode without starting unsupported OAuth', async () => {
    const root = mountDialog('grok')
    await settle()

    expect(endpointMocks.startProviderLevelOAuth).not.toHaveBeenCalled()
    expect(root.textContent).not.toContain('获取授权')
    expect(root.querySelector('textarea')?.getAttribute('placeholder')).toContain('Grok sso/session token')
    expect(root.textContent).toContain('plan_type / pool_tier')
    expect(getButton(root, '导入账号')).toBeTruthy()
  })

  it('submits the xAI authorization code with the active Grok OAuth state', async () => {
    endpointMocks.startProviderLevelOAuth.mockResolvedValue({
      authorization_url: 'https://auth.x.ai/oauth2/authorize?client_id=client-1&state=state-123',
      redirect_uri: 'http://127.0.0.1:56121/callback',
      provider_type: 'grok_oauth',
      instructions: '',
    })
    endpointMocks.completeProviderLevelOAuth.mockResolvedValue({
      provider_type: 'grok_oauth',
      has_refresh_token: true,
    })

    const root = mountDialog('grok_oauth')
    await settle()
    getButtonExact(root, '导入授权')?.click()
    await settle()
    getButtonExact(root, '获取授权')?.click()
    await settle()

    const textarea = getImportTextarea(root)
    textarea.value = 'xai-code_123'
    textarea.dispatchEvent(new Event('input'))
    await settle()
    getButtonExact(root, '验证')?.click()
    await settle()

    expect(endpointMocks.completeProviderLevelOAuth).toHaveBeenCalledTimes(1)
    const callbackUrl = new URL(endpointMocks.completeProviderLevelOAuth.mock.calls[0][1].callback_url)
    expect(callbackUrl.origin).toBe('http://127.0.0.1:56121')
    expect(callbackUrl.pathname).toBe('/callback')
    expect(callbackUrl.searchParams.get('code')).toBe('xai-code_123')
    expect(callbackUrl.searchParams.get('state')).toBe('state-123')
  })

  it('keeps a complete Grok OAuth callback URL unchanged', async () => {
    endpointMocks.startProviderLevelOAuth.mockResolvedValue({
      authorization_url: 'https://auth.x.ai/oauth2/authorize?state=state-123',
      redirect_uri: 'http://127.0.0.1:56121/callback',
      provider_type: 'grok_oauth',
      instructions: '',
    })
    endpointMocks.completeProviderLevelOAuth.mockResolvedValue({
      provider_type: 'grok_oauth',
      has_refresh_token: true,
    })

    const root = mountDialog('grok_oauth')
    await settle()
    getButtonExact(root, '导入授权')?.click()
    await settle()
    getButtonExact(root, '获取授权')?.click()
    await settle()

    const callbackUrl = 'http://127.0.0.1:56121/callback?code=xai-code_456&state=state-456'
    const textarea = getImportTextarea(root)
    textarea.value = callbackUrl
    textarea.dispatchEvent(new Event('input'))
    await settle()
    getButtonExact(root, '验证')?.click()
    await settle()

    expect(endpointMocks.completeProviderLevelOAuth).toHaveBeenCalledWith('provider-1', {
      callback_url: callbackUrl,
      proxy_node_id: undefined,
    })
  })

  it('maps a single Grok JSON token into account metadata import payload', async () => {
    const root = mountDialog('grok')
    await settle()

    const textarea = getImportTextarea(root)
    textarea.value = JSON.stringify({
      token: 'sso-1',
      planType: 'super',
      tier: 'heavy',
      email: 'grok@example.com',
      accountName: 'Grok Heavy',
    })
    textarea.dispatchEvent(new Event('input'))
    await settle()

    getButton(root, '导入账号')?.click()
    await settle()

    expect(endpointMocks.importProviderRefreshToken).toHaveBeenCalledWith('provider-1', {
      access_token: 'sso-1',
      account_name: 'Grok Heavy',
      email: 'grok@example.com',
      plan_type: 'super',
      pool_tier: 'heavy',
      sso_rw_token: undefined,
      cf_cookies: undefined,
      cf_clearance: undefined,
      user_agent: undefined,
      browser_profile: undefined,
      proxy_node_id: undefined,
      refresh_token: undefined,
      expires_at: undefined,
      name: undefined,
      account_id: undefined,
      account_user_id: undefined,
      user_id: undefined,
    })
  })

  it('keeps Grok multiline token import on the batch task path', async () => {
    const root = mountDialog('grok')
    await settle()

    const textarea = getImportTextarea(root)
    textarea.value = 'sso-1\nsso-2'
    textarea.dispatchEvent(new Event('input'))
    await settle()

    getButton(root, '导入账号')?.click()
    await settle()

    expect(endpointMocks.startBatchImportOAuthTask).toHaveBeenCalledWith(
      'provider-1',
      'sso-1\nsso-2',
      undefined,
    )
    expect(endpointMocks.importProviderRefreshToken).not.toHaveBeenCalled()
  })

  it('routes a sub2api export object to the batch import task', async () => {
    const root = mountDialog('codex')
    await settle()

    getButton(root, '导入授权')?.click()
    await settle()

    const raw = JSON.stringify({
      exported_at: '2026-07-14T10:36:05Z',
      proxies: [],
      accounts: [{
        name: 'ignored-custom-name',
        platform: 'openai',
        type: 'oauth',
        credentials: {
          access_token: 'pat-1',
          email: 'first@example.com',
          chatgpt_account_id: 'workspace-1',
          chatgpt_user_id: 'user-1',
          plan_type: 'team',
        },
      }],
    })
    const textarea = getImportTextarea(root)
    textarea.value = raw
    textarea.dispatchEvent(new Event('input'))
    await settle()

    getButtonExact(root, '导入')?.click()
    await settle()

    expect(endpointMocks.startBatchImportOAuthTask).toHaveBeenCalledWith(
      'provider-1',
      raw,
      undefined,
    )
    expect(endpointMocks.importProviderRefreshToken).not.toHaveBeenCalled()
  })

  it('keeps a single account object with exported_at on the single import path', async () => {
    const root = mountDialog('codex')
    await settle()

    getButton(root, '导入授权')?.click()
    await settle()

    const raw = JSON.stringify({
      access_token: 'legacy-token',
      email: 'legacy@example.com',
      exported_at: '2026-07-14T10:36:05Z',
    })
    const textarea = getImportTextarea(root)
    textarea.value = raw
    textarea.dispatchEvent(new Event('input'))
    await settle()

    getButtonExact(root, '导入')?.click()
    await settle()

    expect(endpointMocks.importProviderRefreshToken).toHaveBeenCalledWith(
      'provider-1',
      expect.objectContaining({
        access_token: 'legacy-token',
        email: 'legacy@example.com',
      }),
    )
    expect(endpointMocks.startBatchImportOAuthTask).not.toHaveBeenCalled()
  })

  it('maps a nested ChatGPT auth JSON object into the single import payload', async () => {
    const root = mountDialog('codex')
    await settle()

    getButton(root, '导入授权')?.click()
    await settle()

    const raw = JSON.stringify({
      auth_mode: 'chatgpt',
      openai_api_key: null,
      tokens: {
        id_token: 'nested-id-token',
        access_token: 'nested-access-token',
        refresh_token: 'nested-refresh-token',
        account_id: 'nested-account-id',
      },
      last_refresh: '2026-08-05T12:34:56Z',
      _meta: {
        plan_type: 'pro',
        email: 'nested@example.com',
        imported_at: '2026-08-05T12:35:00Z',
      },
    })
    const textarea = getImportTextarea(root)
    textarea.value = raw
    textarea.dispatchEvent(new Event('input'))
    await settle()

    getButtonExact(root, '导入')?.click()
    await settle()

    expect(endpointMocks.importProviderRefreshToken).toHaveBeenCalledWith(
      'provider-1',
      expect.objectContaining({
        access_token: 'nested-access-token',
        refresh_token: 'nested-refresh-token',
        id_token: 'nested-id-token',
        account_id: 'nested-account-id',
        email: 'nested@example.com',
        plan_type: 'pro',
        last_refresh: '2026-08-05T12:34:56Z',
      }),
    )
    expect(endpointMocks.startBatchImportOAuthTask).not.toHaveBeenCalled()
  })

  it('keeps top-level OAuth fields ahead of nested ChatGPT auth fields', async () => {
    const root = mountDialog('codex')
    await settle()

    getButton(root, '导入授权')?.click()
    await settle()

    const textarea = getImportTextarea(root)
    textarea.value = JSON.stringify({
      access_token: 'top-level-access-token',
      refresh_token: 'top-level-refresh-token',
      account_id: 'top-level-account-id',
      email: 'top-level@example.com',
      plan_type: 'team',
      tokens: {
        access_token: 'nested-access-token',
        refresh_token: 'nested-refresh-token',
        account_id: 'nested-account-id',
      },
      _meta: {
        email: 'nested@example.com',
        plan_type: 'pro',
      },
    })
    textarea.dispatchEvent(new Event('input'))
    await settle()

    getButtonExact(root, '导入')?.click()
    await settle()

    expect(endpointMocks.importProviderRefreshToken).toHaveBeenCalledWith(
      'provider-1',
      expect.objectContaining({
        access_token: 'top-level-access-token',
        refresh_token: 'top-level-refresh-token',
        account_id: 'top-level-account-id',
        email: 'top-level@example.com',
        plan_type: 'team',
      }),
    )
  })

  it('rejects a nested ChatGPT auth object without an access or refresh token', async () => {
    const root = mountDialog('codex')
    await settle()

    getButton(root, '导入授权')?.click()
    await settle()

    const textarea = getImportTextarea(root)
    textarea.value = JSON.stringify({
      auth_mode: 'chatgpt',
      tokens: {
        id_token: 'nested-id-token-only',
        account_id: 'nested-account-id',
      },
      _meta: {
        email: 'nested@example.com',
      },
    })
    textarea.dispatchEvent(new Event('input'))
    await settle()

    getButtonExact(root, '导入')?.click()
    await settle()

    expect(endpointMocks.importProviderRefreshToken).not.toHaveBeenCalled()
    expect(endpointMocks.startBatchImportOAuthTask).not.toHaveBeenCalled()
  })

  it('extracts Grok account fields from a pasted browser cookie header', async () => {
    const root = mountDialog('grok')
    await settle()

    const textarea = getImportTextarea(root)
    textarea.value = 'i18nextLng=zh; cf_clearance=cf-1; sso-rw=rw-1; sso=sso-1; x-userid=user-1'
    textarea.dispatchEvent(new Event('input'))
    await settle()

    getButton(root, '导入账号')?.click()
    await settle()

    expect(endpointMocks.importProviderRefreshToken).toHaveBeenCalledWith('provider-1', expect.objectContaining({
      access_token: 'sso-1',
      sso_rw_token: 'rw-1',
      cf_cookies: 'i18nextlng=zh; cf_clearance=cf-1; x-userid=user-1',
      cf_clearance: 'cf-1',
      user_agent: expect.any(String),
      browser_profile: 'chrome136',
      user_id: 'user-1',
    }))
  })
})
