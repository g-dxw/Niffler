<template>
  <Dialog
    :model-value="internalOpen"
    :title="isEditMode ? t('providerForm.editTitle') : t('providerForm.addTitle')"
    :description="isEditMode ? t('providerForm.editDescription') : t('providerForm.addDescription')"
    :icon="isEditMode ? SquarePen : Server"
    size="xl"
    @update:model-value="handleDialogUpdate"
  >
    <form
      class="space-y-5"
      @submit.prevent="handleSubmit"
    >
      <!-- 基本信息 -->
      <div class="space-y-3">
        <h3 class="text-sm font-medium border-b pb-2">
          {{ t('providerForm.basicInfo') }}
        </h3>

        <div class="space-y-1.5">
          <Label for="name">{{ t('providerForm.name') }} *</Label>
          <Input
            id="name"
            v-model="form.name"
            :placeholder="t('providerForm.namePlaceholder')"
          />
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <Label>{{ t('providerForm.type') }}</Label>
            <Select
              v-model="form.provider_type"
              :disabled="isEditMode"
            >
              <SelectTrigger>
                <SelectValue :placeholder="t('providerForm.choose')" />
              </SelectTrigger>
              <SelectContent>
                <!-- 新建模式：允许自定义及各反代类型 -->
                <template v-if="!isEditMode">
                  <SelectItem value="custom">
                    {{ t('providerForm.custom') }}
                  </SelectItem>
                  <SelectItem value="vertex_ai">
                    Vertex AI
                  </SelectItem>
                  <SelectItem
                    value="claude_code"
                    disabled
                  >
                    {{ t('providerForm.claudeOAuthUnavailable') }}
                  </SelectItem>
                  <SelectItem value="claude_code_api">
                    {{ t('providerForm.claudeCompatible') }}
                  </SelectItem>
                  <SelectItem value="codex">
                    Codex
                  </SelectItem>
                  <SelectItem value="chatgpt_web">
                    ChatGPT Web
                  </SelectItem>
                  <SelectItem value="gemini_cli">
                    Gemini CLI
                  </SelectItem>
                  <SelectItem value="grok">
                    Grok
                  </SelectItem>
                  <SelectItem value="kiro">
                    Kiro
                  </SelectItem>
                  <SelectItem value="antigravity">
                    Antigravity
                  </SelectItem>
                </template>
                <!-- 编辑模式：显示所有类型（兼容已有数据） -->
                <template v-else>
                  <SelectItem value="custom">
                    {{ t('providerForm.custom') }}
                  </SelectItem>
                  <SelectItem value="vertex_ai">
                    Vertex AI
                  </SelectItem>
                  <SelectItem value="claude_code">
                    ClaudeCode OAuth
                  </SelectItem>
                  <SelectItem value="claude_code_api">
                    {{ t('providerForm.claudeCompatible') }}
                  </SelectItem>
                  <SelectItem value="codex">
                    Codex
                  </SelectItem>
                  <SelectItem value="chatgpt_web">
                    ChatGPT Web
                  </SelectItem>
                  <SelectItem value="gemini_cli">
                    Gemini CLI
                  </SelectItem>
                  <SelectItem value="grok">
                    Grok
                  </SelectItem>
                  <SelectItem value="kiro">
                    Kiro
                  </SelectItem>
                  <SelectItem value="antigravity">
                    Antigravity
                  </SelectItem>
                </template>
              </SelectContent>
            </Select>
            <p
              v-if="!isEditMode && form.provider_type !== 'custom' && form.provider_type !== 'claude_code_api'"
              class="text-xs text-muted-foreground"
            >
              {{ t('providerForm.fixedEndpointHint') }}
            </p>
          </div>
          <div class="space-y-1.5">
            <Label for="website">{{ t('providerForm.website') }}</Label>
            <Input
              id="website"
              v-model="form.website"
              :placeholder="t('providerForm.websitePlaceholder')"
            />
          </div>
        </div>
      </div>

      <!-- 计费与限流 / 请求配置 -->
      <div class="space-y-3">
        <div class="grid grid-cols-2 gap-4">
          <h3 class="text-sm font-medium border-b pb-2">
            {{ t('providerForm.billingRate') }}
          </h3>
          <h3 class="text-sm font-medium border-b pb-2">
            {{ t('providerForm.requestConfig') }}
          </h3>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <Label>{{ t('providerForm.billingType') }}</Label>
            <Select
              v-model="form.billing_type"
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="monthly_quota">
                  {{ t('providerForm.monthlyQuota') }}
                </SelectItem>
                <SelectItem value="pay_as_you_go">
                  {{ t('providerForm.payAsYouGo') }}
                </SelectItem>
                <SelectItem value="free_tier">
                  {{ t('providerForm.freeTier') }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="space-y-1.5">
            <Label>{{ t('providerForm.defaultCostMultiplier') }}</Label>
            <Input
              :model-value="form.cost_multiplier ?? ''"
              type="number"
              min="0"
              step="0.01"
              :placeholder="t('providerForm.defaultOne')"
              @update:model-value="(v) => form.cost_multiplier = parseNumberInput(v, { allowFloat: true })"
            />
            <p class="text-xs text-muted-foreground">
              {{ t('providerForm.costMultiplierHint') }}
            </p>
          </div>
          <div class="space-y-1.5">
            <Label>{{ t('providerForm.maxRetries') }}</Label>
            <Input
              :model-value="form.max_retries ?? ''"
              type="number"
              min="0"
              max="999"
              :placeholder="t('providerForm.defaultTwo')"
              @update:model-value="(v) => form.max_retries = parseNumberInput(v)"
            />
          </div>
        </div>

        <!-- 超时配置 -->
        <div class="grid grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <Label>
              {{ t('providerForm.streamTimeout') }}
              <span class="text-xs text-muted-foreground">({{ t('providerForm.seconds') }})</span>
            </Label>
            <Input
              :model-value="form.stream_first_byte_timeout ?? ''"
              type="number"
              min="1"
              max="300"
              step="1"
              placeholder="30"
              @update:model-value="(v) => form.stream_first_byte_timeout = parseNumberInput(v)"
            />
          </div>
          <div class="space-y-1.5">
            <Label>
              {{ t('providerForm.requestTimeout') }}
              <span class="text-xs text-muted-foreground">({{ t('providerForm.seconds') }})</span>
            </Label>
            <Input
              :model-value="form.request_timeout ?? ''"
              type="number"
              min="1"
              max="600"
              step="1"
              placeholder="300"
              @update:model-value="(v) => form.request_timeout = parseNumberInput(v)"
            />
          </div>
        </div>

        <!-- 月卡配置 -->
        <div
          v-if="form.billing_type === 'monthly_quota'"
          class="grid grid-cols-2 gap-4 p-3 border rounded-lg bg-muted/50"
        >
          <div class="space-y-1.5">
            <Label class="text-xs">{{ t('providerForm.periodQuota') }}</Label>
            <Input
              :model-value="form.monthly_quota_usd ?? ''"
              type="number"
              step="0.01"
              min="0"
              @update:model-value="(v) => form.monthly_quota_usd = parseNumberInput(v, { allowFloat: true })"
            />
          </div>
          <div class="space-y-1.5">
            <Label class="text-xs">{{ t('providerForm.resetCycle') }}</Label>
            <Input
              :model-value="form.quota_reset_day ?? ''"
              type="number"
              min="1"
              max="365"
              @update:model-value="(v) => form.quota_reset_day = parseNumberInput(v) ?? 30"
            />
          </div>
          <div class="space-y-1.5">
            <Label class="text-xs">
              {{ t('providerForm.periodStart') }} <span class="text-red-500">*</span>
            </Label>
            <Input
              v-model="form.quota_last_reset_at"
              type="datetime-local"
            />
          </div>
          <div class="space-y-1.5">
            <Label class="text-xs">{{ t('providerForm.expiry') }}</Label>
            <Input
              v-model="form.quota_expires_at"
              type="datetime-local"
            />
          </div>
        </div>
      </div>

      <!-- 功能开关 -->
      <div class="space-y-3">
        <h3 class="text-sm font-medium border-b pb-2">
          {{ t('providerForm.featureToggles') }}
        </h3>

        <div class="flex items-center justify-between p-3 border rounded-lg bg-muted/50">
          <div class="space-y-0.5">
            <span class="text-sm font-medium">{{ t('providerForm.keepPriority') }}</span>
            <p class="text-xs text-muted-foreground">
              {{ t('providerForm.keepPriorityHint') }}
            </p>
          </div>
          <Switch
            :model-value="form.keep_priority_on_conversion"
            @update:model-value="(v: boolean) => form.keep_priority_on_conversion = v"
          />
        </div>

        <div class="flex items-center justify-between p-3 border rounded-lg bg-muted/50">
          <div class="space-y-0.5">
            <span class="text-sm font-medium">{{ t('providerForm.poolMode') }}</span>
            <p class="text-xs text-muted-foreground">
              {{ t('providerForm.poolModeHint') }}
            </p>
          </div>
          <Switch
            :model-value="form.pool_mode_enabled"
            @update:model-value="(v: boolean) => form.pool_mode_enabled = v"
          />
        </div>

        <div
          v-if="form.provider_type === 'kiro'"
          class="flex items-center justify-between p-3 border rounded-lg bg-muted/50"
        >
          <div class="space-y-0.5">
            <span class="text-sm font-medium">{{ t('providerForm.simulatedCache') }}</span>
            <p class="text-xs text-muted-foreground leading-relaxed">
              {{ t('providerForm.simulatedCacheHint') }}
            </p>
          </div>
          <Switch
            :model-value="form.kiro_simulated_cache_enabled"
            @update:model-value="(v: boolean) => form.kiro_simulated_cache_enabled = v"
          />
        </div>

        <div class="flex items-center justify-between gap-4 p-3 border rounded-lg bg-muted/50">
          <div class="space-y-0.5">
            <span class="text-sm font-medium">{{ t('providerForm.piiProtection') }}</span>
            <p class="text-xs text-muted-foreground leading-relaxed">
              {{ t('providerForm.piiProtectionHint') }}
            </p>
          </div>
        </div>
      </div>
    </form>

    <template #footer>
      <Button
        type="button"
        variant="outline"
        :disabled="loading"
        @click="handleCancel"
      >
        {{ t('providerForm.cancel') }}
      </Button>
      <Button
        :disabled="loading || !form.name"
        @click="handleSubmit"
      >
        {{ loading ? (isEditMode ? t('providerForm.saving') : t('providerForm.creating')) : (isEditMode ? t('providerForm.save') : t('providerForm.create')) }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Dialog,
  Button,
  Input,
  Label,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  Switch,
} from '@/components/ui'
import { Server, SquarePen } from 'lucide-vue-next'
import { useToast } from '@/composables/useToast'
import { useFormDialog } from '@/composables/useFormDialog'
import {
  createProvider,
  normalizePoolAdvancedConfig,
  updateProvider,
  type ProviderWithEndpointsSummary,
} from '@/api/endpoints'
import { parseApiError } from '@/utils/errorParser'
import { parseNumberInput } from '@/utils/form'
import { dateTimeLocalToRfc3339, formatDateTimeLocalInput } from '@/utils/date'

const props = defineProps<{
  modelValue: boolean
  provider?: ProviderWithEndpointsSummary | null  // 编辑模式时传入
  maxPriority?: number  // 当前已有的最大优先级值
}>()
const { t } = useI18n()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'providerCreated': []
  'providerUpdated': [provider: ProviderWithEndpointsSummary]
}>()

