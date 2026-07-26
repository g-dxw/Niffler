<script setup lang="ts">
import { ref, computed, onMounted, watch, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import Card from '@/components/ui/card.vue'
import Button from '@/components/ui/button.vue'
import Badge from '@/components/ui/badge.vue'
import Table from '@/components/ui/table.vue'
import TableBody from '@/components/ui/table-body.vue'
import TableCell from '@/components/ui/table-cell.vue'
import TableHead from '@/components/ui/table-head.vue'
import TableHeader from '@/components/ui/table-header.vue'
import TableRow from '@/components/ui/table-row.vue'
import Input from '@/components/ui/input.vue'
import Pagination from '@/components/ui/pagination.vue'
import RefreshButton from '@/components/ui/refresh-button.vue'
import Select from '@/components/ui/select.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import SelectValue from '@/components/ui/select-value.vue'
import ScatterChart from '@/components/charts/ScatterChart.vue'
import { Trash2, Eraser, Search, X, BarChart3, ChevronDown, ChevronRight, Database, ArrowRight, HardDrive } from 'lucide-vue-next'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { cacheApi, modelMappingCacheApi, redisCacheApi, type CacheStats, type CacheConfig, type UserAffinity, type ModelMappingCacheStats, type RedisCacheCategoriesResponse } from '@/api/cache'
import type { TTLAnalysisUser } from '@/api/cache'
import { formatNumber, formatTokens, formatCost, formatRemainingTime } from '@/utils/format'
import {
  useTTLAnalysis,
  getTTLBadgeVariant,
  getFrequencyClass
} from '@/composables/useTTLAnalysis'
import { log } from '@/utils/logger'
import { formatApiFormat } from '@/api/endpoints/types/api-format'

// ==================== 缓存统计与亲和性列表 ====================

const stats = ref<CacheStats | null>(null)
const { t, locale } = useI18n()
const analysisHoursOptions = computed(() => [
  { value: '12', label: t('ttlAnalysis.hours', { count: 12 }) },
  { value: '24', label: t('ttlAnalysis.hours', { count: 24 }) },
  { value: '72', label: t('ttlAnalysis.days', { count: 3 }) },
  { value: '168', label: t('ttlAnalysis.days', { count: 7 }) },
  { value: '336', label: t('ttlAnalysis.days', { count: 14 }) },
  { value: '720', label: t('ttlAnalysis.days', { count: 30 }) },
])

function frequencyLabel(ttl: number): string {
  if (ttl <= 5) return t('ttlAnalysis.frequencyHigh')
  if (ttl <= 15) return t('ttlAnalysis.frequencyMediumHigh')
  if (ttl <= 30) return t('ttlAnalysis.frequencyMedium')
  return t('ttlAnalysis.frequencyLow')
}
const config = ref<CacheConfig | null>(null)
const loading = ref(false)
const affinityList = ref<UserAffinity[]>([])
const listLoading = ref(false)
const tableKeyword = ref('')
const matchedUserId = ref<string | null>(null)
const clearingRowAffinityKey = ref<string | null>(null)
const clearingAllAffinity = ref(false)
const currentPage = ref(1)
const pageSize = ref(20)
const currentTime = ref(Math.floor(Date.now() / 1000))
const isPageVisible = ref(typeof document === 'undefined' ? true : !document.hidden)
const nextExpireAt = ref<number | null>(null)

// ==================== 模型映射缓存 ====================

const modelMappingStats = ref<ModelMappingCacheStats | null>(null)
const modelMappingLoading = ref(false)
const clearingModelMapping = ref(false)
const clearingModelName = ref<string | null>(null)

// ==================== Redis 缓存分类管理 ====================

const redisCacheData = ref<RedisCacheCategoriesResponse | null>(null)
const redisCacheLoading = ref(false)
const clearingCategory = ref<string | null>(null)

const { success: showSuccess, error: showError, info: showInfo } = useToast()
const { confirm: showConfirm } = useConfirm()

let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null
let skipNextKeywordWatch = false
let countdownTimer: ReturnType<typeof setInterval> | null = null

// ==================== TTL 分析 (使用 composable) ====================

const {
  ttlAnalysis,
  hitAnalysis,
  ttlAnalysisLoading,
  analysisHours,
  expandedUserId,
  userTimelineData,
  userTimelineLoading,
  userTimelineChartData,
  toggleUserExpand,
  refreshAnalysis
} = useTTLAnalysis()

// ==================== 计算属性 ====================

const paginatedAffinityList = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return affinityList.value.slice(start, end)
})

// ==================== 缓存统计方法 ====================

async function fetchCacheStats() {
  loading.value = true
  try {
    stats.value = await cacheApi.getStats()
  } catch (error) {
    showError(t('cacheMonitoring.loadStatsFailed'))
    log.error('Failed to load cache statistics', error)
  } finally {
    loading.value = false
  }
}

async function fetchCacheConfig() {
  try {
    config.value = await cacheApi.getConfig()
  } catch (error) {
    log.error('Failed to load cache configuration', error)
  }
}

async function fetchAffinityList(keyword?: string) {
  listLoading.value = true
  try {
    const response = await cacheApi.listAffinities(keyword)
    affinityList.value = response.items
    matchedUserId.value = response.matched_user_id ?? null
    currentTime.value = Math.floor(Date.now() / 1000)
    pruneExpiredAffinities(currentTime.value, true)

    if (keyword && response.total === 0) {
      showInfo(t('cacheMonitoring.noMatch'))
    }
  } catch (error) {
    showError(t('cacheMonitoring.loadListFailed'))
    log.error('Failed to load cache list', error)
  } finally {
    listLoading.value = false
  }
}

async function resetAffinitySearch() {
  if (searchDebounceTimer) {
    clearTimeout(searchDebounceTimer)
    searchDebounceTimer = null
  }

  if (!tableKeyword.value) {
    currentPage.value = 1
    await fetchAffinityList()
    return
  }

  skipNextKeywordWatch = true
  tableKeyword.value = ''
  currentPage.value = 1
  await fetchAffinityList()
}

async function clearSingleAffinity(item: UserAffinity) {
  const affinityKey = item.affinity_key?.trim()
  const endpointId = item.endpoint_id?.trim()
  const modelId = item.global_model_id?.trim()
  const apiFormat = item.api_format?.trim()

  if (!affinityKey || !endpointId || !modelId || !apiFormat) {
    showError(t('cacheMonitoring.incompleteRecord'))
    return
  }

  const label = item.user_api_key_name || affinityKey
  const modelLabel = item.model_display_name || item.model_name || modelId
  const confirmed = await showConfirm({
    title: t('cacheMonitoring.confirmClear'),
    message: t('cacheMonitoring.confirmSingle', { label, model: modelLabel }),
    confirmText: t('cacheMonitoring.confirmClear'),
    variant: 'destructive'
  })

  if (!confirmed) return

  clearingRowAffinityKey.value = affinityKey
  try {
    await cacheApi.clearSingleAffinity(
      affinityKey,
      endpointId,
      modelId,
      apiFormat,
      item.client_family,
      item.session_hash
    )
    showSuccess(t('cacheMonitoring.clearSuccess'))
    await fetchCacheStats()
    await fetchAffinityList(tableKeyword.value.trim() || undefined)
  } catch (error) {
    showError(t('cacheMonitoring.clearFailed'))
    log.error('Failed to clear cache entry', error)
  } finally {
    clearingRowAffinityKey.value = null
  }
}

