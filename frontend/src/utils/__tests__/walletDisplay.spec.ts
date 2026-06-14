import { describe, expect, it } from 'vitest'

import { paymentOrderMethodLabel } from '../walletDisplay'

describe('paymentOrderMethodLabel', () => {
  it('shows the real DoDoPay channel when callback records it', () => {
    expect(paymentOrderMethodLabel({
      payment_method: 'dodopay',
      payment_provider: 'dodopay',
      payment_channel: 'WECHAT',
    })).toBe('微信支付')
    expect(paymentOrderMethodLabel({
      payment_method: 'dodopay',
      payment_provider: 'dodopay',
      payment_channel: 'ALIPAY',
    })).toBe('支付宝支付')
  })

  it('does not treat non-gateway channel as payment channel', () => {
    expect(paymentOrderMethodLabel({
      payment_method: 'gift_code',
      payment_provider: 'redeem_code',
      payment_channel: 'gift',
    })).toBe('礼品卡')
  })
})
