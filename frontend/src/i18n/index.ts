import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN'
import enUS from './locales/en-US'

export const SUPPORTED_LOCALES = ['zh-CN', 'en-US'] as const
export type AppLocale = (typeof SUPPORTED_LOCALES)[number]

const messages = {
  'zh-CN': {
    ...zhCN,
    poolManagement: {
      ...zhCN.poolManagement,
      refresh: '刷新',
      refreshing: '刷新中...',
      refreshData: '刷新数据',
      refreshDataQuota: '刷新数据和额度',
      refreshComplete: '刷新完成：成功 {success}，失败 {failed}，跳过 {skipped}',
      refreshFailedDetail: '刷新失败：{error}，跳过 {skipped}',
      refreshPageFailed: '刷新当前页失败',
      refreshDataCooldown: '{wait} 后可再次刷新',
      refreshEligible: '可刷新 {eligible} / {total}',
      refreshProviderFailed: '刷新提供商失败',
      refreshEndpointsFailed: '刷新端点失败',
      unnamedAccount: '未命名账号',
      selectAccount: '选择账号 {name}',
    },
  },
  'en-US': {
    ...enUS,
    poolManagement: {
      ...enUS.poolManagement,
      refresh: 'Refresh',
      refreshing: 'Refreshing...',
      refreshData: 'Refresh data',
      refreshDataQuota: 'Refresh data and quota',
      refreshComplete: 'Refresh complete: {success} succeeded, {failed} failed, {skipped} skipped',
      refreshFailedDetail: 'Refresh failed: {error}; {skipped} skipped',
      refreshPageFailed: 'Failed to refresh current page',
      refreshDataCooldown: 'Refresh available again in {wait}',
      refreshEligible: 'Eligible to refresh: {eligible} / {total}',
      refreshProviderFailed: 'Failed to refresh provider',
      refreshEndpointsFailed: 'Failed to refresh endpoints',
      unnamedAccount: 'Unnamed account',
      selectAccount: 'Select account {name}',
    },
  },
}

function resolveInitialLocale(): AppLocale {
  if (import.meta.env.MODE === 'test') return 'zh-CN'
  const saved = localStorage.getItem('niffler-locale')
  if (saved === 'zh-CN' || saved === 'en-US') return saved
  return navigator.language.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US'
}

export const i18n = createI18n({
  legacy: false,
  locale: resolveInitialLocale(),
  fallbackLocale: 'zh-CN',
  messages,
})

export function setAppLocale(locale: AppLocale) {
  i18n.global.locale.value = locale
  localStorage.setItem('niffler-locale', locale)
  document.documentElement.lang = locale
}
