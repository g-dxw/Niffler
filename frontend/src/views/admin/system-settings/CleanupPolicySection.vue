<template>
  <CardSection
    :title="t('cleanupPolicy.title')"
    :description="t('cleanupPolicy.description')"
  >
    <template #actions>
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <Switch
            id="enable-auto-cleanup"
            :model-value="enableAutoCleanup"
            @update:model-value="$emit('toggleAutoCleanup', $event)"
          />
          <div>
            <Label
              for="enable-auto-cleanup"
              class="text-sm cursor-pointer"
            >
              {{ t('cleanupPolicy.enable') }}
            </Label>
            <p class="text-xs text-muted-foreground">
              {{ t('cleanupPolicy.daily') }}
            </p>
          </div>
        </div>
        <Button
          variant="destructive"
          size="sm"
          :disabled="manualCleanupRunning"
          @click="openManualCleanupDialog"
        >
          <Trash2 class="w-3.5 h-3.5 mr-1.5" />
          {{ manualCleanupRunning ? t('cleanupPolicy.cleaning') : t('cleanupPolicy.cleanNow') }}
        </Button>
        <Button
          size="sm"
          :disabled="loading || !hasChanges"
          @click="$emit('save')"
        >
          {{ loading ? t('cleanupPolicy.saving') : t('cleanupPolicy.save') }}
        </Button>
      </div>
    </template>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div>
        <Label
          for="detail-log-retention-days"
          class="block text-sm font-medium"
        >
          {{ t('cleanupPolicy.details') }}
        </Label>
        <Input
          id="detail-log-retention-days"
          :model-value="detailLogRetentionDays"
          type="number"
          placeholder="1"
          class="mt-1"
          @update:model-value="$emit('update:detailLogRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('cleanupPolicy.detailHint') }}
        </p>
      </div>

      <div>
        <Label
          for="compressed-log-retention-days"
          class="block text-sm font-medium"
        >
          {{ t('cleanupPolicy.compressed') }}
        </Label>
        <Input
          id="compressed-log-retention-days"
          :model-value="compressedLogRetentionDays"
          type="number"
          placeholder="2"
          class="mt-1"
          @update:model-value="$emit('update:compressedLogRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('cleanupPolicy.compressedHint') }}
        </p>
      </div>

      <div>
        <Label
          for="header-retention-days"
          class="block text-sm font-medium"
        >
          {{ t('cleanupPolicy.headers') }}
        </Label>
        <Input
          id="header-retention-days"
          :model-value="headerRetentionDays"
          type="number"
          placeholder="30"
          class="mt-1"
          @update:model-value="$emit('update:headerRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('cleanupPolicy.headersHint') }}
        </p>
      </div>

      <div>
        <Label
          for="log-retention-days"
          class="block text-sm font-medium"
        >
          {{ t('cleanupPolicy.full') }}
        </Label>
        <Input
          id="log-retention-days"
          :model-value="logRetentionDays"
          type="number"
          placeholder="365"
          class="mt-1"
          @update:model-value="$emit('update:logRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('cleanupPolicy.fullHint') }}
        </p>
      </div>

      <div>
        <Label
          for="cleanup-batch-size"
          class="block text-sm font-medium"
        >
          {{ t('cleanupPolicy.batch') }}
        </Label>
        <Input
          id="cleanup-batch-size"
          :model-value="cleanupBatchSize"
          type="number"
          placeholder="1000"
          class="mt-1"
          @update:model-value="$emit('update:cleanupBatchSize', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('cleanupPolicy.batchHint') }}
        </p>
      </div>

      <div>
        <Label
          for="audit-log-retention-days"
          class="block text-sm font-medium"
        >
          {{ t('cleanupPolicy.audit') }}
        </Label>
        <Input
          id="audit-log-retention-days"
          :model-value="auditLogRetentionDays"
          type="number"
          placeholder="30"
          class="mt-1"
          @update:model-value="$emit('update:auditLogRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('cleanupPolicy.auditHint') }}
        </p>
      </div>

      <div>
        <Label
          for="request-candidates-retention-days"
          class="block text-sm font-medium"
        >
          {{ t('cleanupPolicy.candidates') }}
        </Label>
        <Input
          id="request-candidates-retention-days"
          :model-value="requestCandidatesRetentionDays"
          type="number"
          placeholder="30"
          class="mt-1"
          @update:model-value="$emit('update:requestCandidatesRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('cleanupPolicy.candidatesHint') }}
        </p>
      </div>

      <div>
        <Label
          for="request-candidates-cleanup-batch-size"
          class="block text-sm font-medium"
        >
          {{ t('cleanupPolicy.candidatesBatch') }}
        </Label>
        <Input
          id="request-candidates-cleanup-batch-size"
          :model-value="requestCandidatesCleanupBatchSize"
          type="number"
          placeholder="5000"
          class="mt-1"
          @update:model-value="$emit('update:requestCandidatesCleanupBatchSize', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('cleanupPolicy.candidatesBatchHint') }}
        </p>
      </div>

      <div>
        <Label
          for="proxy-node-metrics-1m-retention-days"
          class="block text-sm font-medium"
        >
          {{ t('cleanupPolicy.metrics1m') }}
        </Label>
        <Input
          id="proxy-node-metrics-1m-retention-days"
          :model-value="proxyNodeMetrics1mRetentionDays"
          type="number"
          min="1"
          max="365"
          placeholder="30"
          class="mt-1"
          @update:model-value="$emit('update:proxyNodeMetrics1mRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('cleanupPolicy.metrics1mHint') }}
        </p>
      </div>

      <div>
        <Label
          for="proxy-node-metrics-1h-retention-days"
          class="block text-sm font-medium"
        >
          {{ t('cleanupPolicy.metrics1h') }}
        </Label>
        <Input
          id="proxy-node-metrics-1h-retention-days"
          :model-value="proxyNodeMetrics1hRetentionDays"
          type="number"
          min="1"
          max="1095"
          placeholder="180"
          class="mt-1"
          @update:model-value="$emit('update:proxyNodeMetrics1hRetentionDays', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('cleanupPolicy.metrics1hHint') }}
        </p>
      </div>

      <div>
        <Label
          for="proxy-node-metrics-cleanup-batch-size"
          class="block text-sm font-medium"
        >
          {{ t('cleanupPolicy.metricsBatch') }}
        </Label>
        <Input
          id="proxy-node-metrics-cleanup-batch-size"
          :model-value="proxyNodeMetricsCleanupBatchSize"
          type="number"
          min="1"
          max="50000"
          placeholder="5000"
          class="mt-1"
          @update:model-value="$emit('update:proxyNodeMetricsCleanupBatchSize', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('cleanupPolicy.metricsBatchHint') }}
        </p>
      </div>
    </div>

    <!-- 清理策略说明 -->
    <div class="mt-4 p-4 bg-muted/50 rounded-lg">
      <h4 class="text-sm font-medium mb-2">
        {{ t('cleanupPolicy.explanation') }}
      </h4>
      <div class="text-xs text-muted-foreground space-y-1">
        <p>1. <strong>{{ t('cleanupPolicy.detailStage') }}</strong>: {{ t('cleanupPolicy.detailStageText') }}</p>
        <p>2. <strong>{{ t('cleanupPolicy.compressedStage') }}</strong>: {{ t('cleanupPolicy.compressedStageText') }}</p>
        <p>3. <strong>{{ t('cleanupPolicy.statsStage') }}</strong>: {{ t('cleanupPolicy.statsStageText') }}</p>
        <p>4. <strong>{{ t('cleanupPolicy.archiveStage') }}</strong>: {{ t('cleanupPolicy.archiveStageText') }}</p>
        <p>5. <strong>{{ t('cleanupPolicy.candidateStage') }}</strong>: {{ t('cleanupPolicy.candidateStageText') }}</p>
        <p>6. <strong>{{ t('cleanupPolicy.auditStage') }}</strong>: {{ t('cleanupPolicy.auditStageText') }}</p>
        <p>7. <strong>{{ t('cleanupPolicy.metricsStage') }}</strong>: {{ t('cleanupPolicy.metricsStageText') }}</p>
      </div>
    </div>

    <ManualCleanupConfirmDialog
      :open="manualCleanupDialogOpen"
      @update:open="manualCleanupDialogOpen = $event"
      @running-change="manualCleanupRunning = $event"
      @completed="handleManualCleanupCompleted"
    />

    <div
      v-if="manualCleanupResult"
      class="mt-4 rounded-md border border-border bg-muted/30 px-4 py-3 text-sm"
    >
      <div class="font-medium">
        {{ manualCleanupResult.title }}
      </div>
      <div
        v-if="manualCleanupResult.description"
        class="mt-1 text-xs text-muted-foreground"
      >
        {{ manualCleanupResult.description }}
      </div>
    </div>

    <div class="mt-4 border border-border rounded-lg overflow-hidden">
      <div class="flex items-center justify-between px-4 py-3 border-b border-border">
        <div>
          <h4 class="text-sm font-medium">
            {{ t('cleanupPolicy.recentRuns') }}
          </h4>
          <p class="text-xs text-muted-foreground">
            {{ t('cleanupPolicy.recentRunsHint') }}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          :disabled="cleanupRunsLoading"
          @click="loadCleanupRuns"
        >
          <RefreshCw
            class="w-3.5 h-3.5 mr-1.5"
            :class="{ 'animate-spin': cleanupRunsLoading }"
          />
          {{ t('common.refresh') }}
        </Button>
      </div>
      <div
        v-if="cleanupRuns.length === 0 && !cleanupRunsLoading"
        class="px-4 py-6 text-sm text-muted-foreground"
      >
        {{ t('cleanupPolicy.noRuns') }}
      </div>
      <div
        v-else
        class="overflow-x-auto"
      >
        <table class="w-full text-sm">
          <thead class="bg-muted/30 text-xs text-muted-foreground">
            <tr>
              <th class="px-4 py-2 text-left font-medium">
                {{ t('common.time') }}
              </th>
              <th class="px-4 py-2 text-left font-medium">
                {{ t('common.type') }}
              </th>
              <th class="px-4 py-2 text-left font-medium">
                {{ t('cleanupPolicy.source') }}
              </th>
              <th class="px-4 py-2 text-left font-medium">
                {{ t('common.status') }}
              </th>
              <th class="px-4 py-2 text-left font-medium">
                {{ t('cleanupPolicy.result') }}
              </th>
              <th class="px-4 py-2 text-right font-medium">
                {{ t('cleanupPolicy.duration') }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="run in cleanupRuns"
              :key="run.id"
              class="border-t border-border"
            >
              <td class="px-4 py-2 whitespace-nowrap">
                {{ formatRunTime(run.started_at_unix_secs) }}
              </td>
              <td class="px-4 py-2 whitespace-nowrap">
                {{ cleanupKindLabel(run.kind) }}
              </td>
              <td class="px-4 py-2 whitespace-nowrap text-muted-foreground">
                {{ run.trigger === 'manual' ? t('cleanupPolicy.manual') : t('cleanupPolicy.automatic') }}
              </td>
              <td class="px-4 py-2 whitespace-nowrap">
                <span :class="cleanupStatusClass(run.status)">
                  {{ cleanupStatusLabel(run.status) }}
                </span>
              </td>
              <td class="px-4 py-2 min-w-[18rem]">
                <div>{{ run.error || run.message }}</div>
                <div class="text-xs text-muted-foreground">
                  {{ cleanupSummaryText(run.summary) }}
                </div>
              </td>
              <td class="px-4 py-2 text-right whitespace-nowrap text-muted-foreground">
                {{ formatDuration(run.duration_ms) }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </CardSection>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t, locale } = useI18n()
import { RefreshCw, Trash2 } from 'lucide-vue-next'
import { adminApi, type CleanupRunRecord } from '@/api/admin'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import Switch from '@/components/ui/switch.vue'
import { CardSection } from '@/components/layout'
import ManualCleanupConfirmDialog from './ManualCleanupConfirmDialog.vue'
import { useToast } from '@/composables/useToast'

defineProps<{
  enableAutoCleanup: boolean
  detailLogRetentionDays: number
  compressedLogRetentionDays: number
  headerRetentionDays: number
  logRetentionDays: number
  cleanupBatchSize: number
  auditLogRetentionDays: number
  requestCandidatesRetentionDays: number
  requestCandidatesCleanupBatchSize: number
  proxyNodeMetrics1mRetentionDays: number
  proxyNodeMetrics1hRetentionDays: number
  proxyNodeMetricsCleanupBatchSize: number
  loading: boolean
  hasChanges: boolean
}>()

defineEmits<{
  save: []
  toggleAutoCleanup: [enabled: boolean]
  'update:detailLogRetentionDays': [value: number]
  'update:compressedLogRetentionDays': [value: number]
  'update:headerRetentionDays': [value: number]
  'update:logRetentionDays': [value: number]
  'update:cleanupBatchSize': [value: number]
  'update:auditLogRetentionDays': [value: number]
  'update:requestCandidatesRetentionDays': [value: number]
  'update:requestCandidatesCleanupBatchSize': [value: number]
  'update:proxyNodeMetrics1mRetentionDays': [value: number]
  'update:proxyNodeMetrics1hRetentionDays': [value: number]
  'update:proxyNodeMetricsCleanupBatchSize': [value: number]
}>()

const cleanupRuns = ref<CleanupRunRecord[]>([])
const cleanupRunsLoading = ref(false)
let cleanupRunsTimer: ReturnType<typeof window.setInterval> | null = null

const manualCleanupDialogOpen = ref(false)
const manualCleanupRunning = ref(false)
const manualCleanupResult = ref<{ title: string; description?: string } | null>(null)
const toast = useToast()

function openManualCleanupDialog() {
  if (manualCleanupRunning.value) return
  manualCleanupDialogOpen.value = true
}

function handleManualCleanupCompleted(task: CleanupRunRecord) {
  manualCleanupRunning.value = false
  manualCleanupResult.value = {
    title: task.message,
    description: cleanupSummaryText(task.summary),
  }
  if (task.status === 'failed') {
    toast.error(task.error || task.message)
  } else {
    toast.success(task.message)
  }
  void loadCleanupRuns()
}

async function loadCleanupRuns() {
  cleanupRunsLoading.value = true
  try {
    const response = await adminApi.getCleanupRuns()
    cleanupRuns.value = response.items.slice(0, 10)
  } finally {
    cleanupRunsLoading.value = false
  }
}

function cleanupKindLabel(kind: string): string {
  const labels: Record<string, string> = {
    usage_cleanup: t('cleanupPolicy.kindUsage'),
    audit_cleanup: t('cleanupPolicy.kindAudit'),
    request_candidate_cleanup: t('cleanupPolicy.kindCandidates'),
    request_bodies: t('cleanupPolicy.kindBodies'),
    config_purge: t('cleanupPolicy.kindConfig'),
    users_purge: t('cleanupPolicy.kindUsers'),
    usage_purge: t('cleanupPolicy.kindUsagePurge'),
    audit_logs_purge: t('cleanupPolicy.kindAuditPurge'),
    stats_purge: t('cleanupPolicy.kindStats'),
    system_cleanup: t('cleanupPolicy.kindSystem'),
  }
  return labels[kind] || kind
}

function cleanupStatusLabel(status: string): string {
  if (status === 'processing') return t('manualCleanup.running')
  if (status === 'failed') return t('manualCleanup.failed')
  return t('manualCleanup.complete')
}

function cleanupStatusClass(status: string): string {
  if (status === 'processing') return 'text-amber-500'
  if (status === 'failed') return 'text-destructive'
  return 'text-emerald-500'
}

function formatRunTime(value: number): string {
  if (!value) return '-'
  return new Date(value * 1000).toLocaleString(locale.value)
}

function formatDuration(value: number | null): string {
  if (value === null || value === undefined) return '-'
  if (value < 1000) return `${value}ms`
  return `${(value / 1000).toFixed(1)}s`
}

function cleanupSummaryText(summary: Record<string, unknown>): string {
  const total = typeof summary.total === 'number' ? summary.total : null
  if (total !== null) return t('cleanupPolicy.affectedRows', { total })

  const entries = Object.entries(summary)
    .filter(([, value]) => typeof value === 'number' && value > 0)
    .map(([key, value]) => `${summaryLabel(key)} ${value}`)
  return entries.length > 0 ? entries.join(' / ') : t('cleanupPolicy.noChanges')
}

function summaryLabel(key: string): string {
  const labels: Record<string, string> = {
    body_externalized: t('cleanupPolicy.summaryBodyExternalized'),
    legacy_body_refs_migrated: t('cleanupPolicy.summaryMigrated'),
    body_cleaned: t('cleanupPolicy.summaryBodyCleaned'),
    header_cleaned: t('cleanupPolicy.summaryHeaderCleaned'),
    keys_cleaned: 'Key',
    records_deleted: t('cleanupPolicy.summaryRecordsDeleted'),
    audit_logs_deleted: t('cleanupPolicy.summaryAuditDeleted'),
    request_candidates_deleted: t('cleanupPolicy.summaryCandidatesDeleted'),
  }
  return labels[key] || key
}

onMounted(() => {
  void loadCleanupRuns()
  cleanupRunsTimer = window.setInterval(() => {
    void loadCleanupRuns()
  }, 15_000)
})

onBeforeUnmount(() => {
  if (cleanupRunsTimer) {
    window.clearInterval(cleanupRunsTimer)
    cleanupRunsTimer = null
  }
})
</script>
