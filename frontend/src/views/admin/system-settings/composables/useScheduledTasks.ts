import { ref, computed, type Ref } from 'vue'
import { CalendarCheck, RefreshCw } from 'lucide-vue-next'
import { useToast } from '@/composables/useToast'
import { adminApi } from '@/api/admin'
import { log } from '@/utils/logger'
import type { SystemConfig } from './useSystemConfig'
import { useI18n } from 'vue-i18n'

export function useScheduledTasks(systemConfig: Ref<SystemConfig>) {
  const { success, error } = useToast()
  const { t } = useI18n()

  const checkinConfigLoading = ref(false)

  // 签到时间的原始值（用于回滚）
  const previousCheckinTime = ref('')

  // 初始化原始值（在配置加载完成后调用）
  function initPreviousValues() {
    previousCheckinTime.value = systemConfig.value.provider_checkin_time
  }

  // 签到时间
  const checkinHour = computed(() => {
    const time = systemConfig.value.provider_checkin_time
    if (!time || !time.includes(':')) return '01'
    return time.split(':')[0]
  })

  const checkinMinute = computed(() => {
    const time = systemConfig.value.provider_checkin_time
    if (!time || !time.includes(':')) return '05'
    return time.split(':')[1]
  })

  function updateCheckinTime(hour: string, minute: string) {
    systemConfig.value.provider_checkin_time = `${hour}:${minute}`
  }

  const hasCheckinTimeChanged = computed(() => {
    return systemConfig.value.provider_checkin_time !== previousCheckinTime.value
  })

  // Toggle handlers
  async function handleProviderCheckinToggle(enabled: boolean) {
    const previousValue = systemConfig.value.enable_provider_checkin
    systemConfig.value.enable_provider_checkin = enabled
    try {
      await adminApi.updateSystemConfig(
        'enable_provider_checkin',
        enabled,
    t('systemTaskUi.providerCheckinToggle')
      )
      success(enabled ? t('scheduledTaskMessages.checkinEnabled') : t('scheduledTaskMessages.checkinDisabled'))
    } catch (err) {
      error(t('scheduledTaskMessages.saveFailed'))
      log.error('保存自动签到配置失败:', err)
      systemConfig.value.enable_provider_checkin = previousValue
    }
  }

  async function handleOAuthTokenRefreshToggle(enabled: boolean) {
    const previousValue = systemConfig.value.enable_oauth_token_refresh
    systemConfig.value.enable_oauth_token_refresh = enabled
    try {
      await adminApi.updateSystemConfig(
        'enable_oauth_token_refresh',
        enabled,
    t('systemTaskUi.tokenRefreshToggle')
      )
      success(enabled ? t('scheduledTaskMessages.oauthRefreshEnabled') : t('scheduledTaskMessages.oauthRefreshDisabled'))
    } catch (err) {
      error(t('scheduledTaskMessages.saveFailed'))
      log.error('保存 OAuth Token 自动刷新配置失败:', err)
      systemConfig.value.enable_oauth_token_refresh = previousValue
    }
  }

  // Cancel handlers
  function handleCheckinTimeCancel() {
    systemConfig.value.provider_checkin_time = previousCheckinTime.value
  }

  // Save handlers
  async function handleCheckinTimeSave() {
    const newTime = systemConfig.value.provider_checkin_time
    if (!newTime || !/^\d{2}:\d{2}$/.test(newTime)) {
      error(t('scheduledTaskMessages.invalidTime'))
      return
    }

    checkinConfigLoading.value = true
    try {
      await adminApi.updateSystemConfig(
        'provider_checkin_time',
        newTime,
    t('systemTaskUi.checkinTime')
      )
      previousCheckinTime.value = newTime
      success(t('scheduledTaskMessages.checkinTimeSaved', { time: newTime }))
    } catch (err) {
      error(t('scheduledTaskMessages.checkinTimeFailed'))
      log.error('保存签到时间失败:', err)
    } finally {
      checkinConfigLoading.value = false
    }
  }

  // 定时任务配置列表
  const scheduledTasks = computed(() => [
    {
      id: 'provider-checkin',
      icon: CalendarCheck,
    title: t('systemTaskUi.providerCheckin'),
    description: t('systemTaskUi.providerCheckinDesc'),
      enabled: systemConfig.value.enable_provider_checkin,
      hasTimeConfig: true,
      hour: checkinHour.value,
      minute: checkinMinute.value,
      updateTime: updateCheckinTime,
      hasChanges: hasCheckinTimeChanged.value,
      loading: checkinConfigLoading.value,
      onToggle: handleProviderCheckinToggle,
      onSave: handleCheckinTimeSave,
      onCancel: handleCheckinTimeCancel,
    },
    {
      id: 'oauth-token-refresh',
      icon: RefreshCw,
    title: t('systemTaskUi.tokenRefresh'),
    description: t('systemTaskUi.tokenRefreshDesc'),
      enabled: systemConfig.value.enable_oauth_token_refresh,
      hasTimeConfig: false,
      hour: '',
      minute: '',
      updateTime: () => {},
      hasChanges: false,
      loading: false,
      onToggle: handleOAuthTokenRefreshToggle,
      onSave: () => {},
      onCancel: () => {},
    },
  ])

  return {
    checkinConfigLoading,
    scheduledTasks,
    initPreviousValues,
  }
}
