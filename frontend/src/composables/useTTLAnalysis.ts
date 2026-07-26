/**
 * TTL 分析 composable
 * 封装缓存亲和性 TTL 分析相关的状态和逻辑
 */
import { ref, computed, watch } from 'vue'
import { useToast } from '@/composables/useToast'
import {
  cacheAnalysisApi,
  type TTLAnalysisResponse,
  type CacheHitAnalysisResponse,
  type IntervalTimelineResponse
} from '@/api/cache'
import { log } from '@/utils/logger'
import type { TimeScatterData } from '@/components/charts/scatter-types'
import { useI18n } from 'vue-i18n'
import { i18n } from '@/i18n'

// 时间范围选项只保存稳定数据，显示文案在组件/计算属性中生成。
export const ANALYSIS_HOURS_OPTIONS = [
  { value: '12', unit: 'hours', count: 12 },
  { value: '24', unit: 'hours', count: 24 },
  { value: '72', unit: 'days', count: 3 },
  { value: '168', unit: 'days', count: 7 },
  { value: '336', unit: 'days', count: 14 },
  { value: '720', unit: 'days', count: 30 }
] as const

export function createAnalysisHoursOptions(
  translate: (key: string, params: { count: number }) => string
) {
  return ANALYSIS_HOURS_OPTIONS.map(option => ({
    value: option.value,
    label: translate(`ttlAnalysis.${option.unit}`, { count: option.count }),
  }))
}

// 间隔颜色配置
export const INTERVAL_COLORS = {
  short: 'rgba(34, 197, 94, 0.6)',    // green: 0-5 分钟
  medium: 'rgba(59, 130, 246, 0.6)',  // blue: 5-15 分钟
  normal: 'rgba(168, 85, 247, 0.6)',  // purple: 15-30 分钟
  long: 'rgba(249, 115, 22, 0.6)',    // orange: 30-60 分钟
  veryLong: 'rgba(239, 68, 68, 0.6)'  // red: >60 分钟
} as const

/**
 * 根据间隔时间获取对应的颜色
 */
export function getIntervalColor(interval: number): string {
  if (interval <= 5) return INTERVAL_COLORS.short
  if (interval <= 15) return INTERVAL_COLORS.medium
  if (interval <= 30) return INTERVAL_COLORS.normal
  if (interval <= 60) return INTERVAL_COLORS.long
  return INTERVAL_COLORS.veryLong
}

/**
 * 获取 TTL 推荐的 Badge 样式
 */
export function getTTLBadgeVariant(ttl: number): 'default' | 'secondary' | 'outline' | 'destructive' {
  if (ttl <= 5) return 'default'
  if (ttl <= 15) return 'secondary'
  if (ttl <= 30) return 'outline'
  return 'destructive'
}

/**
 * 获取使用频率标签
 */
export function getFrequencyLabel(ttl: number): string {
  if (ttl <= 5) return i18n.global.t('commonUi.highFrequency')
  if (ttl <= 15) return i18n.global.t('commonUi.mediumHighFrequency')
  if (ttl <= 30) return i18n.global.t('commonUi.mediumFrequency')
  return i18n.global.t('commonUi.lowFrequency')
}

/**
 * 获取使用频率样式类名
 */
export function getFrequencyClass(ttl: number): string {
  if (ttl <= 5) return 'text-success font-medium'
  if (ttl <= 15) return 'text-blue-500 font-medium'
  if (ttl <= 30) return 'text-muted-foreground'
  return 'text-destructive'
}