async function clearAllCache() {
  const firstConfirm = await showConfirm({
    title: t('cacheMonitoring.dangerousAction'),
    message: t('cacheMonitoring.clearAllWarning'),
    confirmText: t('cacheMonitoring.continue'),
    variant: 'destructive'
  })
  if (!firstConfirm) return

  const secondConfirm = await showConfirm({
    title: t('cacheMonitoring.confirmAgain'),
    message: t('cacheMonitoring.clearAllAgain'),
    confirmText: t('cacheMonitoring.confirmClear'),
    variant: 'destructive'
  })
  if (!secondConfirm) return

  clearingAllAffinity.value = true
  try {
    await cacheApi.clearAllCache()
    showSuccess(t('cacheMonitoring.clearAllSuccess'))
    await fetchCacheStats()
    await fetchAffinityList(tableKeyword.value.trim() || undefined)
  } catch (error) {
    showError(t('cacheMonitoring.clearFailed'))
    log.error('Failed to clear all cache', error)
  } finally {
    clearingAllAffinity.value = false
  }
}

// ==================== 工具方法 ====================

function getRemainingTime(expireAt?: number): string {
  return formatRemainingTime(expireAt, currentTime.value)
}

function truncateMiddle(value: string, head = 8, tail = 6): string {
  const normalized = value.trim()
  if (!normalized) return '---'
  if (normalized.length <= head + tail + 3) return normalized
  return `${normalized.slice(0, head)}...${normalized.slice(-tail)}`
}

function affinityUserLabel(item: UserAffinity): string {
  return item.username || item.email || item.user_id || t('cacheMonitoring.unknown')
}

function affinityUserTitle(item: UserAffinity): string | undefined {
  return item.email || item.username || item.user_id || undefined
}

function affinityUserApiKeyLabel(item: UserAffinity): string {
  return item.user_api_key_name || item.user_api_key_prefix || truncateMiddle(item.affinity_key)
}

function affinityModelLabel(item: UserAffinity): string {
  return item.model_display_name || item.model_name || item.global_model_id || '---'
}

function affinityModelSubtitle(item: UserAffinity): string {
  if (!item.model_display_name || !item.model_name || item.model_display_name === item.model_name) {
    return ''
  }
  return item.model_name
}

function formatClientFamily(family?: string | null): string {
  switch (family) {
    case 'codex':
      return 'Codex'
    case 'opencode':
      return 'OpenCode'
    case 'claude_code':
      return 'Claude Code'
    case 'openai_js_sdk':
      return 'OpenAI JS SDK'
    case 'generic':
  return t('cacheMonitoring.general')
    case undefined:
    case null:
    case '':
  return t('cacheMonitoring.unknown')
    default:
      return family
  }
}

function providerKeyLabel(item: UserAffinity): string {
  if (item.key_name) return item.key_name
  if (item.key_prefix === '[OAuth Token]') return t('cacheMonitoring.oauthAuth')
  return item.key_prefix || '---'
}

function providerKeyTitle(item: UserAffinity): string | undefined {
  if (item.key_name && item.key_prefix) return `${item.key_name} · ${item.key_prefix}`
  return item.key_name || item.key_prefix || undefined
}

function formatAffinityRequestCount(item: UserAffinity): string {
  if (item.request_count_known === false) return '—'
  return formatNumber(item.request_count || 0)
}

function formatAffinityRequestCountUnit(item: UserAffinity): string {
  const count = formatAffinityRequestCount(item)
  return count === '—' ? count : t('cacheMonitoring.times', { count })
}

function formatIntervalDescription(user: TTLAnalysisUser): string {
  const p90 = user.percentiles.p90
  if (p90 === null || p90 === undefined) return '-'
  if (p90 < 1) {
    const seconds = Math.round(p90 * 60)
    return t('cacheMonitoring.intervalSeconds', { value: seconds })
  }
  return t('cacheMonitoring.intervalMinutes', { value: p90.toFixed(1) })
}

function handlePageChange() {
  window.scrollTo({ top: 0, behavior: 'smooth' })
}

function recalculateNextExpireAt(now: number = currentTime.value) {
  let nearestExpireAt: number | null = null

  for (const item of affinityList.value) {
    if (!item.expire_at || item.expire_at <= now) continue
    if (nearestExpireAt === null || item.expire_at < nearestExpireAt) {
      nearestExpireAt = item.expire_at
    }
  }

  nextExpireAt.value = nearestExpireAt
}

function pruneExpiredAffinities(now: number, silent = false) {
  const beforeCount = affinityList.value.length
  const activeItems = affinityList.value.filter(
    item => item.expire_at && item.expire_at > now
  )

  if (activeItems.length === beforeCount) {
    recalculateNextExpireAt(now)
    return
  }

  affinityList.value = activeItems
  recalculateNextExpireAt(now)

  if (!silent) {
    showInfo(t('cacheMonitoring.expiredRemoved', { count: beforeCount - activeItems.length }))
  }
}

// ==================== 定时器管理 ====================

function startCountdown() {
  if (!isPageVisible.value) return
  if (countdownTimer) clearInterval(countdownTimer)

  countdownTimer = setInterval(() => {
    currentTime.value = Math.floor(Date.now() / 1000)

    if (nextExpireAt.value !== null && currentTime.value >= nextExpireAt.value) {
      pruneExpiredAffinities(currentTime.value)
    }
  }, 1000)
}

function stopCountdown() {
  if (countdownTimer) {
    clearInterval(countdownTimer)
    countdownTimer = null
  }
}

function handleVisibilityChange() {
  isPageVisible.value = !document.hidden
  if (!isPageVisible.value) {
    stopCountdown()
    return
  }
  currentTime.value = Math.floor(Date.now() / 1000)
  if (nextExpireAt.value !== null && currentTime.value >= nextExpireAt.value) {
    pruneExpiredAffinities(currentTime.value)
  }
  startCountdown()
}

// ==================== 模型映射缓存方法 ====================

async function fetchModelMappingStats() {
  modelMappingLoading.value = true
  try {
    modelMappingStats.value = await modelMappingCacheApi.getStats()
  } catch (error) {
    showError(t('cacheMonitoring.loadMappingFailed'))
    log.error('Failed to load model mapping cache statistics', error)
  } finally {
    modelMappingLoading.value = false
  }
}

