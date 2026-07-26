import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApp, nextTick, type App } from '@/test/vue'

import PaymentGatewaySettings from '../PaymentGatewaySettings.vue'

const gatewayConfig = {
  provider: 'epay' as const,
  enabled: false,
  endpoint_url: '',
  callback_base_url: '',
  merchant_id: '',
  has_secret: false,
  pay_currency: 'CNY',
  usd_exchange_rate: 7.2,
  min_recharge_usd: 1,
  channels: [
    { channel: 'alipay', display_name: '支付宝' },
    { channel: 'wxpay', display_name: '微信支付' },
  ],
  created_at: null,
  updated_at: null,
}

const { epayGet, dodopayGet } = vi.hoisted(() => ({
  epayGet: vi.fn(),
  dodopayGet: vi.fn(),
}))

vi.mock('@/api/billing', () => ({
  epayGatewayApi: {
    get: epayGet,
    update: vi.fn(),
    test: vi.fn(),
  },
  dodopayGatewayApi: {
    get: dodopayGet,
    update: vi.fn(),
    test: vi.fn(),
  },
}))

vi.mock('@/composables/useToast', () => ({
  useToast: () => ({ success: vi.fn(), error: vi.fn() }),
}))

vi.mock('@/utils/logger', () => ({
  log: { error: vi.fn() },
}))

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

async function flushView() {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

afterEach(() => {
  for (const { app, root } of mountedApps.splice(0)) {
    app.unmount()
    root.remove()
  }
  epayGet.mockReset()
  dodopayGet.mockReset()
  document.body.innerHTML = ''
})

describe('payment gateway settings', () => {
  it('renders provider tabs and the active provider without computed-ref errors', async () => {
    epayGet.mockResolvedValue(gatewayConfig)
    dodopayGet.mockResolvedValue({ ...gatewayConfig, provider: 'dodopay' })

    const root = document.createElement('div')
    document.body.appendChild(root)
    const app = createApp(PaymentGatewaySettings)
    app.mount(root)
    mountedApps.push({ app, root })
    await flushView()

    expect(epayGet).toHaveBeenCalledOnce()
    expect(root.textContent).toContain('易支付')
    expect(root.textContent).toContain('DoDoPay')
    expect(root.textContent).toContain('易支付 收款信息')
  })
})
