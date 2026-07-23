<template>
  <Dialog
    :open="open"
    size="lg"
    :title="t('manualCleanup.title')"
    :description="t('manualCleanup.description')"
    :persistent="isLocked"
    @update:open="handleOpenChange"
  >
    <div class="px-4 sm:px-6 py-4 space-y-4">
      <div>
        <Label class="block text-sm font-medium">
          {{ t('manualCleanup.method') }}
        </Label>
        <div class="mt-2 grid grid-cols-1 sm:grid-cols-3 gap-2">
          <button
            v-for="item in modeOptions"
            :key="item.value"
            type="button"
            class="rounded-md border px-3 py-2 text-left text-sm transition-colors"
            :class="mode === item.value ? 'border-primary bg-primary/10 text-primary' : 'border-border bg-card hover:bg-muted/60'"
            :disabled="isLocked"
            @click="setMode(item.value)"
          >
            <span class="font-medium">{{ item.label }}</span>
            <span class="mt-1 block text-xs text-muted-foreground">{{ item.description }}</span>
          </button>
        </div>
      </div>

      <div v-if="mode === 'older_than_days'">
        <Label
          for="manual-cleanup-older-than-days"
          class="block text-sm font-medium"
        >
          {{ t('manualCleanup.olderThan') }}
        </Label>
        <Input
          id="manual-cleanup-older-than-days"
          :model-value="olderThanDays ?? ''"
          type="number"
          min="1"
          :placeholder="t('manualCleanup.placeholderDays')"
          class="mt-1"
          :disabled="isLocked"
          @update:model-value="handleDaysChange"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('manualCleanup.conservative') }}
        </p>
      </div>

      <div>
        <Label class="block text-sm font-medium">
          {{ t('manualCleanup.scope') }}
        </Label>
        <div class="mt-2 grid grid-cols-1 sm:grid-cols-2 gap-2">
          <label
            v-for="target in targetOptions"
            :key="target.value"
            class="flex min-h-16 items-start gap-3 rounded-md border border-border bg-card px-3 py-2"
          >
            <Checkbox
              class="mt-0.5"
              :checked="selectedTargets.includes(target.value)"
              :disabled="isLocked"
              @update:checked="toggleTarget(target.value, $event)"
            />
            <span>
              <span class="block text-sm font-medium">{{ target.label }}</span>
              <span class="block text-xs text-muted-foreground">{{ target.description }}</span>
            </span>
          </label>
        </div>
        <p
          v-if="mode === 'before_now'"
          class="mt-2 text-xs text-amber-600"
        >
          {{ t('manualCleanup.beforeNowHint') }}
        </p>
        <p
          v-if="targetError"
          class="mt-2 text-xs text-destructive"
        >
          {{ targetError }}
        </p>
      </div>

      <div class="rounded-md border border-border bg-muted/30 px-4 py-3">
        <div class="flex items-center justify-between">
          <h4 class="text-sm font-medium">
            {{ t('manualCleanup.impact') }}
          </h4>
          <button
            v-if="!previewLoading"
            type="button"
            class="text-xs text-muted-foreground hover:text-foreground"
            :disabled="isLocked"
            @click="loadPreview"
          >
            {{ t('manualCleanup.refresh') }}
          </button>
          <span
            v-else
            class="text-xs text-muted-foreground"
          >
            {{ t('manualCleanup.calculating') }}
          </span>
        </div>
        <div
          v-if="previewError"
          class="mt-2 text-xs text-destructive"
        >
          {{ previewError }}
        </div>
        <div
          v-else-if="preview"
          class="mt-2 grid grid-cols-2 gap-y-1 gap-x-4 text-xs text-muted-foreground"
        >
          <div>{{ t('manualCleanup.detail') }}</div>
          <div class="text-right text-foreground">
            {{ formatCount(preview.counts.detail) }}
          </div>
          <div>{{ t('manualCleanup.compressed') }}</div>
          <div class="text-right text-foreground">
            {{ formatCount(preview.counts.compressed) }}
          </div>
          <div>{{ t('manualCleanup.headers') }}</div>
          <div class="text-right text-foreground">
            {{ formatCount(preview.counts.header) }}
          </div>
          <div>{{ t('manualCleanup.records') }}</div>
          <div class="text-right text-destructive font-medium">
            {{ formatCount(preview.counts.log) }}
          </div>
        </div>
        <div
          v-else-if="!previewLoading"
          class="mt-2 text-xs text-muted-foreground"
        >
          {{ t('manualCleanup.noEstimate') }}
        </div>
      </div>

      <div
        v-if="activeTask || taskError"
        class="rounded-md border border-border bg-card px-4 py-3"
      >
        <div class="flex items-center justify-between gap-3">
          <div class="min-w-0">
            <div class="text-sm font-medium">
              {{ activeTask?.message || t('manualCleanup.startFailed') }}
            </div>
            <div class="mt-1 text-xs text-muted-foreground">
              {{ activeTask ? cleanupStatusLabel(activeTask.status) : taskError }}
            </div>
          </div>
          <span
            v-if="activeTask"
            :class="cleanupStatusClass(activeTask.status)"
            class="shrink-0 text-xs"
          >
            {{ cleanupStatusLabel(activeTask.status) }}
          </span>
        </div>
        <div
          v-if="activeTask"
          class="mt-3 h-2 overflow-hidden rounded-full bg-muted"
        >
          <div
            class="h-full rounded-full bg-primary transition-all"
            :class="{ 'animate-pulse': activeTask.status === 'processing' }"
            :style="{ width: `${taskProgressPercent}%` }"
          />
        </div>
        <div
          v-if="activeTask"
          class="mt-2 text-xs text-muted-foreground"
        >
          {{ cleanupSummaryText(activeTask.summary) }}
        </div>
      </div>

      <div>
        <Label
          for="manual-cleanup-confirm-phrase"
          class="block text-sm font-medium"
        >
          {{ t('manualCleanup.confirmInput', { phrase: confirmPhrase }) }}
        </Label>
        <Input
          id="manual-cleanup-confirm-phrase"
          :model-value="typedPhrase"
          class="mt-1"
          autocomplete="off"
          :placeholder="confirmPhrase"
          :disabled="isLocked || isFinished"
          @update:model-value="typedPhrase = String($event)"
          @keydown.enter.prevent="maybeSubmitOnEnter"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('manualCleanup.lockedHint') }}
        </p>
      </div>
    </div>

    <template #footer>
      <Button
        v-if="!isFinished"
        variant="destructive"
        :disabled="!canSubmit"
        @click="handleConfirm"
      >
        {{ isLocked ? t('manualCleanup.processing') : t('manualCleanup.confirm') }}
      </Button>
      <Button
        variant="outline"
        :disabled="isLocked"
        @click="handleCancel"
      >
        {{ isFinished ? t('manualCleanup.close') : t('manualCleanup.cancel') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
import { Dialog } from '@/components/ui'
import Button from '@/components/ui/button.vue'
import Checkbox from '@/components/ui/checkbox.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import {
  adminApi,
  type CleanupRunRecord,
  type ManualUsageCleanupPreview,
  type ManualUsageCleanupRequest,
} from '@/api/admin'
import { parseApiError } from '@/utils/errorParser'
import {
  MANUAL_USAGE_CLEANUP_CONFIRM_PHRASE,
  allowedTargetsForMode,
  defaultManualCleanupTargets,
  isConfirmPhraseMatched,
  normalizeManualCleanupTargets,
  normalizeOlderThanDaysInput,
  type ManualCleanupMode,
  type ManualCleanupTarget,
} from './manualCleanupForm'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'running-change': [value: boolean]
  completed: [task: CleanupRunRecord]
}>()