const { success, error: showError } = useToast()
const loading = ref(false)

// 内部状态
const internalOpen = computed(() => props.modelValue)

// 计算新建时的默认优先级
const defaultPriority = computed(() => {
  if (props.maxPriority != null) {
    return Math.min(props.maxPriority + 10, 10000)
  }
  return 100
})

// 表单数据
const form = ref({
  name: '',
  provider_type: 'custom' as 'custom' | 'vertex_ai' | 'claude_code' | 'claude_code_api' | 'codex' | 'chatgpt_web' | 'gemini_cli' | 'antigravity' | 'kiro' | 'grok',
  description: '',
  website: '',
  // 计费配置
  billing_type: 'pay_as_you_go' as 'monthly_quota' | 'pay_as_you_go' | 'free_tier',
  cost_multiplier: undefined as number | undefined,
  monthly_quota_usd: undefined as number | undefined,
  quota_reset_day: 30,
  quota_last_reset_at: '',  // 周期开始时间
  quota_expires_at: '',
  provider_priority: 100,
  keep_priority_on_conversion: false,  // 格式转换时是否保持优先级
  // 状态配置
  is_active: true,
  rate_limit: undefined as number | undefined,
  concurrent_limit: undefined as number | undefined,
  // 请求配置
  max_retries: undefined as number | undefined,
  // 超时配置（秒）
  stream_first_byte_timeout: undefined as number | undefined,
  request_timeout: undefined as number | undefined,
  // 号池模式
  pool_mode_enabled: false,
  // Kiro 专属配置
  kiro_simulated_cache_enabled: false,
})

