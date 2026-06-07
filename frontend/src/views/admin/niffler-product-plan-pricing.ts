import type { GlobalModelResponse } from '@/api/global-models'

export type ProductPlanModelPriceKey =
  | 'input'
  | 'output'
  | 'cache_creation'
  | 'cache_read'
  | 'request'

export interface ProductPlanModelPriceRow {
  key: ProductPlanModelPriceKey
  label: string
  basePrice: number | null
  salesPrice: number | null
  unit: string
}

type PriceField =
  | 'input_price_per_1m'
  | 'output_price_per_1m'
  | 'cache_creation_price_per_1m'
  | 'cache_read_price_per_1m'

export function getProductPlanModelEffectiveMultiplier(
  planSalesMultiplier: number | string | null | undefined,
  modelSalesMultiplierOverride: number | string | null | undefined
): number {
  const override = toNonNegativeFiniteNumber(modelSalesMultiplierOverride)
  if (override !== null) return override
  return toNonNegativeFiniteNumber(planSalesMultiplier) ?? 1
}

export function buildProductPlanModelPriceRows(
  model: Pick<GlobalModelResponse, 'default_price_per_request' | 'default_tiered_pricing'> | null | undefined,
  multiplier: number
): ProductPlanModelPriceRow[] {
  if (!model) return []
  return [
    buildTokenPriceRow('input', '输入', getFirstTierPrice(model, 'input_price_per_1m'), multiplier),
    buildTokenPriceRow('output', '输出', getFirstTierPrice(model, 'output_price_per_1m'), multiplier),
    buildTokenPriceRow(
      'cache_creation',
      '缓存创建',
      getFirstTierPrice(model, 'cache_creation_price_per_1m'),
      multiplier
    ),
    buildTokenPriceRow('cache_read', '缓存读取', getFirstTierPrice(model, 'cache_read_price_per_1m'), multiplier),
    {
      key: 'request',
      label: '固定请求费',
      basePrice: toNonNegativeFiniteNumber(model.default_price_per_request),
      salesPrice: multiplyPrice(toNonNegativeFiniteNumber(model.default_price_per_request), multiplier),
      unit: '/次',
    },
  ].filter(row => row.basePrice !== null)
}

export function formatProductPlanModelPrice(value: number | null, unit: string): string {
  if (value === null) return '-'
  return `$${formatCompactNumber(value)}${unit}`
}

function buildTokenPriceRow(
  key: ProductPlanModelPriceKey,
  label: string,
  basePrice: number | null,
  multiplier: number
): ProductPlanModelPriceRow {
  return {
    key,
    label,
    basePrice,
    salesPrice: multiplyPrice(basePrice, multiplier),
    unit: '/M tokens',
  }
}

function getFirstTierPrice(
  model: Pick<GlobalModelResponse, 'default_tiered_pricing'>,
  field: PriceField
): number | null {
  const firstTier = model.default_tiered_pricing?.tiers?.[0]
  return toNonNegativeFiniteNumber(firstTier?.[field])
}

function multiplyPrice(value: number | null, multiplier: number): number | null {
  if (value === null) return null
  return value * multiplier
}

function toNonNegativeFiniteNumber(value: number | string | null | undefined): number | null {
  if (value === null || value === undefined || value === '') return null
  const parsed = Number(value)
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : null
}

function formatCompactNumber(value: number): string {
  const fixed = value >= 1 ? value.toFixed(4) : value.toFixed(6)
  return fixed.replace(/\.?0+$/, '')
}
