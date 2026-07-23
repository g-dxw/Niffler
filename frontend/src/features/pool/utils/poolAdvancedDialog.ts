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
import { i18n } from '@/i18n'