const confirmPhrase = MANUAL_USAGE_CLEANUP_CONFIRM_PHRASE

const mode = ref<ManualCleanupMode>('policy')
const olderThanDays = ref<number | null>(null)
const selectedTargets = ref<ManualCleanupTarget[]>(defaultManualCleanupTargets('policy'))
const targetsTouched = ref(false)
const typedPhrase = ref('')
const preview = ref<ManualUsageCleanupPreview | null>(null)
const previewLoading = ref(false)
const previewError = ref<string | null>(null)
const submitting = ref(false)
const activeTask = ref<CleanupRunRecord | null>(null)
const taskError = ref<string | null>(null)

let previewDebounceTimer: ReturnType<typeof setTimeout> | null = null
let previewSeq = 0
let taskPollTimer: ReturnType<typeof window.setInterval> | null = null

const modeOptions = computed<Array<{ value: ManualCleanupMode; label: string; description: string }>>(() => [
  { value: 'policy', label: t('manualCleanup.modes.policy'), description: t('manualCleanup.modeHints.policy') },
  { value: 'older_than_days', label: t('manualCleanup.modes.olderThanDays'), description: t('manualCleanup.modeHints.olderThanDays') },
  { value: 'before_now', label: t('manualCleanup.modes.beforeNow'), description: t('manualCleanup.modeHints.beforeNow') },
])