async function clearAllModelMappingCache() {
  const confirmed = await showConfirm({
    title: t('cacheMonitoring.confirmClear'),
    message: t('cacheMonitoring.confirmAllMappings'),
    confirmText: t('cacheMonitoring.confirmClear'),
    variant: 'destructive'
  })

  if (!confirmed) return

  clearingModelMapping.value = true
  try {
    const result = await modelMappingCacheApi.clearAll()
    showSuccess(t('cacheMonitoring.keysCleared', { count: result.deleted_count }))
    await fetchModelMappingStats()
  } catch (error) {
    showError(t('cacheMonitoring.clearMappingFailed'))
    log.error('Failed to clear model mapping cache', error)
  } finally {
    clearingModelMapping.value = false
  }
}

async function clearModelMappingByName(modelName: string) {
  clearingModelName.value = modelName
  try {
    await modelMappingCacheApi.clearByName(modelName)
    showSuccess(t('cacheMonitoring.modelMappingCleared', { model: modelName }))
    await fetchModelMappingStats()
  } catch (error) {
    showError(t('cacheMonitoring.clearFailed'))
    log.error('Failed to clear model mapping cache', error)
  } finally {
    clearingModelName.value = null
  }
}

async function clearProviderModelMapping(providerId: string, globalModelId: string, displayName?: string) {
  const confirmed = await showConfirm({
    title: t('cacheMonitoring.confirmClear'),
    message: t('cacheMonitoring.confirmProviderMapping', { name: displayName || t('cacheMonitoring.providerMapping') }),
    confirmText: t('cacheMonitoring.confirmClear'),
    variant: 'destructive'
  })

  if (!confirmed) return

  try {
    await modelMappingCacheApi.clearProviderModel(providerId, globalModelId)
    showSuccess(t('cacheMonitoring.providerMappingCleared'))
    await fetchModelMappingStats()
  } catch (error) {
    showError(t('cacheMonitoring.clearFailed'))
    log.error('Failed to clear provider model mapping cache', error)
  }
}

// ==================== Redis 缓存分类管理方法 ====================

async function fetchRedisCacheCategories() {
  redisCacheLoading.value = true
  try {
    redisCacheData.value = await redisCacheApi.getCategories()
  } catch (error) {
    showError(t('cacheMonitoring.loadRedisFailed'))
    log.error('Failed to load Redis cache categories', error)
  } finally {
    redisCacheLoading.value = false
  }
}

async function clearRedisCategory(categoryKey: string, categoryName: string, count: number) {
  if (count === 0) {
    showInfo(t('cacheMonitoring.categoryEmpty', { name: categoryName }))
    return
  }
  const confirmed = await showConfirm({
    title: t('cacheMonitoring.clearCategoryTitle', { name: categoryName }),
    message: t('cacheMonitoring.clearCategoryConfirm', { name: categoryName, count }),
  })
  if (!confirmed) return

  clearingCategory.value = categoryKey
  try {
    const result = await redisCacheApi.clearCategory(categoryKey)
    showSuccess(t('cacheMonitoring.categoryCleared', { name: categoryName, count: result.deleted_count }))
    await fetchRedisCacheCategories()
  } catch (error) {
    showError(t('cacheMonitoring.clearCategoryFailed', { name: categoryName }))
    log.error('Failed to clear Redis cache category', error)
  } finally {
    clearingCategory.value = null
  }
}

const redisCategoriesWithKeys = computed(() => {
  if (!redisCacheData.value?.categories) return []
  return redisCacheData.value.categories.filter(c => c.count > 0)
})

const redisCategoriesEmpty = computed(() => {
  if (!redisCacheData.value?.categories) return []
  return redisCacheData.value.categories.filter(c => c.count === 0)
})

function formatTTL(ttl: number | null): string {
  if (ttl === null || ttl < 0) return '-'
  if (ttl < 60) return `${ttl}s`
  const minutes = Math.floor(ttl / 60)
  const seconds = ttl % 60
  if (seconds === 0) return `${minutes}m`
  return `${minutes}m${seconds}s`
}

function getUnmappedStatusBadge(status: string): { variant: 'default' | 'secondary' | 'destructive' | 'outline', text: string } {
  switch (status) {
    case 'not_found':
      return { variant: 'secondary', text: t('cacheMonitoring.notFound') }
    case 'invalid':
      return { variant: 'destructive', text: t('cacheMonitoring.invalid') }
    case 'error':
      return { variant: 'destructive', text: t('cacheMonitoring.error') }
    default:
      return { variant: 'outline', text: status }
  }
}

// ==================== 刷新所有数据 ====================

async function refreshData() {
  await Promise.all([
    fetchCacheStats(),
    fetchCacheConfig(),
    fetchAffinityList(),
    fetchModelMappingStats(),
    fetchRedisCacheCategories()
  ])
}

// ==================== 生命周期 ====================

watch(tableKeyword, (value) => {
  if (skipNextKeywordWatch) {
    skipNextKeywordWatch = false
    return
  }

  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)

  const keyword = value.trim()
  searchDebounceTimer = setTimeout(() => {
    fetchAffinityList(keyword || undefined)
    searchDebounceTimer = null
  }, 600)
})

onMounted(() => {
  document.addEventListener('visibilitychange', handleVisibilityChange)
  fetchCacheStats()
  fetchCacheConfig()
  fetchAffinityList()
  fetchModelMappingStats()
  fetchRedisCacheCategories()
  startCountdown()
  refreshAnalysis()
})

onBeforeUnmount(() => {
  document.removeEventListener('visibilitychange', handleVisibilityChange)
  if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
  stopCountdown()
})
</script>

