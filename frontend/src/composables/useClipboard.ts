import { useToast } from './useToast'
import { log } from '@/utils/logger'
import { useI18n } from 'vue-i18n'

export function useClipboard() {
  const { success, error: showError } = useToast()
  const { t } = useI18n()

  async function copyToClipboard(text: string, showToast = true): Promise<boolean> {
    try {
      if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(text)
        if (showToast) success(t('clipboard.copied'))
        return true
      }

      // Fallback for non-secure contexts
      const textArea = document.createElement('textarea')
      textArea.value = text
      textArea.style.position = 'fixed'
      textArea.style.left = '-999999px'
      textArea.style.top = '-999999px'
      document.body.appendChild(textArea)
      textArea.focus()
      textArea.select()

      try {
        const successful = document.execCommand('copy')
        if (successful) {
          if (showToast) success(t('clipboard.copied'))
          return true
        }
        if (showToast) showError(t('clipboard.copyFailed'))
        return false
      } finally {
        document.body.removeChild(textArea)
      }
    } catch (err) {
      log.error('复制失败:', err)
      if (showToast) showError(t('clipboard.copyFailedManual'))
      return false
    }
  }

  return { copyToClipboard }
}
