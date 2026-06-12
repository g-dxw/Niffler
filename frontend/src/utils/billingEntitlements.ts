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
    || Boolean((item as DailyQuotaEntitlement).limits)
  )
}

export function quotaConsumptionMultiplierLabel(
  item: Pick<DailyQuotaEntitlement, 'quota_multiplier'>,
): string | null {
  const multiplier = Number(item.quota_multiplier ?? 1)
  if (!Number.isFinite(multiplier) || multiplier <= 0 || Math.abs(multiplier - 1) < 0.000001) {
    return null
  }
  return `消耗倍率 ${formatQuotaMultiplier(multiplier)} 倍`
}

function formatQuotaMultiplier(value: number): string {
  return value.toFixed(4).replace(/\.?0+$/, '')
}