<template>
  <div class="space-y-6">
    <!-- 标题 -->
    <div>
      <h2 class="text-2xl font-bold">
        {{ t('cacheMonitoring.title') }}
      </h2>
      <p class="text-sm text-muted-foreground mt-1">
        {{ t('cacheMonitoring.description') }}
      </p>
    </div>

    <!-- 亲和性系统状态 -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <Card class="p-4">
        <div class="text-xs text-muted-foreground">
          {{ t('cacheMonitoring.activeAffinity') }}
        </div>
        <div class="text-2xl font-bold mt-1">
          {{ stats?.affinity_stats?.active_affinities || stats?.affinity_stats?.total_affinities || 0 }}
        </div>
        <div class="text-xs text-muted-foreground mt-1">
          TTL {{ config?.cache_ttl_seconds || 300 }}s
        </div>
      </Card>

      <Card class="p-4">
        <div class="text-xs text-muted-foreground">
          {{ t('cacheMonitoring.providerSwitch') }}
        </div>
        <div
          class="text-2xl font-bold mt-1"
          :class="(stats?.affinity_stats?.provider_switches || 0) > 0 ? 'text-destructive' : ''"
        >
          {{ stats?.affinity_stats?.provider_switches || 0 }}
        </div>
        <div class="text-xs text-muted-foreground mt-1">
          {{ t('cacheMonitoring.keySwitch') }} {{ stats?.affinity_stats?.key_switches || 0 }}
        </div>
      </Card>

      <Card class="p-4">
        <div class="text-xs text-muted-foreground">
          {{ t('cacheMonitoring.invalidated') }}
        </div>
        <div
          class="text-2xl font-bold mt-1"
          :class="(stats?.affinity_stats?.cache_invalidations || 0) > 0 ? 'text-warning' : ''"
        >
          {{ stats?.affinity_stats?.cache_invalidations || 0 }}
        </div>
        <div class="text-xs text-muted-foreground mt-1">
            {{ t('cacheMonitoring.unavailable') }}
        </div>
      </Card>

      <Card class="p-4">
        <div class="text-xs text-muted-foreground flex items-center gap-1">
          {{ t('cacheMonitoring.reservation') }}
          <Badge
            v-if="config?.dynamic_reservation?.enabled"
            variant="outline"
            class="text-[10px] px-1"
          >
            {{ t('cacheMonitoring.dynamic') }}
          </Badge>
        </div>
        <div class="text-2xl font-bold mt-1">
          <template v-if="config?.dynamic_reservation?.enabled">
            {{ (config.dynamic_reservation.config.stable_min_reservation * 100).toFixed(0) }}-{{ (config.dynamic_reservation.config.stable_max_reservation * 100).toFixed(0) }}%
          </template>
          <template v-else>
            {{ config ? (config.cache_reservation_ratio * 100).toFixed(0) : '30' }}%
          </template>
        </div>
        <div class="text-xs text-muted-foreground mt-1">
          {{ stats ? (stats.cache_reservation_ratio * 100).toFixed(0) : '-' }}%
        </div>
      </Card>
    </div>

    <!-- 缓存亲和性列表 -->
    <Card class="overflow-hidden">
      <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <h3 class="text-sm sm:text-base font-semibold shrink-0">
            {{ t('cacheMonitoring.affinityList') }}
          </h3>
          <div class="flex flex-wrap items-center gap-2">
            <div class="relative">
              <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground z-10 pointer-events-none" />
              <Input
                id="cache-affinity-search"
                v-model="tableKeyword"
                :placeholder="t('cacheMonitoring.search')"
                class="w-32 sm:w-48 h-8 text-sm pl-8 pr-8"
              />
              <button
                v-if="tableKeyword"
                type="button"
                class="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground z-10"
                @click="resetAffinitySearch"
              >
                <X class="h-3.5 w-3.5" />
              </button>
            </div>
            <div class="hidden sm:block h-4 w-px bg-border" />
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8 text-muted-foreground/70 hover:text-destructive"
              :disabled="clearingAllAffinity"
              :title="t('cacheMonitoring.clearAll')"
              @click="clearAllCache"
            >
              <Eraser class="h-4 w-4" />
            </Button>
            <RefreshButton
              :loading="loading || listLoading"
              @click="refreshData"
            />
          </div>
        </div>
      </div>

      <Table class="hidden xl:table">
        <TableHeader>
          <TableRow>
            <TableHead class="w-36">
              {{ t('cacheMonitoring.user') }}
            </TableHead>
            <TableHead class="w-28">
              Key
            </TableHead>
            <TableHead class="w-28">
              Provider
            </TableHead>
            <TableHead class="w-40">
              {{ t('cacheMonitoring.model') }}
            </TableHead>
            <TableHead class="w-24">
              {{ t('cacheMonitoring.client') }}
            </TableHead>
            <TableHead class="w-36">
              {{ t('cacheMonitoring.formatKey') }}
            </TableHead>
            <TableHead class="w-20 text-center">
              {{ t('cacheMonitoring.remaining') }}
            </TableHead>
            <TableHead class="w-14 text-center">
              {{ t('cacheMonitoring.count') }}
            </TableHead>
            <TableHead class="w-12 text-right">
              {{ t('cacheMonitoring.actions') }}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody v-if="!listLoading && affinityList.length">
          <TableRow
            v-for="item in paginatedAffinityList"
            :key="`${item.affinity_key}-${item.endpoint_id}-${item.key_id}-${item.global_model_id || item.model_name || 'unknown'}-${item.api_format || 'unknown'}-${item.client_family || 'unknown'}-${item.session_hash || 'legacy'}`"
          >
            <TableCell>
              <div class="flex items-center gap-1.5">
                <Badge
                  v-if="item.is_standalone"
                  variant="outline"
                  class="text-warning border-warning/30 text-[10px] px-1"
                >
                  {{ t('cacheMonitoring.independent') }}
                </Badge>
                <span
                  class="text-sm font-medium truncate max-w-[120px]"
                  :title="affinityUserTitle(item)"
                >{{ affinityUserLabel(item) }}</span>
              </div>
            </TableCell>
            <TableCell>
              <div class="flex items-center gap-1.5">
                <span
                  class="text-sm truncate max-w-[80px]"
                  :title="item.user_api_key_name || item.affinity_key"
                >{{ affinityUserApiKeyLabel(item) }}</span>
                <Badge
                  v-if="item.api_format && item.rate_multipliers?.[item.api_format] && item.rate_multipliers[item.api_format] !== 1.0"
                  variant="outline"
                  class="text-warning border-warning/30 text-[10px] px-2"
                >
                  {{ item.rate_multipliers[item.api_format] }}x
                </Badge>
              </div>
              <div class="text-xs text-muted-foreground font-mono">
                {{ item.user_api_key_prefix || '---' }}
              </div>
            </TableCell>
            <TableCell>
              <div
                class="text-sm truncate max-w-[100px]"
                :title="item.provider_name || undefined"
              >
                {{ item.provider_name || t('cacheMonitoring.unknown') }}
              </div>
            </TableCell>
            <TableCell>
              <div
                class="text-sm truncate max-w-[150px]"
                :title="affinityModelLabel(item)"
              >
                {{ affinityModelLabel(item) }}
              </div>
              <div
                v-if="affinityModelSubtitle(item)"
                class="text-xs text-muted-foreground"
                :title="item.model_name || undefined"
              >
                {{ affinityModelSubtitle(item) }}
              </div>
            </TableCell>
            <TableCell>
              <Badge
                variant="outline"
                class="text-[10px] px-2 font-normal"
              >
                {{ formatClientFamily(item.client_family) }}
              </Badge>
            </TableCell>
            <TableCell>
              <div class="text-sm">
                {{ formatApiFormat(item.api_format) }}
              </div>
              <div
                class="text-xs text-muted-foreground truncate max-w-[130px]"
                :title="providerKeyTitle(item)"
              >
                {{ providerKeyLabel(item) }}
              </div>
            </TableCell>
            <TableCell class="text-center">
              <span class="text-xs">{{ getRemainingTime(item.expire_at) }}</span>
            </TableCell>
            <TableCell class="text-center">
              <span
                class="text-sm"
                :title="item.request_count_known === false ? t('cacheMonitoring.noExact') : undefined"
              >{{ formatAffinityRequestCount(item) }}</span>
            </TableCell>
            <TableCell class="text-right">
              <Button
                size="icon"
                variant="ghost"
                class="h-7 w-7 text-muted-foreground/70 hover:text-destructive"
                :disabled="clearingRowAffinityKey === item.affinity_key"
                :title="t('cacheMonitoring.clear')"
                @click="clearSingleAffinity(item)"
              >
                <Trash2 class="h-3.5 w-3.5" />
              </Button>
            </TableCell>
          </TableRow>
        </TableBody>
        <TableBody v-else>
          <TableRow>
            <TableCell
              colspan="9"
              class="text-center py-6 text-sm text-muted-foreground"
            >
              {{ listLoading ? t('cacheMonitoring.loading') : t('cacheMonitoring.empty') }}
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>

      <!-- 移动端卡片列表 -->
      <div
        v-if="!listLoading && affinityList.length > 0"
        class="xl:hidden divide-y divide-border/40"
      >
        <div
          v-for="item in paginatedAffinityList"
          :key="`m-${item.affinity_key}-${item.endpoint_id}-${item.key_id}-${item.global_model_id || item.model_name || 'unknown'}-${item.api_format || 'unknown'}-${item.client_family || 'unknown'}-${item.session_hash || 'legacy'}`"
          class="p-4 space-y-2"
        >
          <div class="flex items-start justify-between gap-3">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-1.5">
                <Badge
                  v-if="item.is_standalone"
                  variant="outline"
                  class="text-warning border-warning/30 text-[10px] px-1"
                >
                  {{ t('cacheMonitoring.independent') }}
                </Badge>
                <span class="text-sm font-medium truncate">{{ affinityUserLabel(item) }}</span>
              </div>
              <div class="text-xs text-muted-foreground mt-0.5">
                {{ affinityUserApiKeyLabel(item) }} · {{ item.user_api_key_prefix || truncateMiddle(item.affinity_key) }}
              </div>
            </div>
            <Button
              size="icon"
              variant="ghost"
              class="h-7 w-7 text-muted-foreground/70 hover:text-destructive shrink-0"
              :disabled="clearingRowAffinityKey === item.affinity_key"
              @click="clearSingleAffinity(item)"
            >
              <Trash2 class="h-3.5 w-3.5" />
            </Button>
          </div>
          <div class="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <span>{{ item.provider_name || t('cacheMonitoring.unknown') }}</span>
            <span>·</span>
            <span class="truncate max-w-[100px]">{{ affinityModelLabel(item) }}</span>
            <span>·</span>
            <Badge
              variant="outline"
              class="text-[10px] px-1.5 font-normal"
            >
              {{ formatClientFamily(item.client_family) }}
            </Badge>
          </div>
          <div class="flex items-center justify-between text-xs">
            <span class="text-muted-foreground">{{ formatApiFormat(item.api_format) }} · {{ providerKeyLabel(item) }}</span>
            <span>{{ getRemainingTime(item.expire_at) }} · {{ formatAffinityRequestCountUnit(item) }}</span>
          </div>
        </div>
      </div>
      <div
        v-else-if="!listLoading && affinityList.length === 0"
        class="xl:hidden text-center py-6 text-sm text-muted-foreground"
      >
        {{ t('cacheMonitoring.empty') }}
      </div>

      <Pagination
        v-if="affinityList.length > 0"
        :current="currentPage"
        :total="affinityList.length"
        :page-size="pageSize"
        cache-key="cache-monitoring-page-size"
        @update:current="currentPage = $event; handlePageChange()"
        @update:page-size="pageSize = $event"
      />
    </Card>

    <!-- 模型映射缓存管理 -->
    <Card class="overflow-hidden">
      <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <div class="flex items-center gap-3 shrink-0">
            <Database class="h-5 w-5 text-muted-foreground hidden sm:block" />
            <h3 class="text-sm sm:text-base font-semibold">
              {{ t('cacheMonitoring.modelMapping') }}
            </h3>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8 text-muted-foreground/70 hover:text-destructive"
              :title="t('cacheMonitoring.clearMappings')"
              :disabled="clearingModelMapping || !modelMappingStats?.available"
              @click="clearAllModelMappingCache"
            >
              <Eraser class="h-4 w-4" />
            </Button>
            <RefreshButton
              :loading="modelMappingLoading"
              @click="fetchModelMappingStats"
            />
          </div>
        </div>
      </div>

      <!-- 映射缓存表格 -->
      <Table
        v-if="modelMappingStats?.available && modelMappingStats.mappings && modelMappingStats.mappings.length > 0"
        class="hidden md:table"
      >
        <TableHeader>
          <TableRow>
            <TableHead class="w-[25%]">
              {{ t('cacheMonitoring.globalModel') }}
            </TableHead>
            <TableHead class="w-8 text-center" />
            <TableHead class="w-[30%]">
              {{ t('cacheMonitoring.mappedModel') }}
            </TableHead>
            <TableHead class="w-[25%]">
              {{ t('cacheMonitoring.provider') }}
            </TableHead>
            <TableHead class="w-[10%] text-center">
              {{ t('cacheMonitoring.remaining') }}
            </TableHead>
            <TableHead class="w-[5%] text-right">
              {{ t('cacheMonitoring.actions') }}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow
            v-for="mapping in modelMappingStats.mappings"
            :key="mapping.mapping_name"
          >
            <TableCell>
              <div v-if="mapping.global_model_name">
                <div class="text-sm font-medium">
                  {{ mapping.global_model_display_name || mapping.global_model_name }}
                </div>
                <div
                  v-if="mapping.global_model_display_name && mapping.global_model_display_name !== mapping.global_model_name"
                  class="text-xs text-muted-foreground font-mono"
                >
                  {{ mapping.global_model_name }}
                </div>
              </div>
              <span
                v-else
                class="text-sm text-muted-foreground"
              >-</span>
            </TableCell>
            <TableCell class="text-center">
              <ArrowRight class="h-4 w-4 text-muted-foreground" />
            </TableCell>
            <TableCell>
              <span class="text-sm font-mono">{{ mapping.mapping_name }}</span>
            </TableCell>
            <TableCell>
              <div
                v-if="mapping.providers && mapping.providers.length > 0"
                class="flex flex-wrap gap-1"
              >
                <Badge
                  v-for="provider in mapping.providers.slice(0, 3)"
                  :key="provider"
                  variant="outline"
                  class="text-xs"
                >
                  {{ provider }}
                </Badge>
                <Badge
                  v-if="mapping.providers.length > 3"
                  variant="outline"
                  class="text-xs"
                >
                  +{{ mapping.providers.length - 3 }}
                </Badge>
              </div>
              <span
                v-else
                class="text-sm text-muted-foreground"
              >-</span>
            </TableCell>
            <TableCell class="text-center">
              <span class="text-xs text-muted-foreground">{{ formatTTL(mapping.ttl) }}</span>
            </TableCell>
            <TableCell class="text-right">
              <Button
                size="icon"
                variant="ghost"
                class="h-6 w-6 text-muted-foreground/50 hover:text-destructive"
                :disabled="clearingModelName === mapping.mapping_name"
                :title="t('cacheMonitoring.clear')"
                @click="clearModelMappingByName(mapping.mapping_name)"
              >
                <X class="h-3.5 w-3.5" />
              </Button>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>

      <!-- 移动端卡片列表 -->
      <div
        v-if="modelMappingStats?.available && modelMappingStats.mappings && modelMappingStats.mappings.length > 0"
        class="md:hidden divide-y divide-border/40"
      >
        <div
          v-for="mapping in modelMappingStats.mappings"
          :key="`m-${mapping.mapping_name}`"
          class="p-4 space-y-2"
        >
          <div class="flex items-center justify-between gap-2">
            <span class="text-sm font-medium truncate">{{ mapping.global_model_display_name || mapping.global_model_name || '-' }}</span>
            <Button
              size="icon"
              variant="ghost"
              class="h-6 w-6 text-muted-foreground/50 hover:text-destructive shrink-0"
              :disabled="clearingModelName === mapping.mapping_name"
              @click="clearModelMappingByName(mapping.mapping_name)"
            >
              <X class="h-3.5 w-3.5" />
            </Button>
          </div>
          <div class="flex items-center gap-2 text-xs text-muted-foreground">
            <ArrowRight class="h-3.5 w-3.5 shrink-0" />
            <span class="font-mono">{{ mapping.mapping_name }}</span>
          </div>
          <div
            v-if="mapping.providers && mapping.providers.length > 0"
            class="flex flex-wrap gap-1"
          >
            <Badge
              v-for="provider in mapping.providers"
              :key="provider"
              variant="outline"
              class="text-xs"
            >
              {{ provider }}
            </Badge>
          </div>
        </div>
      </div>

      <!-- 未映射条目（NOT_FOUND 等） -->
      <div
        v-if="modelMappingStats?.available && modelMappingStats.unmapped && modelMappingStats.unmapped.length > 0"
        class="px-6 py-4 border-t border-border/40"
      >
        <div class="text-xs text-muted-foreground mb-2">
          {{ t('cacheMonitoring.unmapped') }}
        </div>
        <div class="flex flex-wrap gap-1.5">
          <Badge
            v-for="entry in modelMappingStats.unmapped"
            :key="entry.mapping_name"
            :variant="getUnmappedStatusBadge(entry.status).variant"
            class="text-xs font-mono cursor-pointer"
                        :title="t('cacheMonitoring.statusClearTitle', { status: getUnmappedStatusBadge(entry.status).text })"
            @click="clearModelMappingByName(entry.mapping_name)"
          >
            {{ entry.mapping_name }}
          </Badge>
        </div>
      </div>

      <!-- Provider 模型映射缓存 -->
      <div
        v-if="modelMappingStats?.available && modelMappingStats.provider_model_mappings && modelMappingStats.provider_model_mappings.length > 0"
        class="border-t border-border/40"
      >
        <div class="px-6 py-3 text-xs text-muted-foreground border-b border-border/30 bg-muted/20">
          {{ t('cacheMonitoring.providerMapping') }}
        </div>
        <!-- 桌面端表格 -->
        <Table class="hidden md:table">
          <TableHeader>
            <TableRow>
              <TableHead class="w-[15%]">
                {{ t('cacheMonitoring.provider') }}
              </TableHead>
              <TableHead class="w-[25%]">
                {{ t('cacheMonitoring.requestName') }}
              </TableHead>
              <TableHead class="w-8 text-center" />
              <TableHead class="w-[25%]">
                {{ t('cacheMonitoring.mappedModel') }}
              </TableHead>
              <TableHead class="w-[10%] text-center">
                {{ t('cacheMonitoring.remaining') }}
              </TableHead>
              <TableHead class="w-[10%] text-center">
                {{ t('cacheMonitoring.count') }}
              </TableHead>
              <TableHead class="w-[7%] text-right">
                {{ t('cacheMonitoring.actions') }}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <template
              v-for="(mapping, index) in modelMappingStats.provider_model_mappings"
              :key="index"
            >
              <TableRow
                v-for="(alias, aliasIndex) in (mapping.aliases || [])"
                :key="`${index}-${aliasIndex}`"
              >
                <TableCell>
                  <Badge
                    variant="outline"
                    class="text-xs"
                  >
                    {{ mapping.provider_name }}
                  </Badge>
                </TableCell>
                <TableCell>
                  <span class="text-sm font-mono">{{ alias }}</span>
                </TableCell>
                <TableCell class="text-center">
                  <ArrowRight class="h-4 w-4 text-muted-foreground" />
                </TableCell>
                <TableCell>
                  <span class="text-sm font-mono font-medium">{{ mapping.provider_model_name }}</span>
                </TableCell>
                <TableCell class="text-center">
                  <span class="text-xs text-muted-foreground">{{ formatTTL(mapping.ttl) }}</span>
                </TableCell>
                <TableCell class="text-center">
                  <span class="text-sm">{{ mapping.hit_count || 0 }}</span>
                </TableCell>
                <TableCell class="text-right">
                  <Button
                    size="icon"
                    variant="ghost"
                    class="h-7 w-7 text-muted-foreground/70 hover:text-destructive"
                              :title="t('cacheMonitoring.clear')"
                    @click="clearProviderModelMapping(mapping.provider_id, mapping.global_model_id, `${mapping.provider_name} - ${alias}`)"
                  >
                    <Trash2 class="h-3.5 w-3.5" />
                  </Button>
                </TableCell>
              </TableRow>
            </template>
          </TableBody>
        </Table>
        <!-- 移动端卡片 -->
        <div class="md:hidden divide-y divide-border/40">
          <template
            v-for="(mapping, index) in modelMappingStats.provider_model_mappings"
            :key="`m-pm-${index}`"
          >
            <div
              v-for="(alias, aliasIndex) in (mapping.aliases || [])"
              :key="`m-pm-${index}-${aliasIndex}`"
              class="p-4 space-y-2"
            >
              <div class="flex items-center justify-between">
                <Badge
                  variant="outline"
                  class="text-xs"
                >
                  {{ mapping.provider_name }}
                </Badge>
                <div class="flex items-center gap-2">
                  <span class="text-xs text-muted-foreground">{{ formatTTL(mapping.ttl) }}</span>
                          <span class="text-xs">{{ t('cacheMonitoring.times', { count: mapping.hit_count || 0 }) }}</span>
                  <Button
                    size="icon"
                    variant="ghost"
                    class="h-6 w-6 text-muted-foreground/70 hover:text-destructive"
                              :title="t('cacheMonitoring.clear')"
                    @click="clearProviderModelMapping(mapping.provider_id, mapping.global_model_id, `${mapping.provider_name} - ${alias}`)"
                  >
                    <Trash2 class="h-3 w-3" />
                  </Button>
                </div>
              </div>
              <div class="flex items-center gap-2 text-sm">
                <span class="font-mono">{{ alias }}</span>
                <ArrowRight class="h-3.5 w-3.5 shrink-0 text-muted-foreground/60" />
                <span class="font-mono font-medium">{{ mapping.provider_model_name }}</span>
              </div>
            </div>
          </template>
        </div>
      </div>

      <!-- 无缓存状态 -->
      <div
        v-else-if="modelMappingStats?.available && (!modelMappingStats.mappings || modelMappingStats.mappings.length === 0) && (!modelMappingStats.unmapped || modelMappingStats.unmapped.length === 0) && (!modelMappingStats.provider_model_mappings || modelMappingStats.provider_model_mappings.length === 0)"
        class="px-6 py-8 text-center text-sm text-muted-foreground"
      >
        {{ t('cacheMonitoring.noMapping') }}
      </div>

      <!-- Redis 未启用 -->
      <div
        v-else-if="modelMappingStats && !modelMappingStats.available"
        class="px-6 py-8 text-center text-sm text-muted-foreground"
      >
        {{ modelMappingStats.message || t('cacheMonitoring.redisDisabled') }}
      </div>

      <!-- 加载中 -->
      <div
        v-else-if="modelMappingLoading"
        class="px-6 py-8 text-center text-sm text-muted-foreground"
      >
                      {{ t('cacheMonitoring.loading') }}
      </div>
    </Card>

    <!-- Redis 缓存分类管理 -->
    <Card class="overflow-hidden">
      <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <div class="flex items-center gap-3 shrink-0">
            <HardDrive class="h-5 w-5 text-muted-foreground hidden sm:block" />
            <h3 class="text-sm sm:text-base font-semibold">
              {{ t('cacheMonitoring.redisManagement') }}
            </h3>
            <Badge
              v-if="redisCacheData?.total_keys !== undefined"
              variant="secondary"
            >
                {{ t('cacheMonitoring.keyCount', { count: redisCacheData.total_keys }) }}
            </Badge>
          </div>
          <div class="flex items-center gap-2">
            <RefreshButton
              :loading="redisCacheLoading"
              size="sm"
              :title="t('cacheMonitoring.refreshCategories')"
              @click="fetchRedisCacheCategories"
            />
          </div>
        </div>
      </div>

      <!-- 有数据 -->
      <div v-if="redisCacheData?.available && redisCacheData.categories.length > 0">
        <!-- 有缓存的分类 -->
        <div
          v-if="redisCategoriesWithKeys.length > 0"
          class="divide-y divide-border/40"
        >
          <div
            v-for="cat in redisCategoriesWithKeys"
            :key="cat.key"
            class="flex items-center justify-between px-4 sm:px-6 py-2.5 hover:bg-muted/30 transition-colors"
          >
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium">{{ cat.name }}</span>
                <Badge variant="outline">
                  {{ cat.count }}
                </Badge>
              </div>
              <p class="text-xs text-muted-foreground mt-0.5 truncate">
                {{ cat.description }}
              </p>
            </div>
            <Button
              variant="ghost"
              size="sm"
              class="shrink-0 ml-3 text-destructive hover:text-destructive hover:bg-destructive/10"
              :disabled="clearingCategory === cat.key"
                    :title="t('cacheMonitoring.clearCategory')"
              @click="clearRedisCategory(cat.key, cat.name, cat.count)"
            >
              <Trash2 class="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>

        <!-- 空分类折叠 -->
        <div
          v-if="redisCategoriesEmpty.length > 0"
          class="px-4 sm:px-6 py-3 border-t border-border/40"
        >
          <p class="text-xs text-muted-foreground">
              {{ t('cacheMonitoring.emptyCategories', { count: redisCategoriesEmpty.length, names: redisCategoriesEmpty.map(c => c.name).join(locale === 'en-US' ? ', ' : '、') }) }}
          </p>
        </div>

        <!-- 全部为空 -->
        <div
          v-if="redisCategoriesWithKeys.length === 0"
          class="px-6 py-8 text-center"
        >
          <HardDrive class="h-10 w-10 text-muted-foreground/30 mx-auto mb-2" />
          <p class="text-sm text-muted-foreground">
              {{ t('cacheMonitoring.allCategoriesEmpty') }}
          </p>
        </div>
      </div>

      <!-- Redis 不可用 -->
      <div
        v-else-if="redisCacheData && !redisCacheData.available"
        class="px-6 py-8 text-center text-sm text-muted-foreground"
      >
              {{ redisCacheData.message || t('cacheMonitoring.redisDisabled') }}
      </div>

      <!-- 加载中 -->
      <div
        v-else-if="redisCacheLoading"
        class="px-6 py-8 text-center text-sm text-muted-foreground"
      >
              {{ t('cacheMonitoring.scanningRedis') }}
      </div>
    </Card>

    <!-- TTL 分析区域 -->
    <Card class="overflow-hidden">
      <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <div class="flex items-center gap-3 shrink-0">
            <BarChart3 class="h-5 w-5 text-muted-foreground hidden sm:block" />
            <h3 class="text-sm sm:text-base font-semibold">
              {{ t('cacheMonitoring.ttlAnalysis') }}
            </h3>
            <span class="text-xs text-muted-foreground hidden sm:inline">{{ t('cacheMonitoring.ttlAnalysisHint') }}</span>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <Select
              v-model="analysisHours"
            >
              <SelectTrigger class="w-24 sm:w-28 h-8">
                <SelectValue :placeholder="t('cacheMonitoring.period')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="option in analysisHoursOptions"
                  :key="option.value"
                  :value="option.value"
                >
                  {{ option.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </div>

      <!-- 缓存命中概览 -->
      <div
        v-if="hitAnalysis"
        class="px-6 py-4 border-b border-border/40 bg-muted/30"
      >
        <div class="grid grid-cols-2 md:grid-cols-5 gap-6">
          <div>
            <div class="text-xs text-muted-foreground">
                  {{ t('cacheMonitoring.requestHitRate') }}
            </div>
            <div class="text-2xl font-bold text-success">
              {{ hitAnalysis.request_cache_hit_rate }}%
            </div>
            <div class="text-xs text-muted-foreground">
                {{ t('cacheMonitoring.requestCount', { hit: formatNumber(hitAnalysis.requests_with_cache_hit), total: formatNumber(hitAnalysis.total_requests) }) }}
            </div>
          </div>
          <div>
            <div class="text-xs text-muted-foreground">
                  {{ t('cacheMonitoring.tokenHitRate') }}
            </div>
            <div class="text-2xl font-bold">
              {{ hitAnalysis.token_cache_hit_rate }}%
            </div>
            <div class="text-xs text-muted-foreground">
                {{ t('cacheMonitoring.tokenHits', { count: formatTokens(hitAnalysis.total_cache_read_tokens) }) }}
            </div>
          </div>
          <div>
            <div class="text-xs text-muted-foreground">
                  {{ t('cacheMonitoring.creationCost') }}
            </div>
            <div class="text-2xl font-bold">
              {{ formatCost(hitAnalysis.total_cache_creation_cost_usd) }}
            </div>
            <div class="text-xs text-muted-foreground">
              {{ formatTokens(hitAnalysis.total_cache_creation_tokens) }} tokens
            </div>
          </div>
          <div>
            <div class="text-xs text-muted-foreground">
                  {{ t('cacheMonitoring.readCost') }}
            </div>
            <div class="text-2xl font-bold">
              {{ formatCost(hitAnalysis.total_cache_read_cost_usd) }}
            </div>
            <div class="text-xs text-muted-foreground">
              {{ formatTokens(hitAnalysis.total_cache_read_tokens) }} tokens
            </div>
          </div>
          <div>
            <div class="text-xs text-muted-foreground">
                  {{ t('cacheMonitoring.estimatedSavings') }}
            </div>
            <div class="text-2xl font-bold text-success">
              {{ formatCost(hitAnalysis.estimated_savings_usd) }}
            </div>
          </div>
        </div>
      </div>

      <!-- 用户 TTL 分析表格 -->
      <Table v-if="ttlAnalysis && ttlAnalysis.users.length > 0">
        <TableHeader>
          <TableRow>
            <TableHead class="w-10" />
            <TableHead class="w-[20%]">
                  {{ t('cacheMonitoring.user') }}
            </TableHead>
            <TableHead class="w-[15%] text-center">
                  {{ t('cacheMonitoring.requestTotal') }}
            </TableHead>
            <TableHead class="w-[15%] text-center">
                  {{ t('cacheMonitoring.frequency') }}
            </TableHead>
            <TableHead class="w-[15%] text-center">
                  {{ t('cacheMonitoring.recommendedTtl') }}
            </TableHead>
                <TableHead>{{ t('cacheMonitoring.explanation') }}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <template
            v-for="user in ttlAnalysis.users"
            :key="user.group_id"
          >
            <TableRow
              class="cursor-pointer hover:bg-muted/50"
              @click="toggleUserExpand(user.group_id)"
            >
              <TableCell class="p-2">
                <button class="p-1 hover:bg-muted rounded">
                  <ChevronDown
                    v-if="expandedUserId === user.group_id"
                    class="h-4 w-4 text-muted-foreground"
                  />
                  <ChevronRight
                    v-else
                    class="h-4 w-4 text-muted-foreground"
                  />
                </button>
              </TableCell>
              <TableCell>
                          <span class="text-sm font-medium">{{ user.username || t('cacheMonitoring.unknownUser') }}</span>
              </TableCell>
              <TableCell class="text-center">
                <span class="text-sm font-medium">{{ user.request_count }}</span>
              </TableCell>
              <TableCell class="text-center">
                <span
                  class="text-sm"
                  :class="getFrequencyClass(user.recommended_ttl_minutes)"
                >
                  {{ frequencyLabel(user.recommended_ttl_minutes) }}
                </span>
              </TableCell>
              <TableCell class="text-center">
                <Badge :variant="getTTLBadgeVariant(user.recommended_ttl_minutes)">
                        {{ t('cacheMonitoring.minutes', { count: user.recommended_ttl_minutes }) }}
                </Badge>
              </TableCell>
              <TableCell>
                <span class="text-xs text-muted-foreground">
                  {{ formatIntervalDescription(user) }}
                </span>
              </TableCell>
            </TableRow>
            <!-- 展开行：显示用户散点图 -->
            <TableRow
              v-if="expandedUserId === user.group_id"
              class="bg-muted/30"
            >
              <TableCell
                colspan="6"
                class="p-0"
              >
                <div class="px-6 py-4">
                  <div class="flex items-center justify-between mb-3">
                    <h4 class="text-sm font-medium">
                              {{ t('cacheMonitoring.requestTimeline') }}
                    </h4>
                    <div class="flex items-center gap-3 text-xs text-muted-foreground">
                            <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-green-500" /> {{ t('cacheMonitoring.range0to5') }}</span>
                            <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-blue-500" /> {{ t('cacheMonitoring.range5to15') }}</span>
                            <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-purple-500" /> {{ t('cacheMonitoring.range15to30') }}</span>
                            <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-orange-500" /> {{ t('cacheMonitoring.range30to60') }}</span>
                            <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-red-500" /> {{ t('cacheMonitoring.rangeOver60') }}</span>
                      <span
                        v-if="userTimelineData"
                        class="ml-2"
                          >{{ t('cacheMonitoring.dataPoints', { count: userTimelineData.total_points }) }}</span>
                    </div>
                  </div>
                  <div
                    v-if="userTimelineLoading"
                    class="h-64 flex items-center justify-center"
                  >
                          <span class="text-sm text-muted-foreground">{{ t('cacheMonitoring.loading') }}</span>
                  </div>
                  <div
                    v-else-if="userTimelineData && userTimelineData.points.length > 0"
                    class="h-64"
                  >
                    <ScatterChart :data="userTimelineChartData" />
                  </div>
                  <div
                    v-else
                    class="h-64 flex items-center justify-center"
                  >
                          <span class="text-sm text-muted-foreground">{{ t('cacheMonitoring.noData') }}</span>
                  </div>
                </div>
              </TableCell>
            </TableRow>
          </template>
        </TableBody>
      </Table>

      <!-- 分析完成但无数据 -->
      <div
        v-else-if="ttlAnalysis && ttlAnalysis.users.length === 0"
        class="px-6 py-12 text-center"
      >
        <BarChart3 class="h-12 w-12 text-muted-foreground/50 mx-auto mb-3" />
        <p class="text-sm text-muted-foreground">
                {{ t('cacheMonitoring.noUserData') }}
        </p>
        <p class="text-xs text-muted-foreground mt-1">
                {{ t('cacheMonitoring.adjustAnalysis') }}
        </p>
      </div>

      <!-- 加载中 -->
      <div
        v-else-if="ttlAnalysisLoading"
        class="px-6 py-12 text-center"
      >
        <p class="text-sm text-muted-foreground">
            {{ t('cacheMonitoring.analyzing') }}
        </p>
      </div>
    </Card>
  </div>
</template>
