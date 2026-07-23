/**
 * Demo Mode Configuration
 * 用于 GitHub Pages 等静态托管环境的演示模式
 */

// 检测是否为演示模式环境
export function isDemoMode(): boolean {
  const hostname = window.location.hostname
  return (
    hostname.includes('github.io') ||
    hostname.includes('vercel.app') ||
    hostname.includes('netlify.app') ||
    hostname.includes('pages.dev') ||
    import.meta.env.VITE_DEMO_MODE === 'true'
  )
}

import { i18n } from '@/i18n'

// Demo 账号配置
export const DEMO_ACCOUNTS = {
  admin: {
    email: 'admin@demo.aether.io',
    password: 'demo123',
    hint: i18n.global.t('staticUi.adminAccount')
  },
  user: {
    email: 'user@demo.aether.io',
    password: 'demo123',
    hint: i18n.global.t('staticUi.userAccount')
  }
} as const

// Demo 模式提示信息
export const DEMO_MODE_INFO = {
  title: i18n.global.t('staticUi.demoMode'),
  description: i18n.global.t('staticUi.demoDescription'),
  accountHint: i18n.global.t('staticUi.demoAccounts')
} as const
