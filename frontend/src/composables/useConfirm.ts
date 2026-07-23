import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { i18n } from '@/i18n'

export type ConfirmVariant = 'danger' | 'destructive' | 'warning' | 'info' | 'question'

export interface ConfirmOptions {
  title?: string
  message: string
  confirmText?: string
  cancelText?: string
  variant?: ConfirmVariant
}

interface ConfirmState extends ConfirmOptions {
  isOpen: boolean
  resolve?: (value: boolean) => void
}

const state = ref<ConfirmState>({
  isOpen: false,
  message: '',
  title: i18n.global.t('confirm.defaults.title'),
  confirmText: i18n.global.t('confirm.defaults.confirm'),
  cancelText: i18n.global.t('confirm.defaults.cancel'),
  variant: 'question'
})

export function useConfirm() {
  const { t } = useI18n()
  /**
   * 显示确认对话框
   * @param options 对话框选项
   * @returns Promise<boolean> - true表示确认，false表示取消
   */
  const confirm = (options: ConfirmOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      state.value = {
        isOpen: true,
        title: options.title || t('confirm.defaults.title'),
        message: options.message,
        confirmText: options.confirmText || t('confirm.defaults.confirm'),
        cancelText: options.cancelText || t('confirm.defaults.cancel'),
        variant: options.variant || 'question',
        resolve
      }
    })
  }

  /**
   * 便捷方法：危险操作确认（红色主题）
   */
  const confirmDanger = (message: string, title?: string, confirmText?: string): Promise<boolean> => {
    return confirm({
      message,
      title: title || t('confirm.dangerTitle'),
      confirmText: confirmText || t('confirm.delete'),
      variant: 'danger'
    })
  }

  /**
   * 便捷方法：警告确认（黄色主题）
   */
  const confirmWarning = (message: string, title?: string): Promise<boolean> => {
    return confirm({
      message,
      title: title || t('confirm.warningTitle'),
      confirmText: t('confirm.continue'),
      variant: 'warning'
    })
  }

  /**
   * 便捷方法：信息确认（蓝色主题）
   */
  const confirmInfo = (message: string, title?: string): Promise<boolean> => {
    return confirm({
      message,
      title: title || t('confirm.infoTitle'),
      confirmText: t('confirm.ok'),
      variant: 'info'
    })
  }

  /**
   * 处理确认
   */
  const handleConfirm = () => {
    if (state.value.resolve) {
      state.value.resolve(true)
    }
    state.value.isOpen = false
  }

  /**
   * 处理取消
   */
  const handleCancel = () => {
    if (state.value.resolve) {
      state.value.resolve(false)
    }
    state.value.isOpen = false
  }

  return {
    state,
    confirm,
    confirmDanger,
    confirmWarning,
    confirmInfo,
    handleConfirm,
    handleCancel
  }
}
