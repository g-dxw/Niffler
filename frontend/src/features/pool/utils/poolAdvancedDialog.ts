import { i18n } from '@/i18n'
import type { PoolAdvancedConfig } from '@/api/endpoints/types/provider'

export type PoolHealthToggleKey =
  | 'health_policy_enabled'
  | 'probing_enabled'
  | 'account_self_check_enabled'
  | 'auto_remove_banned_keys'
  | 'skip_exhausted_accounts'

export interface PoolHealthToggleCard {
  key: PoolHealthToggleKey
  label: string
  description: string
}

export interface PoolCooldownFieldLayout {
  fields: string[]
  desktopColumnsClass: string
}

export interface PoolSecondarySectionLayout {
  wrapperClass: string
}

export interface PoolCostFieldLayout {
  fields: string[]
  desktopColumnsClass: string
}

export interface PoolAccountSelfCheckDefaults {
  enabled: boolean
  intervalMinutes: number | null
  concurrency: number | null
}

export function resolvePoolAccountSelfCheckDefaults(
  providerType: string | null | undefined,
  config: PoolAdvancedConfig | null | undefined,
): PoolAccountSelfCheckDefaults {
  const isGrokOAuth = (providerType || '').trim().toLowerCase() === 'grok_oauth'
  return {
    enabled: config?.account_self_check_enabled ?? config?.self_check_enabled ?? isGrokOAuth,
    intervalMinutes: config?.account_self_check_interval_minutes ?? config?.self_check_interval_minutes ?? (isGrokOAuth ? 30 : null),
    concurrency: config?.account_self_check_concurrency ?? config?.self_check_concurrency ?? (isGrokOAuth ? 1 : null),
  }
}

export function buildPoolHealthToggleCards(): PoolHealthToggleCard[] {
  return [
    {
      key: 'health_policy_enabled',
      label: i18n.global.t('poolAdvancedUi.healthPolicy'),
      description: i18n.global.t('poolAdvancedUi.healthPolicyDesc'),
    },
    {
      key: 'probing_enabled',
      label: i18n.global.t('poolAdvancedUi.adaptiveHotPool'),
      description: i18n.global.t('poolAdvancedUi.adaptiveHotPoolDesc'),
    },
    {
      key: 'account_self_check_enabled',
      label: i18n.global.t('poolAdvancedUi.accountCheck'),
      description: i18n.global.t('poolAdvancedUi.accountCheckDesc'),
    },
    {
      key: 'auto_remove_banned_keys',
      label: i18n.global.t('poolAdvancedUi.autoRemove'),
      description: i18n.global.t('poolAdvancedUi.autoRemoveDesc'),
    },
    {
      key: 'skip_exhausted_accounts',
      label: i18n.global.t('poolAdvancedUi.skipExhausted'),
      description: i18n.global.t('poolAdvancedUi.skipExhaustedDesc'),
    },
  ]
}

export function buildPoolCooldownFieldLayout(): PoolCooldownFieldLayout {
  return {
    fields: [
      'rate_limit_cooldown_seconds',
      'overload_cooldown_seconds',
      'sticky_session_ttl_seconds',
      'global_priority',
    ],
    desktopColumnsClass: 'xl:grid-cols-4',
  }
}

export function buildPoolSecondarySectionLayout(): PoolSecondarySectionLayout {
  return {
    wrapperClass: 'space-y-4',
  }
}

export function buildPoolCostFieldLayout(): PoolCostFieldLayout {
  return {
    fields: [
      'cost_window_seconds',
      'cost_limit_per_key_tokens',
      'cost_soft_threshold_percent',
    ],
    desktopColumnsClass: 'xl:grid-cols-3',
  }
}