export function useTTLAnalysis() {
  const { error: showError, info: showInfo } = useToast()
  const { t } = useI18n()

  // 状态
  const ttlAnalysis = ref<TTLAnalysisResponse | null>(null)
  const hitAnalysis = ref<CacheHitAnalysisResponse | null>(null)
  const ttlAnalysisLoading = ref(false)
  const hitAnalysisLoading = ref(false)
  const analysisHours = ref('24')

  // 用户散点图展开状态
  const expandedUserId = ref<string | null>(null)
  const userTimelineData = ref<IntervalTimelineResponse | null>(null)
  const userTimelineLoading = ref(false)

  // 计算属性：是否正在加载
  const isLoading = computed(() => ttlAnalysisLoading.value || hitAnalysisLoading.value)

  // 获取 TTL 分析数据
  async function fetchTTLAnalysis() {
    ttlAnalysisLoading.value = true
    try {
      const hours = parseInt(analysisHours.value)
      const result = await cacheAnalysisApi.analyzeTTL({ hours })
      ttlAnalysis.value = result

      if (result.total_users_analyzed === 0) {
        const periodText = hours >= 24 ? t('ttlAnalysis.days', { count: hours / 24 }) : t('ttlAnalysis.hours', { count: hours })
        showInfo(t('ttlAnalysis.noData', { period: periodText }))
      }
    } catch (error) {
      showError(t('ttlAnalysis.fetchFailed'))
      log.error('获取 TTL 分析失败', error)
    } finally {
      ttlAnalysisLoading.value = false
    }
  }

  // 获取缓存命中分析数据
  async function fetchHitAnalysis() {
    hitAnalysisLoading.value = true
    try {
      hitAnalysis.value = await cacheAnalysisApi.analyzeHit({
        hours: parseInt(analysisHours.value)
      })
    } catch (error) {
      showError(t('ttlAnalysis.hitFetchFailed'))
      log.error('获取缓存命中分析失败', error)
    } finally {
      hitAnalysisLoading.value = false
    }
  }

  // 获取指定用户的时间线数据
  async function fetchUserTimeline(userId: string) {
    userTimelineLoading.value = true
    try {
      userTimelineData.value = await cacheAnalysisApi.getIntervalTimeline({
        hours: parseInt(analysisHours.value),
        limit: 2000,
        user_id: userId
      })
    } catch (error) {
      showError(t('ttlAnalysis.timelineFetchFailed'))
      log.error('获取用户时间线数据失败', error)
    } finally {
      userTimelineLoading.value = false
    }
  }

  // 切换用户行展开状态
  async function toggleUserExpand(userId: string) {
    if (expandedUserId.value === userId) {
      expandedUserId.value = null
      userTimelineData.value = null
    } else {
      expandedUserId.value = userId
      await fetchUserTimeline(userId)
    }
  }

  // 刷新所有分析数据
  async function refreshAnalysis() {
    expandedUserId.value = null
    userTimelineData.value = null
    await Promise.all([fetchTTLAnalysis(), fetchHitAnalysis()])
  }

  // 用户时间线散点图数据
  const userTimelineChartData = computed<TimeScatterData>(() => {
    if (!userTimelineData.value || userTimelineData.value.points.length === 0) {
      return { datasets: [] }
    }

    const points = userTimelineData.value.points

    return {
      datasets: [{
        label: i18n.global.t('commonUi.requestInterval'),
        data: points.map(p => ({ x: p.x, y: p.y })),
        backgroundColor: points.map(p => getIntervalColor(p.y)),
        borderColor: points.map(p => getIntervalColor(p.y).replace('0.6', '1')),
        pointRadius: 3,
        pointHoverRadius: 5
      }]
    }
  })

  // 监听时间范围变化
  watch(analysisHours, () => {
    refreshAnalysis()
  })

  return {
    // 状态
    ttlAnalysis,
    hitAnalysis,
    ttlAnalysisLoading,
    hitAnalysisLoading,
    analysisHours,
    expandedUserId,
    userTimelineData,
    userTimelineLoading,
    isLoading,
    userTimelineChartData,

    // 方法
    fetchTTLAnalysis,
    fetchHitAnalysis,
    fetchUserTimeline,
    toggleUserExpand,
    refreshAnalysis
  }
}
