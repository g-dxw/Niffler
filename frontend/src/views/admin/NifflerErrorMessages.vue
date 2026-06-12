<template>
  <PageContainer>
    <PageHeader
      title="错误提示"
      description="配置平台和上游错误返回给用户看的文案。"
      :icon="AlertTriangle"
    >
      <template #actions>
        <Button
          variant="outline"
          class="admin-filter-action"
          :disabled="pageLoading"
          @click="refreshAll"
        >
          <RefreshCw
            class="mr-2 h-4 w-4"
            :class="{ 'animate-spin': pageLoading }"
          />
          刷新
        </Button>
        <Button
          class="admin-entry-action"
          @click="errorReturnSettingDialogOpen = true"
        >
          <Plus class="mr-2 h-4 w-4" />
          新增规则
        </Button>
      </template>
    </PageHeader>

    <div class="mt-6 grid gap-5 xl:grid-cols-[320px_minmax(0,1fr)]">
      <Card class="p-4">
        <h2 class="text-sm font-semibold">
          规则类型
        </h2>
        <div class="mt-3 space-y-2">
          <button
            v-for="item in scopeFilters"
            :key="item.value"
            type="button"
            class="admin-entry-row w-full rounded-lg border px-3 py-3 text-left transition-colors"
            :class="scopeFilter === item.value ? 'border-primary/50 bg-primary/10' : 'border-border/70 hover:bg-muted/40'"
            @click="scopeFilter = item.value"
          >
            <div class="text-sm font-medium">
              {{ item.label }}
            </div>
            <p class="mt-1 text-xs text-muted-foreground">
              {{ item.description }}
            </p>
          </button>
        </div>
      </Card>

      <Card class="overflow-hidden">
        <div class="flex flex-col gap-3 border-b border-border/70 px-5 py-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 class="text-sm font-semibold">
              规则
            </h2>
            <p class="mt-1 text-xs text-muted-foreground">
              命中规则后，只改用户看到的错误说明；管理员仍能查看原始错误。
            </p>
          </div>
          <Badge variant="secondary">
            {{ filteredErrorReturnSettings.length }}
          </Badge>
        </div>

        <p
          v-if="errorReturnSettingError"
          class="border-b border-destructive/20 bg-destructive/5 px-5 py-3 text-sm text-destructive"
        >
          {{ errorReturnSettingError }}
        </p>

        <div
          v-if="errorReturnSettingLoading && errorReturnSettings.length === 0"
          class="flex items-center justify-center py-16 text-sm text-muted-foreground"
        >
          <Loader2 class="mr-2 h-5 w-5 animate-spin" />
          正在读取规则
        </div>

        <div
          v-else-if="filteredErrorReturnSettings.length === 0"
          class="py-16 text-center"
        >
          <AlertTriangle class="mx-auto h-10 w-10 text-muted-foreground/50" />
          <p class="mt-3 font-medium">
            还没有规则
          </p>
          <p class="mt-1 text-sm text-muted-foreground">
            可以先新增常见余额、模型权限、上游风控或广告替换规则。
          </p>
          <Button
            class="admin-entry-action mt-4"
            @click="errorReturnSettingDialogOpen = true"
          >
            新增规则
          </Button>
        </div>

        <Table
          v-else
          class="w-full min-w-[1100px] table-fixed"
        >
          <colgroup>
            <col :style="{ width: errorRuleColumnWidths.scope }">
            <col :style="{ width: errorRuleColumnWidths.match }">
            <col :style="{ width: errorRuleColumnWidths.response }">
            <col :style="{ width: errorRuleColumnWidths.message }">
            <col :style="{ width: errorRuleColumnWidths.protection }">
            <col :style="{ width: errorRuleColumnWidths.status }">
          </colgroup>
          <TableHeader>
            <TableRow>
              <SortableTableHead :sortable="false" resize-column-key="scope" :resizable="true" @resize-start="handleErrorRuleColumnResizeStart">
                范围
              </SortableTableHead>
              <SortableTableHead :sortable="false" resize-column-key="match" :resizable="true" @resize-start="handleErrorRuleColumnResizeStart">
                匹配
              </SortableTableHead>
              <SortableTableHead :sortable="false" resize-column-key="response" :resizable="true" @resize-start="handleErrorRuleColumnResizeStart">
                返回
              </SortableTableHead>
              <SortableTableHead :sortable="false" resize-column-key="message" :resizable="true" @resize-start="handleErrorRuleColumnResizeStart">
                用户文案
              </SortableTableHead>
              <SortableTableHead :sortable="false" resize-column-key="protection" :resizable="true" @resize-start="handleErrorRuleColumnResizeStart">
                账号保护
              </SortableTableHead>
              <SortableTableHead :sortable="false" resize-column-key="status" :resizable="true" @resize-start="handleErrorRuleColumnResizeStart">
                状态
              </SortableTableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow
              v-for="rule in filteredErrorReturnSettings"
              :key="rule.id"
            >
              <TableCell>
                <div class="font-medium">
                  {{ errorScopeLabel(rule.scope) }}
                </div>
                <div class="mt-1 text-xs text-muted-foreground">
                  {{ upstreamServiceLabel(rule.upstream_service_id) }}
                </div>
              </TableCell>
              <TableCell>
                <div>{{ matchLabel(rule) }}</div>
                <div
                  v-if="rule.handling_step"
                  class="mt-1 text-xs text-muted-foreground"
                >
                  {{ handlingStepLabel(rule.handling_step) }}
                </div>
              </TableCell>
              <TableCell>
                {{ responseModeLabel(rule.response_mode) }}
              </TableCell>
              <TableCell>
                <div
                  class="whitespace-pre-wrap break-words"
                  :title="rule.user_message"
                >
                  {{ rule.user_message }}
                </div>
              </TableCell>
              <TableCell>
                <div>{{ protectionActionLabel(rule.account_protection_action) }}</div>
                <div
                  v-if="rule.pause_duration"
                  class="mt-1 text-xs text-muted-foreground"
                >
                  {{ pauseDurationLabel(rule.pause_duration) }}
                </div>
              </TableCell>
              <TableCell>
                <Badge :variant="rule.is_active ? 'outline' : 'secondary'">
                  {{ rule.is_active ? '启用' : '停用' }}
                </Badge>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </Card>
    </div>

    <Dialog
      v-model="errorReturnSettingDialogOpen"
      size="2xl"
      title="新增规则"
      description="状态码和关键词可以只填一个；上游风控关键词可以触发账号保护记录。"
      :icon="AlertTriangle"
    >
      <form
        class="space-y-5"
        @submit.prevent="submitErrorReturnSetting"
      >
        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-2">
            <Label for="error-scope">范围</Label>
            <Select v-model="errorReturnSettingForm.scope">
              <SelectTrigger id="error-scope">
                <SelectValue placeholder="选择范围" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="platform">
                  平台错误
                </SelectItem>
                <SelectItem value="upstream">
                  上游错误
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div
            v-if="errorReturnSettingForm.scope === 'upstream'"
            class="space-y-2"
          >
            <Label for="error-upstream">上游</Label>
            <Select v-model="errorReturnSettingForm.upstream_service_id">
              <SelectTrigger id="error-upstream">
                <SelectValue placeholder="选择上游" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__all__">
                  全部上游
                </SelectItem>
                <SelectItem
                  v-for="service in services"
                  :key="service.id"
                  :value="service.id"
                >
                  {{ service.display_name }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div
            v-if="errorReturnSettingForm.scope === 'upstream'"
            class="space-y-2"
          >
            <Label for="error-step">处理类型</Label>
            <Select v-model="errorReturnSettingForm.handling_step">
              <SelectTrigger id="error-step">
                <SelectValue placeholder="选择类型" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="risk_keyword">
                  风控关键词
                </SelectItem>
                <SelectItem value="contact_or_marketing_replacement">
                  广告替换
                </SelectItem>
                <SelectItem value="status_code_message">
                  状态码文案
                </SelectItem>
                <SelectItem value="default_upstream_message">
                  默认上游文案
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-2">
            <Label for="error-status-code">状态码</Label>
            <Input
              id="error-status-code"
              v-model="errorReturnSettingForm.match_status_code"
              type="number"
              min="100"
              max="599"
              step="1"
              placeholder="例如 403"
            />
          </div>

          <div class="space-y-2 sm:col-span-2">
            <Label for="error-match-text">
              {{ errorReturnSettingForm.scope === 'platform' ? '错误代码' : '关键词' }}
            </Label>
            <Input
              id="error-match-text"
              v-model="errorReturnSettingForm.match_text"
              :placeholder="errorReturnSettingForm.scope === 'platform' ? '例如 insufficient_balance' : '例如 abuse 或上游客服信息'"
            />
          </div>

          <div class="space-y-2">
            <Label for="error-response-mode">返回方式</Label>
            <Select v-model="errorReturnSettingForm.response_mode">
              <SelectTrigger id="error-response-mode">
                <SelectValue placeholder="选择方式" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="replace">
                  替换
                </SelectItem>
                <SelectItem value="append">
                  追加
                </SelectItem>
                <SelectItem value="redact">
                  脱敏
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div
            v-if="errorReturnSettingForm.scope === 'upstream'"
            class="space-y-2"
          >
            <Label for="error-protection">账号保护</Label>
            <Select v-model="errorReturnSettingForm.account_protection_action">
              <SelectTrigger id="error-protection">
                <SelectValue placeholder="选择动作" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="record_only">
                  只记录
                </SelectItem>
                <SelectItem value="pause_scheduling">
                  暂停调度
                </SelectItem>
                <SelectItem value="disable_account">
                  人工处理
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div
            v-if="errorReturnSettingForm.scope === 'upstream' && errorReturnSettingForm.account_protection_action === 'pause_scheduling'"
            class="space-y-2"
          >
            <Label for="error-pause-duration">暂停时长</Label>
            <Select v-model="errorReturnSettingForm.pause_duration">
              <SelectTrigger id="error-pause-duration">
                <SelectValue placeholder="选择时长" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="ten_minutes">
                  10 分钟
                </SelectItem>
                <SelectItem value="one_hour">
                  1 小时
                </SelectItem>
                <SelectItem value="twenty_four_hours">
                  24 小时
                </SelectItem>
                <SelectItem value="manual_restore">
                  手动恢复
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="flex items-center gap-3 pt-7">
            <Switch
              id="error-active"
              v-model="errorReturnSettingForm.is_active"
            />
            <Label for="error-active">启用</Label>
          </div>
        </div>

        <div class="space-y-2">
          <Label for="error-user-message">用户文案</Label>
          <Textarea
            id="error-user-message"
            v-model="errorReturnSettingForm.user_message"
            rows="4"
            placeholder="例如：请求内容触发上游安全限制，请调整任务后重试。"
            required
          />
        </div>
      </form>

      <template #footer>
        <Button
          class="admin-entry-action"
          type="submit"
          :disabled="savingErrorReturnSetting"
          @click="submitErrorReturnSetting"
        >
          {{ savingErrorReturnSetting ? '保存中...' : '保存规则' }}
        </Button>
        <Button
          class="admin-entry-action"
          type="button"
          variant="outline"
          :disabled="savingErrorReturnSetting"
          @click="errorReturnSettingDialogOpen = false"
        >
          取消
        </Button>
      </template>
    </Dialog>
  </PageContainer>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import {
  AlertTriangle,
  Loader2,
  Plus,
  RefreshCw,
} from 'lucide-vue-next'
import { PageContainer, PageHeader } from '@/components/layout'
import {
  Badge,
  Button,
  Card,
  Dialog,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
  SortableTableHead,
  Textarea,
} from '@/components/ui'
import {
  createNifflerErrorReturnSetting,
  listNifflerErrorReturnSettings,
  listNifflerUpstreamServices,
  type CreateNifflerErrorReturnSettingPayload,
  type NifflerAccountProtectionAction,
  type NifflerErrorResponseScope,
  type NifflerErrorReturnSetting,
  type NifflerPauseDuration,
  type NifflerUpstreamErrorHandlingStep,
  type NifflerUpstreamService,
  type NifflerUserResponseMode,
} from '@/api/niffler-core'
import { useToast } from '@/composables/useToast'
import { useResizableTableColumns, type ResizableTableColumn } from '@/composables/useResizableTableColumns'
import { extractErrorMessage } from '@/utils/error'

type ScopeFilter = 'all' | NifflerErrorResponseScope

type ErrorReturnSettingForm = {
  scope: NifflerErrorResponseScope
  upstream_service_id: string
  match_status_code: number | string | null
  match_text: string
  handling_step: NifflerUpstreamErrorHandlingStep | ''
  response_mode: NifflerUserResponseMode
  user_message: string
  account_protection_action: NifflerAccountProtectionAction
  pause_duration: NifflerPauseDuration | ''
  is_active: boolean
}

const { success, error: showError } = useToast()
type ErrorRuleColumnKey = 'scope' | 'match' | 'response' | 'message' | 'protection' | 'status'
const errorRuleColumns: ResizableTableColumn<ErrorRuleColumnKey>[] = [
  { key: 'scope', width: '150px', minWidth: 130 },
  { key: 'match', width: '220px', minWidth: 180 },
  { key: 'response', width: '130px', minWidth: 110 },
  { key: 'message', width: '360px', minWidth: 260 },
  { key: 'protection', width: '160px', minWidth: 140 },
  { key: 'status', width: '100px', minWidth: 90 },
]
const {
  columnWidths: errorRuleColumnWidths,
  startResize: handleErrorRuleColumnResizeStart,
} = useResizableTableColumns<ErrorRuleColumnKey>({
  storageKey: 'niffler-error-rules-table-column-widths',
  columns: errorRuleColumns,
  defaultMinWidth: 90,
})

const scopeFilters: Array<{
  value: ScopeFilter
  label: string
  description: string
}> = [
  { value: 'all', label: '全部规则', description: '查看平台和上游错误。' },
  { value: 'platform', label: '平台错误', description: '余额、密钥、模型权限等本地错误。' },
  { value: 'upstream', label: '上游错误', description: '第三方或官方上游返回的错误。' },
]

const defaultErrorReturnSettingForm = (): ErrorReturnSettingForm => ({
  scope: 'platform',
  upstream_service_id: '__all__',
  match_status_code: null,
  match_text: '',
  handling_step: '',
  response_mode: 'replace',
  user_message: '',
  account_protection_action: 'record_only',
  pause_duration: '',
  is_active: true,
})

const services = ref<NifflerUpstreamService[]>([])
const errorReturnSettings = ref<NifflerErrorReturnSetting[]>([])
const serviceLoading = ref(false)
const errorReturnSettingLoading = ref(false)
const savingErrorReturnSetting = ref(false)
const errorReturnSettingError = ref('')
const scopeFilter = ref<ScopeFilter>('all')
const errorReturnSettingDialogOpen = ref(false)
const errorReturnSettingForm = ref<ErrorReturnSettingForm>(defaultErrorReturnSettingForm())

const pageLoading = computed(() => serviceLoading.value || errorReturnSettingLoading.value)

const filteredErrorReturnSettings = computed(() => {
  if (scopeFilter.value === 'all') return errorReturnSettings.value
  return errorReturnSettings.value.filter(rule => rule.scope === scopeFilter.value)
})

const serviceNameById = computed(() =>
  new Map(services.value.map(service => [service.id, service.display_name]))
)

watch(errorReturnSettingDialogOpen, (open) => {
  if (!open) {
    errorReturnSettingForm.value = defaultErrorReturnSettingForm()
  }
})

async function refreshAll() {
  await Promise.all([
    loadServices(),
    loadErrorReturnSettings(),
  ])
}

async function loadServices() {
  serviceLoading.value = true
  try {
    const response = await listNifflerUpstreamServices({
      include_inactive: true,
      limit: 100,
    })
    services.value = response.items
  } catch (err) {
    showError(extractErrorMessage(err, '读取上游失败'))
  } finally {
    serviceLoading.value = false
  }
}

async function loadErrorReturnSettings() {
  errorReturnSettingLoading.value = true
  errorReturnSettingError.value = ''
  try {
    const response = await listNifflerErrorReturnSettings({
      include_inactive: true,
      limit: 100,
    })
    errorReturnSettings.value = response.items
  } catch (err) {
    errorReturnSettingError.value = extractErrorMessage(err, '读取规则失败')
    showError(errorReturnSettingError.value)
  } finally {
    errorReturnSettingLoading.value = false
  }
}

async function submitErrorReturnSetting() {
  const payload = normalizeErrorReturnSettingPayload(errorReturnSettingForm.value)
  if (!payload) return

  savingErrorReturnSetting.value = true
  try {
    await createNifflerErrorReturnSetting(payload)
    success('规则已保存')
    errorReturnSettingDialogOpen.value = false
    await loadErrorReturnSettings()
  } catch (err) {
    showError(extractErrorMessage(err, '保存规则失败'))
  } finally {
    savingErrorReturnSetting.value = false
  }
}

function normalizeErrorReturnSettingPayload(
  form: ErrorReturnSettingForm
): CreateNifflerErrorReturnSettingPayload | null {
  const returnText = form.user_message.trim()
  if (!returnText) {
    showError('用户文案不能为空')
    return null
  }

  const rawStatusCode = form.match_status_code
  let matchStatusCode: number | null = null
  if (rawStatusCode !== null && rawStatusCode !== '') {
    const parsed = Number(rawStatusCode)
    if (!Number.isInteger(parsed) || parsed < 100 || parsed > 599) {
      showError('状态码必须是 100 到 599 之间的整数')
      return null
    }
    matchStatusCode = parsed
  }

  if (form.scope === 'upstream' && !form.handling_step) {
    showError('上游错误必须选择处理类型')
    return null
  }

  if (
    form.scope === 'upstream'
    && form.account_protection_action === 'pause_scheduling'
    && !form.pause_duration
  ) {
    showError('暂停调度必须选择时长')
    return null
  }

  return {
    scope: form.scope,
    upstream_service_id:
      form.scope === 'upstream' && form.upstream_service_id !== '__all__'
        ? form.upstream_service_id
        : null,
    match_status_code: matchStatusCode,
    match_text: emptyToNull(form.match_text),
    handling_step:
      form.scope === 'upstream'
        ? (form.handling_step as NifflerUpstreamErrorHandlingStep)
        : null,
    response_mode: form.response_mode,
    user_message: returnText,
    account_protection_action:
      form.scope === 'upstream' ? form.account_protection_action : 'record_only',
    pause_duration:
      form.scope === 'upstream' && form.account_protection_action === 'pause_scheduling'
        ? (form.pause_duration as NifflerPauseDuration)
        : null,
    is_active: form.is_active,
  }
}

function emptyToNull(value?: string | null): string | null {
  const normalized = value?.trim() ?? ''
  return normalized ? normalized : null
}

function errorScopeLabel(scope: NifflerErrorResponseScope): string {
  const labels: Record<NifflerErrorResponseScope, string> = {
    platform: '平台错误',
    upstream: '上游错误',
  }
  return labels[scope] ?? scope
}

function upstreamServiceLabel(serviceId?: string | null): string {
  if (!serviceId) return '全部上游'
  return serviceNameById.value.get(serviceId) || '未知上游'
}

function matchLabel(rule: NifflerErrorReturnSetting): string {
  const parts: string[] = []
  if (rule.match_status_code) {
    parts.push(`状态码 ${rule.match_status_code}`)
  }
  if (rule.match_text) {
    parts.push(rule.scope === 'platform' ? `错误代码：${rule.match_text}` : `关键词：${rule.match_text}`)
  }
  return parts.length > 0 ? parts.join(' / ') : '默认规则'
}

function handlingStepLabel(step: NifflerUpstreamErrorHandlingStep): string {
  const labels: Record<NifflerUpstreamErrorHandlingStep, string> = {
    risk_keyword: '风控关键词',
    contact_or_marketing_replacement: '广告替换',
    status_code_message: '状态码文案',
    default_upstream_message: '默认上游文案',
  }
  return labels[step] ?? step
}

function responseModeLabel(mode: NifflerUserResponseMode): string {
  const labels: Record<NifflerUserResponseMode, string> = {
    replace: '替换',
    append: '追加',
    redact: '脱敏',
  }
  return labels[mode] ?? mode
}

function protectionActionLabel(action: NifflerAccountProtectionAction): string {
  const labels: Record<NifflerAccountProtectionAction, string> = {
    record_only: '只记录',
    pause_scheduling: '暂停调度',
    disable_account: '人工处理',
  }
  return labels[action] ?? action
}

function pauseDurationLabel(duration: NifflerPauseDuration): string {
  const labels: Record<NifflerPauseDuration, string> = {
    ten_minutes: '10 分钟',
    one_hour: '1 小时',
    twenty_four_hours: '24 小时',
    manual_restore: '手动恢复',
  }
  return labels[duration] ?? duration
}

onMounted(() => {
  void refreshAll()
})
</script>
