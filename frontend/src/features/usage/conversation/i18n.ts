import { i18n } from '@/i18n'

export function conversationText(key: string, params?: Record<string, unknown>): string {
  return i18n.global.t(`conversationUi.${key}`, params)
}
