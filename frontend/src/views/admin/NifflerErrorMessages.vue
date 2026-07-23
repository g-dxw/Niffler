<template>
  <PageContainer>
    <PageHeader
      :title="t('errorMessages.title')"
      :description="t('errorMessages.description')"
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
          {{ t('errorMessages.refresh') }}
        </Button>
        <Button
          class="admin-entry-action"
          @click="errorReturnSettingDialogOpen = true"
        >
          <Plus class="mr-2 h-4 w-4" />
          {{ t('errorMessages.add') }}
        </Button>
      </template>
    </PageHeader>

    <div class="mt-6 grid gap-5 xl:grid-cols-[320px_minmax(0,1fr)]">
      <Card class="p-4">
        <h2 class="text-sm font-semibold">
          {{ t('errorMessages.ruleType') }}
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
              {{ t('errorMessages.rules') }}
            </h2>
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('errorMessages.ruleHint') }}
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
          {{ t('errorMessages.loading') }}
        </div>

        <div
          v-else-if="filteredErrorReturnSettings.length === 0"
          class="py-16 text-center"
        >
          <AlertTriangle class="mx-auto h-10 w-10 text-muted-foreground/50" />
          <p class="mt-3 font-medium">
            {{ t('errorMessages.empty') }}
          </p>
          <p class="mt-1 text-sm text-muted-foreground">
            {{ t('errorMessages.emptyHint') }}
          </p>
          <Button
            class="admin-entry-action mt-4"
            @click="errorReturnSettingDialogOpen = true"
          >
            {{ t('errorMessages.add') }}
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
                {{ t('errorMessages.scope') }}
              </SortableTableHead>
              <SortableTableHead :sortable="false" resize-column-key="match" :resizable="true" @resize-start="handleErrorRuleColumnResizeStart">
                {{ t('errorMessages.match') }}
              </SortableTableHead>
              <SortableTableHead :sortable="false" resize-column-key="response" :resizable="true" @resize-start="handleErrorRuleColumnResizeStart">
                {{ t('errorMessages.response') }}
              </SortableTableHead>
              <SortableTableHead :sortable="false" resize-column-key="message" :resizable="true" @resize-start="handleErrorRuleColumnResizeStart">
                {{ t('errorMessages.userMessage') }}
              </SortableTableHead>
              <SortableTableHead :sortable="false" resize-column-key="protection" :resizable="true" @resize-start="handleErrorRuleColumnResizeStart">
                {{ t('errorMessages.protection') }}
              </SortableTableHead>
              <SortableTableHead :sortable="false" resize-column-key="status" :resizable="true" @resize-start="handleErrorRuleColumnResizeStart">
                {{ t('errorMessages.status') }}
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
                  {{ rule.is_active ? t('errorMessages.enabled') : t('errorMessages.disabled') }}
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
      :title="t('errorMessages.addTitle')"
      :description="t('errorMessages.addHint')"
      :icon="AlertTriangle"
    >
      <form
        class="space-y-5"
        @submit.prevent="submitErrorReturnSetting"
      >
        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-2">
            <Label for="error-scope">{{ t('errorMessages.scope') }}</Label>
            <Select v-model="errorReturnSettingForm.scope">
              <SelectTrigger id="error-scope">
                <SelectValue :placeholder="t('errorMessages.chooseScope')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="platform">
                  {{ t('errorMessages.platform') }}
                </SelectItem>
                <SelectItem value="upstream">
                  {{ t('errorMessages.upstream') }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div
            v-if="errorReturnSettingForm.scope === 'platform'"
            class="space-y-2"
          >
          <Label for="error-platform-code">{{ t('errorMessages.errorReason') }}</Label>
            <Select v-model="errorReturnSettingForm.platform_error_code">
              <SelectTrigger id="error-platform-code">
              <SelectValue :placeholder="t('errorMessages.chooseReason')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="option in platformErrorOptions"
                  :key="option.value"
                  :value="option.value"
                >
                  {{ option.label }} · HTTP {{ option.statusCode }}
                </SelectItem>
              </SelectContent>
            </Select>
            <p
              v-if="selectedPlatformErrorOption"
              class="text-xs text-muted-foreground"
            >
              {{ selectedPlatformErrorOption.description }}
            </p>
          </div>

          <div
            v-if="errorReturnSettingForm.scope === 'upstream'"
            class="space-y-2"
          >
          <Label for="error-upstream">{{ t('errorMessages.upstream') }}</Label>
            <Select v-model="errorReturnSettingForm.upstream_service_id">
              <SelectTrigger id="error-upstream">
              <SelectValue :placeholder="t('errorMessages.chooseUpstream')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__all__">
                {{ t('errorMessages.allUpstream') }}
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
          <Label for="error-step">{{ t('errorMessages.handlingType') }}</Label>
            <Select v-model="errorReturnSettingForm.handling_step">
              <SelectTrigger id="error-step">
              <SelectValue :placeholder="t('errorMessages.chooseType')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="risk_keyword">
                {{ t('errorMessages.risk') }}
                </SelectItem>
                <SelectItem value="contact_or_marketing_replacement">
                {{ t('errorMessages.content') }}
                </SelectItem>
                <SelectItem value="status_code_message">
                {{ t('errorMessages.statusMessage') }}
                </SelectItem>
                <SelectItem value="default_upstream_message">
                {{ t('errorMessages.defaultMessage') }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div
            v-if="showUpstreamStatusCodeField"
            class="space-y-2"
          >
            <Label for="error-status-code">{{ t('errorMessages.upstreamStatusCode') }}</Label>
            <Input
              id="error-status-code"
              v-model="errorReturnSettingForm.match_status_code"
              type="number"
              min="100"
              max="599"
              step="1"
              :placeholder="t('errorMessages.statusCodePlaceholder')"
            />
          </div>

          <div
            v-if="showUpstreamKeywordField"
            class="space-y-2 sm:col-span-2"
          >
            <Label for="error-match-text">
              {{ errorReturnSettingForm.handling_step === 'risk_keyword' ? t('errorMessages.riskKeyword') : t('errorMessages.matchContent') }}
            </Label>
            <Input
              id="error-match-text"
              v-model="errorReturnSettingForm.match_text"
              :placeholder="errorReturnSettingForm.handling_step === 'risk_keyword' ? t('errorMessages.riskPlaceholder') : t('errorMessages.matchPlaceholder')"
            />
          </div>

          <div class="space-y-2">
          <Label for="error-response-mode">{{ t('errorMessages.responseMode') }}</Label>
            <Select v-model="errorReturnSettingForm.response_mode">
              <SelectTrigger id="error-response-mode">
              <SelectValue :placeholder="t('errorMessages.chooseMode')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="option in responseModeOptions"
                  :key="option.value"
                  :value="option.value"
                >
                  {{ option.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div
            v-if="showAccountProtectionField"
            class="space-y-2"
          >
          <Label for="error-protection">{{ t('errorMessages.protection') }}</Label>
            <Select v-model="errorReturnSettingForm.account_protection_action">
              <SelectTrigger id="error-protection">
              <SelectValue :placeholder="t('errorMessages.chooseAction')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="record_only">
                {{ t('errorMessages.recordOnly') }}
                </SelectItem>
                <SelectItem value="pause_scheduling">
                {{ t('errorMessages.pause') }}
                </SelectItem>
                <SelectItem value="disable_account">
                {{ t('errorMessages.disableAccount') }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div
            v-if="showAccountProtectionField && errorReturnSettingForm.account_protection_action === 'pause_scheduling'"
            class="space-y-2"
          >
            <Label for="error-pause-duration">{{ t('errorMessages.pauseDuration') }}</Label>
            <Select v-model="errorReturnSettingForm.pause_duration">
              <SelectTrigger id="error-pause-duration">
                <SelectValue :placeholder="t('errorMessages.chooseDuration')" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="ten_minutes">
                  {{ t('errorMessages.tenMinutes') }}
                </SelectItem>
                <SelectItem value="one_hour">
                  {{ t('errorMessages.oneHour') }}
                </SelectItem>
                <SelectItem value="twenty_four_hours">
                  {{ t('errorMessages.twentyFourHours') }}
                </SelectItem>
                <SelectItem value="manual_restore">
                  {{ t('errorMessages.manualRestore') }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="flex items-center gap-3 pt-7">
            <Switch
              id="error-active"
              v-model="errorReturnSettingForm.is_active"
            />
            <Label for="error-active">{{ t('errorMessages.enabled') }}</Label>
          </div>
        </div>

        <div class="space-y-2">
          <Label for="error-user-message">{{ t('errorMessages.userMessage') }}</Label>
          <Textarea
            id="error-user-message"
            v-model="errorReturnSettingForm.user_message"
            rows="4"
            :placeholder="userMessagePlaceholder"
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
          {{ savingErrorReturnSetting ? t('errorMessages.saving') : t('errorMessages.saveRule') }}
        </Button>
        <Button
          class="admin-entry-action"
          type="button"
          variant="outline"
          :disabled="savingErrorReturnSetting"
          @click="errorReturnSettingDialogOpen = false"
        >
          {{ t('errorMessages.cancel') }}
        </Button>
      </template>
    </Dialog>
  </PageContainer>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
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
  platform_error_code: string
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
const platformErrorOptions = computed(() => [
  {
    value: 'missing_api_key',
    label: t('errorMessages.missingApiKey'),
    statusCode: 401,
    description: t('errorMessages.missingApiKeyHint'),
  },
  {
    value: 'invalid_api_key',
    label: t('errorMessages.invalidApiKey'),
    statusCode: 401,
    description: t('errorMessages.invalidApiKeyHint'),
  },
  {
    value: 'locked_api_key',
    label: t('errorMessages.lockedApiKey'),
    statusCode: 403,
    description: t('errorMessages.lockedApiKeyHint'),
  },
  {
    value: 'wallet_unavailable',
    label: t('errorMessages.walletUnavailable'),
    statusCode: 403,
    description: t('errorMessages.walletUnavailableHint'),
  },
  {
    value: 'insufficient_balance',
    label: t('errorMessages.insufficientBalance'),
    statusCode: 402,
    description: t('errorMessages.insufficientBalanceHint'),
  },
  {
    value: 'provider_not_allowed',
    label: t('errorMessages.providerUnavailable'),
    statusCode: 403,
    description: t('errorMessages.providerUnavailableHint'),
  },
  {
    value: 'api_format_not_allowed',
    label: t('errorMessages.formatUnavailable'),
    statusCode: 403,
    description: t('errorMessages.formatUnavailableHint'),
  },
  {
    value: 'model_not_allowed',
    label: t('errorMessages.modelUnavailable'),
    statusCode: 403,
    description: t('errorMessages.modelUnavailableHint'),
  },
  {
    value: 'rate_limit_exceeded',
    label: t('errorMessages.rateLimited'),
    statusCode: 429,
    description: t('errorMessages.rateLimitedHint'),
  },
  {
    value: 'request_body_normalization_failed',
    label: t('errorMessages.invalidRequest'),
    statusCode: 400,
    description: t('errorMessages.invalidRequestHint'),
  },
  {
    value: 'local_execution_runtime_unavailable',
    label: t('errorMessages.noUpstream'),
    statusCode: 503,
    description: t('errorMessages.noUpstreamHint'),
  },
  {
    value: 'local_proxy_passthrough_removed',
    label: t('errorMessages.unsupportedEndpoint'),
    statusCode: 501,
    description: t('errorMessages.unsupportedEndpointHint'),
  },
] as const)
type PlatformErrorOption = (typeof platformErrorOptions.value)[number]
type PlatformErrorCode = PlatformErrorOption['value']

const responseModeOptionGroups = computed<Record<'basic' | 'keyword', Array<{
  value: NifflerUserResponseMode
  label: string
}>>>(() => ({
  basic: [
    { value: 'replace', label: t('errorMessages.replace') },
    { value: 'append', label: t('errorMessages.append') },
  ],
  keyword: [
    { value: 'replace', label: t('errorMessages.replace') },
    { value: 'append', label: t('errorMessages.append') },
    { value: 'redact', label: t('errorMessages.redact') },
  ],
}))

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

const scopeFilters = computed<Array<{
  value: ScopeFilter
  label: string
  description: string
}>>(() => [
  { value: 'all', label: t('errorMessages.allRules'), description: t('errorMessages.allRulesHint') },
  { value: 'platform', label: t('errorMessages.platform'), description: t('errorMessages.platformHint') },
  { value: 'upstream', label: t('errorMessages.upstream'), description: t('errorMessages.upstreamHint') },
])

const defaultErrorReturnSettingForm = (): ErrorReturnSettingForm => ({
  scope: 'platform',
  upstream_service_id: '__all__',
  platform_error_code: '',
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

const platformErrorOptionByCode = computed(() =>
  new Map(platformErrorOptions.value.map(option => [option.value, option]))
)

function getPlatformErrorOption(code: string | null | undefined): PlatformErrorOption | null {
  if (!code) return null
  if (code === 'balance_exceeded') {
    return platformErrorOptionByCode.value.get('insufficient_balance') ?? null
  }
  return platformErrorOptionByCode.value.get(code as PlatformErrorCode) ?? null
}

const selectedPlatformErrorOption = computed<PlatformErrorOption | null>(() =>
  getPlatformErrorOption(errorReturnSettingForm.value.platform_error_code)
)

const showUpstreamKeywordField = computed(() =>
  errorReturnSettingForm.value.scope === 'upstream'
  && (
    errorReturnSettingForm.value.handling_step === 'risk_keyword'
    || errorReturnSettingForm.value.handling_step === 'contact_or_marketing_replacement'
  )
)

const showUpstreamStatusCodeField = computed(() =>
  errorReturnSettingForm.value.scope === 'upstream'
  && errorReturnSettingForm.value.handling_step === 'status_code_message'
)

const showAccountProtectionField = computed(() =>
  errorReturnSettingForm.value.scope === 'upstream'
  && errorReturnSettingForm.value.handling_step === 'risk_keyword'
)

const responseModeOptions = computed(() =>
  showUpstreamKeywordField.value ? responseModeOptionGroups.value.keyword : responseModeOptionGroups.value.basic
)

const userMessagePlaceholder = computed(() => {
  if (errorReturnSettingForm.value.scope === 'platform') {
    return t('errorMessages.platformMessagePlaceholder')
  }
  if (errorReturnSettingForm.value.handling_step === 'risk_keyword') {
    return t('errorMessages.riskMessagePlaceholder')
  }
  if (errorReturnSettingForm.value.handling_step === 'contact_or_marketing_replacement') {
    return t('errorMessages.upstreamMessagePlaceholder')
  }
  if (errorReturnSettingForm.value.handling_step === 'status_code_message') {
    return t('errorMessages.statusMessagePlaceholder')
  }
  return t('errorMessages.upstreamMessagePlaceholder')
})

watch(errorReturnSettingDialogOpen, (open) => {
  if (!open) {
    errorReturnSettingForm.value = defaultErrorReturnSettingForm()
  }
})

watch(
  () => errorReturnSettingForm.value.scope,
  (scope) => {
    errorReturnSettingForm.value.match_status_code = null
    errorReturnSettingForm.value.match_text = ''
    errorReturnSettingForm.value.response_mode = 'replace'
    errorReturnSettingForm.value.account_protection_action = 'record_only'
    errorReturnSettingForm.value.pause_duration = ''
    if (scope === 'platform') {
      errorReturnSettingForm.value.upstream_service_id = '__all__'
      errorReturnSettingForm.value.handling_step = ''
      return
    }
    errorReturnSettingForm.value.platform_error_code = ''
  }
)

watch(
  () => errorReturnSettingForm.value.handling_step,
  (step) => {
    errorReturnSettingForm.value.response_mode = 'replace'
    if (step !== 'status_code_message') {
      errorReturnSettingForm.value.match_status_code = null
    }
    if (step !== 'risk_keyword' && step !== 'contact_or_marketing_replacement') {
      errorReturnSettingForm.value.match_text = ''
    }
    if (step !== 'risk_keyword') {
      errorReturnSettingForm.value.account_protection_action = 'record_only'
      errorReturnSettingForm.value.pause_duration = ''
    }
  }
)

watch(
  () => errorReturnSettingForm.value.response_mode,
  (mode) => {
    if (mode === 'redact' && !showUpstreamKeywordField.value) {
      errorReturnSettingForm.value.response_mode = 'replace'
    }
  }
)

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
    showError(extractErrorMessage(err, t('errorMessages.loadUpstreamsFailed')))
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
    errorReturnSettingError.value = extractErrorMessage(err, t('errorMessages.loadRulesFailed'))
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
    success(t('errorMessages.saved'))
    errorReturnSettingDialogOpen.value = false
    await loadErrorReturnSettings()
  } catch (err) {
    showError(extractErrorMessage(err, t('errorMessages.saveFailed')))
  } finally {
    savingErrorReturnSetting.value = false
  }
}

function normalizeErrorReturnSettingPayload(
  form: ErrorReturnSettingForm
): CreateNifflerErrorReturnSettingPayload | null {
  const returnText = form.user_message.trim()
  if (!returnText) {
    showError(t('errorMessages.userMessageRequired'))
    return null
  }

  let matchStatusCode: number | null = null
  let matchText: string | null = null
  let accountProtectionAction: NifflerAccountProtectionAction = 'record_only'
  let pauseDuration: NifflerPauseDuration | null = null

  if (form.scope === 'platform') {
    const platformOption = getPlatformErrorOption(form.platform_error_code)
    if (!platformOption) {
      showError(t('errorMessages.platformReasonRequired'))
      return null
    }
    matchStatusCode = platformOption.statusCode
    matchText = platformOption.value
  } else {
    if (!form.handling_step) {
      showError(t('errorMessages.handlingTypeRequired'))
      return null
    }

    if (
      form.handling_step === 'risk_keyword'
      || form.handling_step === 'contact_or_marketing_replacement'
    ) {
      matchText = emptyToNull(form.match_text)
      if (!matchText) {
        showError(form.handling_step === 'risk_keyword' ? t('errorMessages.riskKeywordRequired') : t('errorMessages.matchContentRequired'))
        return null
      }
    }

    if (form.handling_step === 'status_code_message') {
      matchStatusCode = parseStatusCode(form.match_status_code)
      if (matchStatusCode === null) return null
    }

    if (form.handling_step === 'risk_keyword') {
      accountProtectionAction = form.account_protection_action
      if (accountProtectionAction === 'pause_scheduling') {
        if (!form.pause_duration) {
          showError(t('errorMessages.pauseDurationRequired'))
          return null
        }
        pauseDuration = form.pause_duration as NifflerPauseDuration
      }
    }
  }

  return {
    scope: form.scope,
    upstream_service_id:
      form.scope === 'upstream' && form.upstream_service_id !== '__all__'
        ? form.upstream_service_id
        : null,
    match_status_code: matchStatusCode,
    match_text: matchText,
    handling_step:
      form.scope === 'upstream'
        ? (form.handling_step as NifflerUpstreamErrorHandlingStep)
        : null,
    response_mode: form.response_mode,
    user_message: returnText,
    account_protection_action: accountProtectionAction,
    pause_duration: pauseDuration,
    is_active: form.is_active,
  }
}

function parseStatusCode(value: number | string | null): number | null {
  if (value === null || value === '') {
    showError(t('errorMessages.statusCodeRequired'))
    return null
  }
  const parsed = Number(value)
  if (!Number.isInteger(parsed) || parsed < 100 || parsed > 599) {
    showError(t('errorMessages.statusCodeInvalid'))
    return null
  }
  return parsed
}

function emptyToNull(value?: string | null): string | null {
  const normalized = value?.trim() ?? ''
  return normalized ? normalized : null
}

function errorScopeLabel(scope: NifflerErrorResponseScope): string {
  const labels: Record<NifflerErrorResponseScope, string> = {
    platform: t('errorMessages.platform'),
    upstream: t('errorMessages.upstream'),
  }
  return labels[scope] ?? scope
}

function upstreamServiceLabel(serviceId?: string | null): string {
  if (!serviceId) return t('errorMessages.allUpstream')
  return serviceNameById.value.get(serviceId) || t('errorMessages.unknownUpstream')
}

function matchLabel(rule: NifflerErrorReturnSetting): string {
  if (rule.scope === 'platform') {
    const option = rule.match_text
      ? getPlatformErrorOption(rule.match_text)
      : null
    const label = option?.label ?? rule.match_text ?? t('errorMessages.platform')
    return rule.match_status_code ? `${label} · HTTP ${rule.match_status_code}` : label
  }

  if (rule.handling_step === 'default_upstream_message') {
    return t('errorMessages.noSpecificMatch')
  }

  const parts: string[] = []
  if (rule.match_status_code) {
    parts.push(`HTTP ${rule.match_status_code}`)
  }
  if (rule.match_text) {
    parts.push(t('errorMessages.keywordValue', { value: rule.match_text }))
  }
  return parts.length > 0 ? parts.join(' / ') : t('errorMessages.defaultRule')
}

function handlingStepLabel(step: NifflerUpstreamErrorHandlingStep): string {
  const labels: Record<NifflerUpstreamErrorHandlingStep, string> = {
    risk_keyword: t('errorMessages.risk'),
    contact_or_marketing_replacement: t('errorMessages.content'),
    status_code_message: t('errorMessages.statusMessage'),
    default_upstream_message: t('errorMessages.defaultMessage'),
  }
  return labels[step] ?? step
}

function responseModeLabel(mode: NifflerUserResponseMode): string {
  const labels: Record<NifflerUserResponseMode, string> = {
    replace: t('errorMessages.replaceShort'),
    append: t('errorMessages.appendShort'),
    redact: t('errorMessages.redactShort'),
  }
  return labels[mode] ?? mode
}

function protectionActionLabel(action: NifflerAccountProtectionAction): string {
  const labels: Record<NifflerAccountProtectionAction, string> = {
    record_only: t('errorMessages.recordOnly'),
    pause_scheduling: t('errorMessages.pause'),
    disable_account: t('errorMessages.disableAccount'),
  }
  return labels[action] ?? action
}

function pauseDurationLabel(duration: NifflerPauseDuration): string {
  const labels: Record<NifflerPauseDuration, string> = {
    ten_minutes: t('errorMessages.tenMinutes'),
    one_hour: t('errorMessages.oneHour'),
    twenty_four_hours: t('errorMessages.twentyFourHours'),
    manual_restore: t('errorMessages.manualRestore'),
  }
  return labels[duration] ?? duration
}

onMounted(() => {
  void refreshAll()
})
</script>