const targetLabels = computed<Record<ManualCleanupTarget, { label: string; description: string }>>(() => ({
  detail_body: { label: t('manualCleanup.targets.detailLabel'), description: t('manualCleanup.targets.detailDescription') },
  compressed_body: { label: t('manualCleanup.targets.compressedLabel'), description: t('manualCleanup.targets.compressedDescription') },
  headers: { label: t('manualCleanup.targets.headersLabel'), description: t('manualCleanup.targets.headersDescription') },
  records: { label: t('manualCleanup.targets.recordsLabel'), description: t('manualCleanup.targets.recordsDescription') },
}))

const targetOptions = computed(() =>
  allowedTargetsForMode(mode.value).map(value => ({
    value,
    ...targetLabels.value[value],
  }))
)

const normalizedTargets = computed(() =>
  normalizeManualCleanupTargets(mode.value, selectedTargets.value)
)

const targetError = computed(() => {
  if (normalizedTargets.value.length === 0) return t('manualCleanup.targetRequired')
  return null
})

const currentTaskRunning = computed(() => activeTask.value?.status === 'processing')
const isLocked = computed(() => submitting.value || currentTaskRunning.value)
const isFinished = computed(() =>
  activeTask.value?.status === 'completed' || activeTask.value?.status === 'failed'
)

const canSubmit = computed(
  () =>
    !isLocked.value &&
    !isFinished.value &&
    !previewLoading.value &&
    !targetError.value &&
    modeIsValid.value &&
    isConfirmPhraseMatched(typedPhrase.value),
)

const modeIsValid = computed(() => mode.value !== 'older_than_days' || olderThanDays.value !== null)

const taskProgressPercent = computed(() => {
  const task = activeTask.value
  if (!task) return 0
  if (task.status === 'completed') return 100
  if (task.status === 'failed') return 100
  const raw = task.summary?.progress_percent
  if (typeof raw === 'number' && Number.isFinite(raw) && raw > 0) {
    return Math.max(1, Math.min(99, Math.round(raw)))
  }
  return 12
})

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      resetForm()
      void loadPreview()
    } else {
      clearPreviewTimer()
      stopTaskPolling()
    }
  },
)

function resetForm() {
  mode.value = 'policy'
  olderThanDays.value = null
  selectedTargets.value = defaultManualCleanupTargets('policy')
  targetsTouched.value = false
  typedPhrase.value = ''
  preview.value = null
  previewError.value = null
  previewLoading.value = false
  submitting.value = false
  activeTask.value = null
  taskError.value = null
  emit('running-change', false)
}

function setMode(nextMode: ManualCleanupMode) {
  if (isLocked.value || mode.value === nextMode) return
  mode.value = nextMode
  olderThanDays.value = null
  selectedTargets.value = defaultManualCleanupTargets(nextMode)
  targetsTouched.value = false
  activeTask.value = null
  taskError.value = null
  schedulePreview()
}

function handleDaysChange(value: string | number) {
  olderThanDays.value = normalizeOlderThanDaysInput(value)
  schedulePreview()
}

function toggleTarget(target: ManualCleanupTarget, checked: boolean) {
  targetsTouched.value = true
  activeTask.value = null
  taskError.value = null
  const current = new Set(selectedTargets.value)
  if (checked) {
    current.add(target)
  } else {
    current.delete(target)
  }
  selectedTargets.value = normalizeManualCleanupTargets(mode.value, Array.from(current))
  schedulePreview()
}

function buildRequest(): ManualUsageCleanupRequest {
  const request: ManualUsageCleanupRequest = { mode: mode.value }
  if (mode.value === 'older_than_days' && olderThanDays.value !== null) {
    request.older_than_days = olderThanDays.value
  }
  if (targetsTouched.value) {
    request.targets = normalizedTargets.value
  }
  return request
}

