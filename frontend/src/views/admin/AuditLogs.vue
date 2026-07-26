<template>
  <div class="space-y-6 pb-8">
    <!-- 审计日志列表 -->
    <Card
      variant="default"
      class="overflow-hidden"
    >
      <!-- 标题和操作栏 -->
      <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <div class="shrink-0">
            <h3 class="text-sm sm:text-base font-semibold">
              {{ t('auditLogs.title') }}
            </h3>
            <p class="text-xs text-muted-foreground mt-0.5">
              {{ t('auditLogs.description') }}
            </p>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <!-- 搜索框 -->
            <div class="relative">
              <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground z-10 pointer-events-none" />
              <Input
                id="audit-logs-search"
                v-model="searchQuery"
                :placeholder="t('auditLogs.searchUser')"
                class="w-32 sm:w-64 h-8 text-sm pl-8"
                @input="handleSearchChange"
              />
            </div>
            <!-- 分隔线 -->
            <div class="hidden sm:block h-4 w-px bg-border" />
            <!-- 事件类型筛选 -->
            <div class="xl:hidden">
              <Select
                v-model="filters.eventType"
                @update:model-value="handleEventTypeChange"
              >
                <SelectTrigger class="w-24 sm:w-40 h-8 border-border/60">
                  <SelectValue :placeholder="t('auditLogs.allTypes')" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem
                    v-for="option in auditEventTypeFilterOptions"
                    :key="option.value"
                    :value="option.value"
                  >
                    {{ option.label }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            <!-- 时间范围筛选 -->
            <div class="xl:hidden">
              <Select
                v-model="filtersDaysString"
                @update:model-value="handleDaysChange"
              >
                <SelectTrigger class="w-20 sm:w-28 h-8 border-border/60">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem
                    v-for="option in auditDaysFilterOptions"
                    :key="option.value"
                    :value="option.value"
                  >
                    {{ option.label }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            <!-- 重置筛选 -->
            <Button
              v-if="hasActiveFilters"
              variant="ghost"
              size="icon"
              class="h-8 w-8"
              :title="t('auditLogs.reset')"
              @click="handleResetFilters"
            >
              <FilterX class="w-3.5 h-3.5" />
            </Button>
            <div class="hidden sm:block h-4 w-px bg-border" />
            <!-- 导出按钮 -->
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8"
              :title="t('auditLogs.export')"
              @click="exportLogs"
            >
              <Download class="w-3.5 h-3.5" />
            </Button>
            <!-- 刷新按钮 -->
            <RefreshButton
              :loading="loading"
              @click="refreshLogs"
            />
          </div>
        </div>
      </div>

      <div
        v-if="loading"
        class="flex items-center justify-center py-12"
      >
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
      </div>

      <div
        v-else-if="logs.length === 0"
        class="text-center py-12 text-muted-foreground"
      >
        {{ t('auditLogs.empty') }}
      </div>

      <div v-else>
        <Table class="hidden xl:table">
          <TableHeader>
            <TableRow class="border-b border-border/60 hover:bg-transparent">
              <SortableTableHead
                class="h-12 font-semibold"
                column-key="created_at"
                :sortable="false"
                :filter-active="filters.days !== 7"
                :filter-title="t('auditLogs.timeRange')"
                filter-content-class="w-32 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
              >
                {{ t('auditLogs.time') }}
                <template #filter="{ close }">
                  <TableFilterMenu
                    :model-value="filtersDaysString"
                    :options="auditDaysFilterOptions"
                    @update:model-value="handleDaysChange"
                    @select="close"
                  />
                </template>
              </SortableTableHead>
              <TableHead class="h-12 font-semibold">
                {{ t('auditLogs.user') }}
              </TableHead>
              <SortableTableHead
                class="h-12 font-semibold"
                column-key="event_type"
                :sortable="false"
                :filter-active="filters.eventType !== '__all__'"
                :filter-title="t('auditLogs.eventTypeFilter')"
                filter-content-class="w-48 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
              >
                {{ t('auditLogs.eventType') }}
                <template #filter="{ close }">
                  <TableFilterMenu
                    :model-value="filters.eventType"
                    :options="auditEventTypeFilterOptions"
                    @update:model-value="handleEventTypeChange"
                    @select="close"
                  />
                </template>
              </SortableTableHead>
              <TableHead class="h-12 font-semibold">
                {{ t('auditLogs.descriptionCol') }}
              </TableHead>
              <TableHead class="h-12 font-semibold">
                {{ t('auditLogs.ip') }}
              </TableHead>
              <TableHead class="h-12 font-semibold">
                {{ t('auditLogs.status') }}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow
              v-for="entry in logs"
              :key="entry.id"
              class="cursor-pointer border-b border-border/40 hover:bg-muted/30 transition-colors"
              @mousedown="handleMouseDown"
              @click="handleRowClick($event, entry)"
            >
              <TableCell class="text-xs py-4">
                {{ formatDateTime(entry.created_at) }}
              </TableCell>

              <TableCell class="py-4">
                <div
                  v-if="entry.user_id"
                  class="flex flex-col"
                >
                  <span class="text-sm font-medium">
                    {{ entry.user_email || t('auditLogs.userFallback', { id: entry.user_id }) }}
                  </span>
                  <span
                    v-if="entry.user_username"
                    class="text-xs text-muted-foreground"
                  >
                    {{ entry.user_username }}
                  </span>
                </div>
                <span
                  v-else
                  class="text-muted-foreground italic"
                >{{ t('auditLogs.system') }}</span>
              </TableCell>

              <TableCell class="py-4">
                <Badge :variant="getEventTypeBadgeVariant(entry.event_type)">
                  <component
                    :is="getEventTypeIcon(entry.event_type)"
                    class="h-3 w-3 mr-1"
                  />
                  {{ getEventTypeLabel(entry.event_type) }}
                </Badge>
              </TableCell>

              <TableCell
                class="max-w-xs truncate py-4"
                :title="entry.description"
              >
                {{ entry.description || t('auditLogs.noDescription') }}
              </TableCell>

              <TableCell class="py-4">
                <span
                  v-if="entry.ip_address"
                  class="flex items-center text-sm"
                >
                  <Globe class="h-3 w-3 mr-1 text-muted-foreground" />
                  {{ entry.ip_address }}
                </span>
                <span v-else>-</span>
              </TableCell>

              <TableCell class="py-4">
                <Badge
                  v-if="entry.status_code"
                  :variant="getStatusCodeVariant(entry.status_code)"
                >
                  {{ entry.status_code }}
                </Badge>
                <span v-else>-</span>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>

        <!-- 移动端卡片列表 -->
        <div
          v-if="logs.length > 0"
          class="xl:hidden divide-y divide-border/40"
        >
          <div
            v-for="logItem in logs"
            :key="logItem.id"
            class="p-4 space-y-2 hover:bg-muted/30 cursor-pointer transition-colors"
            @click="showLogDetail(logItem)"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="flex-1 min-w-0">
                <Badge :variant="getEventTypeBadgeVariant(logItem.event_type)">
                  <component
                    :is="getEventTypeIcon(logItem.event_type)"
                    class="h-3 w-3 mr-1"
                  />
                  {{ getEventTypeLabel(logItem.event_type) }}
                </Badge>
                <div class="text-xs text-muted-foreground mt-1.5">
                  {{ formatDateTime(logItem.created_at) }}
                </div>
              </div>
              <Badge
                v-if="logItem.status_code"
                :variant="getStatusCodeVariant(logItem.status_code)"
                class="shrink-0"
              >
                {{ logItem.status_code }}
              </Badge>
            </div>
            <div
              v-if="logItem.user_id"
              class="text-sm"
            >
              {{ logItem.user_email || t('auditLogs.userFallback', { id: logItem.user_id }) }}
            </div>
            <div
              class="text-xs text-muted-foreground truncate"
              :title="logItem.description"
            >
              {{ logItem.description || t('auditLogs.noDescription') }}
            </div>
            <div
              v-if="logItem.ip_address"
              class="flex items-center text-xs text-muted-foreground"
            >
              <Globe class="h-3 w-3 mr-1" />
              {{ logItem.ip_address }}
            </div>
          </div>
        </div>

        <!-- 分页控件 -->
        <Pagination
          :current="currentPage"
          :total="totalRecords"
          :page-size="pageSize"
          :page-size-options="[10, 20, 50, 100]"
          cache-key="audit-logs-page-size"
          @update:current="handlePageChange"
          @update:page-size="pageSize = $event; currentPage = 1; loadLogs()"
        />
      </div>
    </Card>

    <!-- 详情对话框 (使用shadcn Dialog组件) -->
    <div
      v-if="selectedLog"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click="closeLogDetail"
    >
      <Card
        class="max-w-2xl w-full mx-4 max-h-[80vh] overflow-y-auto"
        @click.stop
      >
        <div class="p-6">
          <div class="flex justify-between items-center mb-4">
            <h3 class="text-lg font-medium">
              {{ t('auditLogs.detail') }}
            </h3>
            <Button
              variant="ghost"
              size="sm"
              @click="closeLogDetail"
            >
              <X class="h-4 w-4" />
            </Button>
          </div>

          <div class="space-y-4">
            <div>
              <Label>{{ t('auditLogs.eventType') }}</Label>
              <p class="mt-1 text-sm">
                {{ getEventTypeLabel(selectedLog.event_type) }}
              </p>
            </div>

            <Separator />

            <div>
              <Label>{{ t('auditLogs.descriptionCol') }}</Label>
              <p class="mt-1 text-sm">
                {{ selectedLog.description || t('auditLogs.noDescription') }}
              </p>
            </div>

            <div>
              <Label>{{ t('auditLogs.time') }}</Label>
              <p class="mt-1 text-sm">
                {{ formatDateTime(selectedLog.created_at) }}
              </p>
            </div>

            <div v-if="selectedLog.user_id">
              <Label>{{ t('auditLogs.userInfo') }}</Label>
              <div class="mt-1 text-sm">
                <p class="font-medium">
                  {{ selectedLog.user_email || t('auditLogs.userFallback', { id: selectedLog.user_id }) }}
                </p>
                <p
                  v-if="selectedLog.user_username"
                  class="text-muted-foreground"
                >
                  {{ selectedLog.user_username }}
                </p>
                <p class="text-xs text-muted-foreground">
                  ID: {{ selectedLog.user_id }}
                </p>
              </div>
            </div>

            <div v-if="selectedLog.ip_address">
              <Label>{{ t('auditLogs.ip') }}</Label>
              <p class="mt-1 text-sm">
                {{ selectedLog.ip_address }}
              </p>
            </div>

            <div v-if="selectedLog.status_code">
              <Label>{{ t('auditLogs.statusCode') }}</Label>
              <p class="mt-1 text-sm">
                {{ selectedLog.status_code }}
              </p>
            </div>

            <div v-if="selectedLog.error_message">
              <Label>{{ t('auditLogs.errorMessage') }}</Label>
              <p class="mt-1 text-sm text-destructive">
                {{ selectedLog.error_message }}
              </p>
            </div>

            <div v-if="selectedLog.metadata">
              <Label>{{ t('auditLogs.metadata') }}</Label>
              <pre class="mt-1 text-sm bg-muted p-3 rounded-md overflow-x-auto">{{ JSON.stringify(selectedLog.metadata, null, 2) }}</pre>
            </div>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Card,
  Button,
  Badge,
  Separator,
  Label,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  SortableTableHead,
  TableFilterMenu,
  TableCell,
  Input,
  Pagination,
  RefreshButton
} from '@/components/ui'
import { auditApi } from '@/api/audit'
import {
  Download,
  Shield,
  Key,
  Activity,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Globe,
  X,
  User,
  Settings,
  Search,
  FilterX
} from 'lucide-vue-next'

interface AuditLog {
  id: string
  event_type: string
  user_id?: number
  user_email?: string
  user_username?: string
  description: string
  ip_address?: string
  status_code?: number
  error_message?: string
  metadata?: Record<string, unknown>
  created_at: string
}

const loading = ref(false)
const { t } = useI18n()
const logs = ref<AuditLog[]>([])
const selectedLog = ref<AuditLog | null>(null)
let logsRequestId = 0

// 搜索查询
const searchQuery = ref('')


const filters = ref({
  username: '',
  eventType: '__all__',
  days: 7,
  limit: 50
})

const filtersDaysString = ref('7')
const auditEventTypeFilterOptions = computed(() => [
  { value: '__all__', label: t('auditLogs.allTypes') },
  { value: 'login_success', label: t('auditLogs.loginSuccess') },
  { value: 'login_failed', label: t('auditLogs.loginFailed') },
  { value: 'logout', label: t('auditLogs.logout') },
  { value: 'api_key_created', label: t('auditLogs.apiKeyCreated') },
  { value: 'api_key_deleted', label: t('auditLogs.apiKeyDeleted') },
  { value: 'request_success', label: t('auditLogs.requestSuccess') },
  { value: 'request_failed', label: t('auditLogs.requestFailed') },
  { value: 'user_created', label: t('auditLogs.userCreated') },
  { value: 'user_updated', label: t('auditLogs.userUpdated') },
  { value: 'user_deleted', label: t('auditLogs.userDeleted') },
])
const auditDaysFilterOptions = computed(() => [
  { value: '1', label: t('auditLogs.days', { count: 1 }) },
  { value: '7', label: t('auditLogs.days', { count: 7 }) },
  { value: '30', label: t('auditLogs.days', { count: 30 }) },
  { value: '90', label: t('auditLogs.days', { count: 90 }) },
])

const currentPage = ref(1)
const pageSize = ref(20)
const totalRecords = ref(0)

let loadTimeout: number | null = null
const debouncedLoadLogs = () => {
  if (loadTimeout !== null) clearTimeout(loadTimeout)
  loadTimeout = window.setTimeout(resetAndLoad, 500)
}

const hasActiveFilters = computed(() => {
  return searchQuery.value !== '' ||
    filters.value.eventType !== '__all__' ||
    filters.value.days !== 7
})

async function loadLogs() {
  const requestId = ++logsRequestId
  loading.value = true
  try {
    const offset = (currentPage.value - 1) * pageSize.value

    const filterParams = {
      username: filters.value.username || undefined,
      event_type: (filters.value.eventType !== '__all__' ? filters.value.eventType : undefined),
      days: filters.value.days,
      limit: pageSize.value,
      offset
    }

    const data = await auditApi.getAuditLogs(filterParams)
    if (requestId !== logsRequestId) return
    logs.value = data.items || []
    totalRecords.value = data.meta?.total ?? logs.value.length
  } catch (error) {
    if (requestId !== logsRequestId) return
    log.error('获取审计日志失败:', error)
    logs.value = []
    totalRecords.value = 0
  } finally {
    if (requestId === logsRequestId) {
      loading.value = false
    }
  }
}

function refreshLogs() {
  loadLogs()
}

// 搜索变化处理
function handleSearchChange() {
  filters.value.username = searchQuery.value
  debouncedLoadLogs()
}

// 重置筛选条件
function handleResetFilters() {
  searchQuery.value = ''
  filters.value.username = ''
  filters.value.eventType = '__all__'
  filters.value.days = 7
  filtersDaysString.value = '7'
  currentPage.value = 1
  loadLogs()
}

// 页码变化处理
function handlePageChange(page: number) {
  currentPage.value = page
  loadLogs()
}

function handleEventTypeChange(value: string) {
  filters.value.eventType = value
  resetAndLoad()
}

function handleDaysChange(value: string) {
  filtersDaysString.value = value
  filters.value.days = parseInt(value)
  resetAndLoad()
}

function resetAndLoad() {
  currentPage.value = 1
  loadLogs()
}

async function exportLogs() {
  try {
    let allLogs: AuditLog[] = []
    let offset = 0
    const batchSize = 500
    let hasMore = true

    while (hasMore) {
      const data = await auditApi.getAuditLogs({
        username: filters.value.username || undefined,
        event_type: filters.value.eventType !== '__all__' ? filters.value.eventType : undefined,
        days: filters.value.days,
        limit: batchSize,
        offset
      })

      const batch = data.items || []
      allLogs = allLogs.concat(batch)

      if (batch.length < batchSize) {
        hasMore = false
      } else {
        offset += batch.length
        hasMore = offset < (data.meta?.total ?? offset)
      }
    }

    const csvContent = [
      [t('auditLogs.time'), t('auditLogs.email'), t('auditLogs.username'), t('auditLogs.userId'), t('auditLogs.eventType'), t('auditLogs.descriptionCol'), t('auditLogs.ip'), t('auditLogs.statusCode'), t('auditLogs.errorMessage')].join(','),
      ...allLogs.map((log: AuditLog) => [
        log.created_at,
        `"${log.user_email || ''}"`,
        `"${log.user_username || ''}"`,
        log.user_id || '',
        log.event_type,
        `"${log.description || ''}"`,
        log.ip_address || '',
        log.status_code || '',
        `"${log.error_message || ''}"`
      ].join(','))
    ].join('\n')

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `audit-logs-${new Date().toISOString().split('T')[0]}.csv`
    link.click()
  } catch (error) {
    log.error('导出失败:', error)
  }
}

// 使用复用的行点击逻辑
import { useRowClick } from '@/composables/useRowClick'
import { log } from '@/utils/logger'
const { handleMouseDown, shouldTriggerRowClick } = useRowClick()

function handleRowClick(event: MouseEvent, log: AuditLog) {
  if (!shouldTriggerRowClick(event)) return
  showLogDetail(log)
}

function showLogDetail(log: AuditLog) {
  selectedLog.value = log
}

function closeLogDetail() {
  selectedLog.value = null
}

function getEventTypeLabel(eventType: string): string {
  const labels: Record<string, string> = {
    'login_success': t('auditLogs.loginSuccess'), 'login_failed': t('auditLogs.loginFailed'), 'logout': t('auditLogs.logout'),
    'api_key_created': t('auditLogs.apiKeyCreated'), 'api_key_deleted': t('auditLogs.apiKeyDeleted'), 'api_key_used': t('auditLogs.apiKeyUsed'),
    'request_success': t('auditLogs.requestSuccess'), 'request_failed': t('auditLogs.requestFailed'), 'request_rate_limited': t('auditLogs.requestRateLimited'), 'request_quota_exceeded': t('auditLogs.requestQuotaExceeded'),
    'user_created': t('auditLogs.userCreated'), 'user_updated': t('auditLogs.userUpdated'), 'user_deleted': t('auditLogs.userDeleted'),
    'provider_added': t('auditLogs.providerAdded'), 'provider_updated': t('auditLogs.providerUpdated'), 'provider_removed': t('auditLogs.providerRemoved'),
    'suspicious_activity': t('auditLogs.suspiciousActivity'), 'unauthorized_access': t('auditLogs.unauthorizedAccess'), 'data_export': t('auditLogs.dataExport'), 'config_changed': t('auditLogs.configChanged')
  }
  return labels[eventType] || eventType
}

function getEventTypeIcon(eventType: string) {
  const icons: Record<string, unknown> = {
    'login_success': CheckCircle,
    'login_failed': XCircle,
    'logout': User,
    'api_key_created': Key,
    'api_key_deleted': Key,
    'api_key_used': Key,
    'request_success': CheckCircle,
    'request_failed': XCircle,
    'request_rate_limited': AlertTriangle,
    'request_quota_exceeded': AlertTriangle,
    'user_created': User,
    'user_updated': User,
    'user_deleted': User,
    'provider_added': Settings,
    'provider_updated': Settings,
    'provider_removed': Settings,
    'suspicious_activity': Shield,
    'unauthorized_access': Shield,
    'data_export': Activity,
    'config_changed': Settings
  }
  return icons[eventType] || Activity
}

function getEventTypeBadgeVariant(eventType: string): 'default' | 'success' | 'destructive' | 'warning' | 'secondary' {
  if (eventType.includes('success') || eventType.includes('created')) return 'success'
  if (eventType.includes('failed') || eventType.includes('deleted') || eventType.includes('unauthorized')) return 'destructive'
  if (eventType.includes('limited') || eventType.includes('exceeded') || eventType.includes('suspicious')) return 'warning'
  return 'secondary'
}

function getStatusCodeVariant(statusCode: number): 'default' | 'success' | 'destructive' | 'warning' {
  if (statusCode < 300) return 'success'
  if (statusCode < 400) return 'default'
  if (statusCode < 500) return 'warning'
  return 'destructive'
}

function formatDateTime(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

onMounted(() => {
  loadLogs()
})

onBeforeUnmount(() => {
  if (loadTimeout !== null) {
    clearTimeout(loadTimeout)
    loadTimeout = null
  }
  logsRequestId += 1
})
</script>
