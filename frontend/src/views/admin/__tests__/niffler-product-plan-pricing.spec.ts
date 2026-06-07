import { describe, expect, it } from 'vitest'

import {
  buildProductPlanModelPriceRows,
  formatProductPlanModelPrice,
  getProductPlanModelEffectiveMultiplier,
} from '../niffler-product-plan-pricing'

const model = {
  default_price_per_request: 0.01,
  default_tiered_pricing: {
    tiers: [
      {
        up_to: null,
        input_price_per_1m: 3,
        output_price_per_1m: 15,
        cache_creation_price_per_1m: 3.75,
        cache_read_price_per_1m: 0.3,
      },
    ],
  },
}

describe('niffler product plan pricing helpers', () => {
  it('uses model override before the product plan default multiplier', () => {
    expect(getProductPlanModelEffectiveMultiplier(1.2, 0.8)).toBe(0.8)
    expect(getProductPlanModelEffectiveMultiplier(1.2, null)).toBe(1.2)
    expect(getProductPlanModelEffectiveMultiplier('', '')).toBe(1)
  })

  it('builds wallet sales price rows from base model prices', () => {
    const rows = buildProductPlanModelPriceRows(model, 2)

    expect(rows).toEqual([
      { key: 'input', label: '输入', basePrice: 3, salesPrice: 6, unit: '/M tokens' },
      { key: 'output', label: '输出', basePrice: 15, salesPrice: 30, unit: '/M tokens' },
      { key: 'cache_creation', label: '缓存创建', basePrice: 3.75, salesPrice: 7.5, unit: '/M tokens' },
      { key: 'cache_read', label: '缓存读取', basePrice: 0.3, salesPrice: 0.6, unit: '/M tokens' },
      { key: 'request', label: '固定请求费', basePrice: 0.01, salesPrice: 0.02, unit: '/次' },
    ])
  })

  it('formats compact price labels', () => {
    expect(formatProductPlanModelPrice(3, '/M tokens')).toBe('$3/M tokens')
    expect(formatProductPlanModelPrice(0.000012, '/次')).toBe('$0.000012/次')
    expect(formatProductPlanModelPrice(null, '/次')).toBe('-')
  })
})