// 重置表单
function resetForm() {
  form.value = {
    name: '',
    provider_type: 'custom',
    description: '',
    website: '',
    billing_type: 'pay_as_you_go',
    cost_multiplier: undefined,
    monthly_quota_usd: undefined,
    quota_reset_day: 30,
    quota_last_reset_at: '',
    quota_expires_at: '',
    provider_priority: defaultPriority.value,
    keep_priority_on_conversion: false,
    is_active: true,
    rate_limit: undefined,
    concurrent_limit: undefined,
    // 请求配置
    max_retries: undefined,
    // 超时配置
    stream_first_byte_timeout: undefined,
    request_timeout: undefined,
    // 号池模式
    pool_mode_enabled: false,
    // Kiro 专属配置
    kiro_simulated_cache_enabled: false,
  }
}

// 加载提供商数据（编辑模式）
function loadProviderData() {
  if (!props.provider) return
  const poolAdvanced = normalizePoolAdvancedConfig(props.provider.pool_advanced)

  form.value = {
    name: props.provider.name,
    provider_type: props.provider.provider_type || 'custom',
    description: props.provider.description || '',
    website: props.provider.website || '',
    billing_type: (props.provider.billing_type as 'monthly_quota' | 'pay_as_you_go' | 'free_tier') || 'pay_as_you_go',
    cost_multiplier: props.provider.cost_multiplier ?? undefined,
    monthly_quota_usd: props.provider.monthly_quota_usd || undefined,
    quota_reset_day: props.provider.quota_reset_day || 30,
    quota_last_reset_at: formatDateTimeLocalInput(props.provider.quota_last_reset_at),
    quota_expires_at: formatDateTimeLocalInput(props.provider.quota_expires_at),
    provider_priority: props.provider.provider_priority || 999,
    keep_priority_on_conversion: props.provider.keep_priority_on_conversion ?? false,
    is_active: props.provider.is_active,
    rate_limit: undefined,
    concurrent_limit: undefined,
    // 请求配置
    max_retries: props.provider.max_retries ?? undefined,
    // 超时配置
    stream_first_byte_timeout: props.provider.stream_first_byte_timeout ?? undefined,
    request_timeout: props.provider.request_timeout ?? undefined,
    // 号池模式
    pool_mode_enabled: poolAdvanced !== null,
    // Kiro 专属配置
    kiro_simulated_cache_enabled: props.provider.kiro_simulated_cache_enabled ?? false,
  }
}

