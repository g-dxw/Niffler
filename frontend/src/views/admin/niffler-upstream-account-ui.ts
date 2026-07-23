import type {
  CreateNifflerUpstreamAccountPayload,
  NifflerUpstreamAccount,
} from '@/api/niffler-core'
import { i18n } from '@/i18n'

export type NifflerAccountAuthKind = CreateNifflerUpstreamAccountPayload['auth_kind']

export interface NifflerAccountAuthGuide {
  title: string
  description: string
  namePlaceholder: string
  contactHint: string
}

export function getNifflerAccountAuthGuide(authKind: NifflerAccountAuthKind): NifflerAccountAuthGuide {
  if (authKind === 'api_key') {
    return {
      title: i18n.global.t('upstreamUi.apiKeyAccount'),
      description: i18n.global.t('upstreamUi.apiKeyDesc'),
      namePlaceholder: i18n.global.t('upstreamUi.apiKeyPlaceholder'),
      contactHint: i18n.global.t('upstreamUi.contactHint'),
    }
  }

  if (authKind === 'custom_header') {
    return {
      title: i18n.global.t('upstreamUi.customAccount'),
      description: i18n.global.t('upstreamUi.customDesc'),
      namePlaceholder: i18n.global.t('upstreamUi.customPlaceholder'),
      contactHint: i18n.global.t('upstreamUi.customContactHint'),
    }
  }

  return {
    title: i18n.global.t('upstreamUi.oauthAccount'),
    description: i18n.global.t('upstreamUi.oauthDesc'),
    namePlaceholder: i18n.global.t('upstreamUi.oauthPlaceholder'),
    contactHint: i18n.global.t('upstreamUi.oauthContactHint'),
  }
}

export function formatNifflerAccountTestStatus(account: Pick<
  NifflerUpstreamAccount,
  'last_test_error' | 'last_tested_at_unix_ms'
>): string {
  if (account.last_test_error?.trim()) {
    return i18n.global.t('upstreamUi.testFailed')
  }
  if (account.last_tested_at_unix_ms) {
    return i18n.global.t('upstreamUi.testPassed')
  }
  return i18n.global.t('upstreamUi.notTested')
}

export function formatNifflerUnixMs(value?: number | null): string {
  if (!value || !Number.isFinite(value) || value <= 0) {
    return '-'
  }
  return new Date(value).toLocaleString('zh-CN')
}
