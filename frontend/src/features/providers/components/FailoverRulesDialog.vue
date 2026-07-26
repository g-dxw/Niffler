<template>
  <Dialog
    :model-value="open"
    :title="t('failoverRules.title')"
    :description="t('failoverRules.description')"
    :icon="GitBranch"
    size="lg"
    @update:model-value="handleClose"
  >
    <div class="space-y-5 max-h-[60vh] overflow-y-auto px-0.5 py-0.5 -mx-0.5">
      <!-- 成功转移规则 -->
      <div class="space-y-3">
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <h3 class="text-sm font-medium">
              {{ t('failoverRules.successRules') }}
            </h3>
            <p class="text-xs text-muted-foreground mt-0.5">
              {{ t('failoverRules.successHint') }}
            </p>
          </div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            class="shrink-0"
            @click="addRule('success')"
          >
            <Plus class="w-4 h-4 mr-1" />
            {{ t('failoverRules.add') }}
          </Button>
        </div>

        <div
          v-if="successPatterns.length === 0"
          class="text-xs text-muted-foreground px-3 py-4 border border-dashed rounded-lg text-center"
        >
          {{ t('failoverRules.empty') }}
        </div>

        <div
          v-for="(rule, index) in successPatterns"
          :key="'s-' + index"
          class="flex items-center gap-1"
        >
          <Input
            v-model="rule.pattern"
            :placeholder="t('failoverRules.successPlaceholder')"
            size="sm"
            class="font-mono text-xs flex-1"
          />
          <Button
            variant="ghost"
            size="sm"
            class="shrink-0 h-8 w-8 p-0 text-muted-foreground hover:text-destructive"
            @click="removeRule('success', index)"
          >
            <Trash2 class="w-3.5 h-3.5" />
          </Button>
        </div>
      </div>

      <!-- 错误终止规则 -->
      <div class="space-y-3">
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <h3 class="text-sm font-medium">
              {{ t('failoverRules.errorRules') }}
            </h3>
            <p class="text-xs text-muted-foreground mt-0.5">
              {{ t('failoverRules.errorHint') }}
            </p>
          </div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            class="shrink-0"
            @click="addRule('error')"
          >
            <Plus class="w-4 h-4 mr-1" />
            {{ t('failoverRules.add') }}
          </Button>
        </div>

        <div
          v-if="errorPatterns.length === 0"
          class="text-xs text-muted-foreground px-3 py-4 border border-dashed rounded-lg text-center"
        >
          {{ t('failoverRules.empty') }}
        </div>

        <div
          v-for="(rule, index) in errorPatterns"
          :key="'e-' + index"
          class="flex items-center gap-1"
        >
          <Input
            v-model="statusCodeInputs[index]"
            :placeholder="t('failoverRules.statusPlaceholder')"
            size="sm"
            class="font-mono text-xs w-28 shrink-0"
          />
          <Input
            v-model="rule.pattern"
            :placeholder="t('failoverRules.errorPlaceholder')"
            size="sm"
            class="font-mono text-xs flex-1"
          />
          <Button
            variant="ghost"
            size="sm"
            class="shrink-0 h-8 w-8 p-0 text-muted-foreground hover:text-destructive"
            @click="removeRule('error', index)"
          >
            <Trash2 class="w-3.5 h-3.5" />
          </Button>
        </div>
      </div>
    </div>

    <template #footer>
      <Button
        variant="outline"
        :disabled="saving"
        @click="handleClose"
      >
        {{ t('failoverRules.cancel') }}
      </Button>
      <Button
        :disabled="saving"
        @click="handleSave"
      >
        {{ saving ? t('failoverRules.saving') : t('failoverRules.save') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Dialog,
  Button,
  Input,
} from '@/components/ui'
import { GitBranch, Plus, Trash2 } from 'lucide-vue-next'
import { useToast } from '@/composables/useToast'
import { updateProvider, type ProviderWithEndpointsSummary } from '@/api/endpoints'
import { parseApiError } from '@/utils/errorParser'
import type { FailoverRuleItem } from '@/api/endpoints/types'

const props = defineProps<{
  open: boolean
  provider: ProviderWithEndpointsSummary | null
}>()
const { t } = useI18n()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': []
}>()