// 使用 useFormDialog 统一处理对话框逻辑
const { isEditMode, handleDialogUpdate, handleCancel } = useFormDialog({
  isOpen: () => props.modelValue,
  entity: () => props.provider,
  isLoading: loading,
  onClose: () => emit('update:modelValue', false),
  loadData: loadProviderData,
  resetForm,
})

// 新建模式下切换 provider_type 时不自动开启号池模式
watch(() => form.value.provider_type, () => {
  if (!isEditMode.value) {
    form.value.pool_mode_enabled = false
  }
  if (form.value.provider_type !== 'kiro') {
    form.value.kiro_simulated_cache_enabled = false
  }
})

// 提交表单
const handleSubmit = async () => {
  if (!isEditMode.value && form.value.provider_type === 'claude_code') {
    showError(t('providerForm.claudeOAuthDisabled'), t('providerForm.validationFailed'))
    return
  }

  // 月卡类型必须设置周期开始时间
  if (form.value.billing_type === 'monthly_quota' && !form.value.quota_last_reset_at) {
    showError(t('providerForm.periodStartRequired'), t('providerForm.validationFailed'))
    return
  }

  const quotaLastResetAt = dateTimeLocalToRfc3339(form.value.quota_last_reset_at)
  if (form.value.billing_type === 'monthly_quota' && !quotaLastResetAt) {
    showError(t('providerForm.invalidPeriodStart'), t('providerForm.validationFailed'))
    return
  }
  const quotaExpiresAt = dateTimeLocalToRfc3339(form.value.quota_expires_at)
  if (form.value.quota_expires_at && !quotaExpiresAt) {
    showError(t('providerForm.invalidExpiry'), t('providerForm.validationFailed'))
    return
  }

  loading.value = true
  try {
    const currentPoolAdvanced = normalizePoolAdvancedConfig(props.provider?.pool_advanced)
    const basePayload = {
      name: form.value.name,
      provider_type: form.value.provider_type,
      description: form.value.description || undefined,
      website: form.value.website || undefined,
      billing_type: form.value.billing_type,
      cost_multiplier: form.value.cost_multiplier ?? null,
      monthly_quota_usd: form.value.monthly_quota_usd,
      quota_reset_day: form.value.quota_reset_day,
      quota_last_reset_at: quotaLastResetAt,
      quota_expires_at: quotaExpiresAt,
      keep_priority_on_conversion: form.value.keep_priority_on_conversion,
      is_active: form.value.is_active,
      // 请求配置
      max_retries: form.value.max_retries ?? undefined,
      // 超时配置（null 表示清除，使用全局配置）
      stream_first_byte_timeout: form.value.stream_first_byte_timeout ?? null,
      request_timeout: form.value.request_timeout ?? null,
      pool_advanced: form.value.pool_mode_enabled
        ? (currentPoolAdvanced ?? {})
        : null,
      ...(form.value.provider_type === 'kiro'
        ? {
            config: {
              kiro: {
                simulated_cache_enabled: form.value.kiro_simulated_cache_enabled,
              },
            },
          }
        : {}),
    }

    if (isEditMode.value && props.provider) {
      // 更新提供商
      const updated = await updateProvider(props.provider.id, {
        ...basePayload,
        provider_priority: form.value.provider_priority,
      })
      success(t('providerForm.updated'))
      emit('providerUpdated', updated)
    } else {
      // 创建提供商（优先级由后端自动置顶）
      await createProvider(basePayload)
      success(t('providerForm.createdHint'), t('providerForm.created'))
      emit('providerCreated')
    }

    emit('update:modelValue', false)
  } catch (error: unknown) {
    const action = isEditMode.value ? t('providerForm.update') : t('providerForm.create')
    showError(parseApiError(error, t('providerForm.actionFailed', { action })), t('providerForm.actionFailedTitle', { action }))
  } finally {
    loading.value = false
  }
}
</script>
