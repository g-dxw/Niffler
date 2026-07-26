import type { GlobalModelResponse } from '@/api/global-models'
import { i18n } from '@/i18n'

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
  const rows: ProductPlanModelPriceRow[] = [
    buildTokenPriceRow('input', i18n.global.t('pricingUi.input'), getFirstTierPrice(model, 'input_price_per_1m'), multiplier),
    buildTokenPriceRow('output', i18n.global.t('pricingUi.output'), getFirstTierPrice(model, 'output_price_per_1m'), multiplier),
    buildTokenPriceRow(
      'cache_creation',
      i18n.global.t('pricingUi.cacheCreation'),
      getFirstTierPrice(model, 'cache_creation_price_per_1m'),
      multiplier
    ),
    buildTokenPriceRow('cache_read', i18n.global.t('pricingUi.cacheRead'), getFirstTierPrice(model, 'cache_read_price_per_1m'), multiplier),
    {
      key: 'request',
      label: i18n.global.t('pricingUi.fixedRequest'),
      basePrice: toNonNegativeFiniteNumber(model.default_price_per_request),
      salesPrice: multiplyPrice(toNonNegativeFiniteNumber(model.default_price_per_request), multiplier),
      unit: i18n.global.t('pricingUi.perRequest'),
    },
  ]
  return rows.filter(row => row.basePrice !== null)
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