const { success, error: showError } = useToast()
const saving = ref(false)

const successPatterns = ref<FailoverRuleItem[]>([])
const errorPatterns = ref<FailoverRuleItem[]>([])
const statusCodeInputs = ref<string[]>([])

watch(() => [props.open, props.provider], () => {
  if (props.open && props.provider) {
    const rules = props.provider.failover_rules
    successPatterns.value = (rules?.success_failover_patterns || []).map(r => ({ ...r }))
    errorPatterns.value = (rules?.error_stop_patterns || []).map(r => ({ ...r }))
    statusCodeInputs.value = errorPatterns.value.map(r =>
      r.status_codes?.length ? r.status_codes.join(',') : ''
    )
  }
}, { immediate: true })

function addRule(type: 'success' | 'error') {
  const rule: FailoverRuleItem = { pattern: '', description: '' }
  if (type === 'success') {
    successPatterns.value.push(rule)
  } else {
    errorPatterns.value.push(rule)
    statusCodeInputs.value.push('')
  }
}

function removeRule(type: 'success' | 'error', index: number) {
  if (type === 'success') {
    successPatterns.value.splice(index, 1)
  } else {
    errorPatterns.value.splice(index, 1)
    statusCodeInputs.value.splice(index, 1)
  }
}

function handleClose() {
  emit('update:open', false)
}

function parseStatusCodes(input: string): { valid: true; codes?: number[] } | { valid: false; reason: string } {
  const trimmed = input.trim()
  if (!trimmed) return { valid: true }
  const parts = trimmed.split(/[,\s]+/)
  const codes: number[] = []
  for (const part of parts) {
    if (!part) continue
    if (!/^\d+$/.test(part)) return { valid: false, reason: t('failoverRules.notNumber', { value: part }) }
    const n = parseInt(part, 10)
    if (n < 100 || n > 599) return { valid: false, reason: t('failoverRules.outOfRange', { value: n }) }
    codes.push(n)
  }
  return { valid: true, codes: codes.length > 0 ? codes : undefined }
}

function validatePattern(pattern: string): string | null {
  if (!pattern.trim()) return t('failoverRules.patternRequired')
  try {
    new RegExp(pattern)
    return null
  } catch {
    return t('failoverRules.invalidPattern', { pattern })
  }
}

async function handleSave() {
  if (!props.provider) return

  // Validate patterns
  const allPatterns = [...successPatterns.value, ...errorPatterns.value]
  for (const rule of allPatterns) {
    const err = validatePattern(rule.pattern)
    if (err) {
      showError(err, t('failoverRules.validationFailed'))
      return
    }
  }

  // Parse and validate status codes from raw inputs
  for (let i = 0; i < errorPatterns.value.length; i++) {
    const raw = statusCodeInputs.value[i]?.trim() || ''
    const result = parseStatusCodes(raw)
    if (!result.valid) {
      showError(t('failoverRules.statusInvalid', { reason: result.reason }), t('failoverRules.validationFailed'))
      return
    }
    errorPatterns.value[i].status_codes = result.codes
  }

  saving.value = true
  try {
    const filteredSuccess = successPatterns.value.filter(r => r.pattern.trim())
    const filteredError = errorPatterns.value.filter(r => r.pattern.trim())

    const hasRules = filteredSuccess.length > 0 || filteredError.length > 0

    await updateProvider(props.provider.id, {
      failover_rules: hasRules
        ? {
            success_failover_patterns: filteredSuccess,
            error_stop_patterns: filteredError,
          }
        : null,
    })

    success(t('failoverRules.saved'))
    emit('saved')
    handleClose()
  } catch (err: unknown) {
    showError(parseApiError(err, t('failoverRules.saveFailed')), t('failoverRules.saveFailedTitle'))
  } finally {
    saving.value = false
  }
}
</script>
