import type { BillingEntitlement, DailyQuotaEntitlement } from '@/api/billing'

type LegacyUsageEntitlements = {
  limits?: DailyQuotaEntitlement['limits']
}

export type BillingEntitlementsInput = BillingEntitlement[] | LegacyUsageEntitlements | null | undefined

export function normalizeBillingEntitlements(input: BillingEntitlementsInput): BillingEntitlement[] {
  if (Array.isArray(input)) return input
  if (!input || typeof input !== 'object') return []

  if ('limits' in input && input.limits && typeof input.limits === 'object') {
    return [
      {
        type: 'daily_quota',
        limits: input.limits,
      },
    ]
  }

  return []
}

export function hasPackageBillingEntitlement(input: BillingEntitlementsInput): boolean {
  return normalizeBillingEntitlements(input).some((item) =>
    item.type === 'daily_quota'
    || item.type === 'membership_group'
    || Boolean((item as unknown as DailyQuotaEntitlement).limits)
  )
}

export function quotaConsumptionMultiplierLabel(
  item: Pick<DailyQuotaEntitlement, 'quota_multiplier'>,
  translate?: (key: string, params: Record<string, unknown>) => string,
): string | null {
  const multiplier = Number(item.quota_multiplier ?? 1)
  if (!Number.isFinite(multiplier) || multiplier <= 0 || Math.abs(multiplier - 1) < 0.000001) {
    return null
  }
  const value = formatQuotaMultiplier(multiplier)
  return (translate ?? ((key, params) => i18n.global.t(key, params)))('billing.multiplierLabel', { value })
}

function formatQuotaMultiplier(value: number): string {
  return value.toFixed(4).replace(/\.?0+$/, '')
}
import { i18n } from '@/i18n'