function schedulePreview() {
  clearPreviewTimer()
  previewDebounceTimer = setTimeout(() => {
    previewDebounceTimer = null
    void loadPreview()
  }, 300)
}

function clearPreviewTimer() {
  if (previewDebounceTimer) {
    clearTimeout(previewDebounceTimer)
    previewDebounceTimer = null
  }
}

async function loadPreview() {
  const seq = ++previewSeq
  previewLoading.value = true
  previewError.value = null
  try {
    const result = await adminApi.previewManualUsageCleanup(buildRequest())
    if (seq === previewSeq) {
      preview.value = result
    }
  } catch (error) {
    if (seq === previewSeq) {
      preview.value = null
      previewError.value = parseApiError(error)
    }
  } finally {
    if (seq === previewSeq) {
      previewLoading.value = false
    }
  }
}

function handleOpenChange(value: boolean) {
  if (!value && isLocked.value) {
    return
  }
  emit('update:open', value)
}

function handleCancel() {
  if (isLocked.value) return
  emit('update:open', false)
}

function maybeSubmitOnEnter() {
  if (canSubmit.value) {
    void handleConfirm()
  }
}

async function handleConfirm() {
  if (!canSubmit.value) return
  submitting.value = true
  taskError.value = null
  try {
    const response = await adminApi.runManualUsageCleanup(buildRequest())
    if ('detail' in response && response.detail === 'usage_cleanup_already_running') {
      taskError.value = response.message
      emit('running-change', false)
      return
    }
    if (!('task' in response)) {
    taskError.value = t('manualCleanup.startFailed')
      emit('running-change', false)
      return
    }
    activeTask.value = response.task
    emit('running-change', response.task.status === 'processing')
    if (response.task.status === 'processing') {
      startTaskPolling(response.task.id)
    } else {
      emit('completed', response.task)
    }
  } catch (error) {
    taskError.value = parseApiError(error)
    emit('running-change', false)
  } finally {
    submitting.value = false
  }
}

function startTaskPolling(taskId: string) {
  stopTaskPolling()
  void pollTask(taskId)
  taskPollTimer = window.setInterval(() => {
    void pollTask(taskId)
  }, 1_500)
}

function stopTaskPolling() {
  if (taskPollTimer) {
    window.clearInterval(taskPollTimer)
    taskPollTimer = null
  }
}

async function pollTask(taskId: string) {
  try {
    const response = await adminApi.getCleanupRuns()
    const task = response.items.find(item => item.id === taskId)
    if (!task) return
    activeTask.value = task
    const running = task.status === 'processing'
    emit('running-change', running)
    if (!running) {
      stopTaskPolling()
      emit('completed', task)
      void loadPreview()
    }
  } catch (error) {
    taskError.value = parseApiError(error)
  }
}

function cleanupStatusLabel(status: string): string {
  if (status === 'processing') return t('manualCleanup.status.processing')
  if (status === 'failed') return t('manualCleanup.status.failed')
  return t('manualCleanup.status.completed')
}

function cleanupStatusClass(status: string): string {
  if (status === 'processing') return 'text-amber-500'
  if (status === 'failed') return 'text-destructive'
  return 'text-emerald-500'
}

function cleanupSummaryText(summary: Record<string, unknown>): string {
  const total = typeof summary.total === 'number' ? summary.total : null
  if (total !== null && total > 0) return t('manualCleanup.affectedCount', { count: total })
  const entries = Object.entries(summary)
    .filter(([key, value]) => key !== 'progress_percent' && typeof value === 'number' && value > 0)
    .map(([key, value]) => `${summaryLabel(key)} ${value}`)
  return entries.length > 0 ? entries.join(' / ') : t('manualCleanup.waiting')
}

function summaryLabel(key: string): string {
  const labels: Record<string, string> = {
    body_externalized: t('manualCleanup.events.bodyExternalized'),
    legacy_body_refs_migrated: t('manualCleanup.events.migrated'),
    body_cleaned: t('manualCleanup.events.bodyCleaned'),
    header_cleaned: t('manualCleanup.events.headersCleaned'),
    keys_cleaned: 'Key',
    records_deleted: t('manualCleanup.events.recordsDeleted'),
  }
  return labels[key] || key
}

function formatCount(value: number): string {
  return value.toLocaleString()
}

onBeforeUnmount(() => {
  clearPreviewTimer()
  stopTaskPolling()
})
</script>
