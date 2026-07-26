<template>
  <CardSection
    :title="t('emailDeliveryHistory.title')"
    :description="t('emailDeliveryHistory.description')"
  >
    <template #actions>
      <Button
        size="sm"
        variant="outline"
        :disabled="loading"
        @click="loadHistory"
      >
        {{ loading ? t('emailDeliveryHistory.refreshing') : t('emailDeliveryHistory.refresh') }}
      </Button>
    </template>

    <div
      v-if="loading && items.length === 0"
      class="py-8 text-center text-sm text-muted-foreground"
    >
      {{ t('emailDeliveryHistory.loading') }}
    </div>

    <div
      v-else-if="loadError"
      class="rounded-lg border border-destructive/30 bg-destructive/5 px-4 py-3 text-sm text-destructive"
    >
      {{ loadError }}
    </div>

    <div
      v-else-if="items.length === 0"
      class="py-8 text-center text-sm text-muted-foreground"
    >
      {{ t('emailDeliveryHistory.empty') }}
    </div>

    <Table
      v-else
      class="w-full min-w-[820px] table-fixed"
    >
      <colgroup>
        <col :style="{ width: emailHistoryColumnWidths.time }">
        <col :style="{ width: emailHistoryColumnWidths.type }">
        <col :style="{ width: emailHistoryColumnWidths.recipient }">
        <col :style="{ width: emailHistoryColumnWidths.status }">
        <col :style="{ width: emailHistoryColumnWidths.error }">
      </colgroup>
      <TableHeader>
        <TableRow>
          <SortableTableHead :sortable="false" resize-column-key="time" :resizable="true" @resize-start="handleEmailHistoryColumnResizeStart">
            {{ t('emailDeliveryHistory.time') }}
          </SortableTableHead>
          <SortableTableHead :sortable="false" resize-column-key="type" :resizable="true" @resize-start="handleEmailHistoryColumnResizeStart">
            {{ t('emailDeliveryHistory.type') }}
          </SortableTableHead>
          <SortableTableHead :sortable="false" resize-column-key="recipient" :resizable="true" @resize-start="handleEmailHistoryColumnResizeStart">
            {{ t('emailDeliveryHistory.recipient') }}
          </SortableTableHead>
          <SortableTableHead :sortable="false" resize-column-key="status" :resizable="true" @resize-start="handleEmailHistoryColumnResizeStart">
            {{ t('emailDeliveryHistory.status') }}
          </SortableTableHead>
          <SortableTableHead :sortable="false" resize-column-key="error" :resizable="true" @resize-start="handleEmailHistoryColumnResizeStart">
            {{ t('emailDeliveryHistory.failureReason') }}
          </SortableTableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow
          v-for="item in items"
          :key="item.id"
        >
          <TableCell class="whitespace-nowrap text-muted-foreground">
            {{ formatDate(item.created_at) }}
          </TableCell>
          <TableCell>
            {{ formatMessageType(resultOf(item).message_type) }}
          </TableCell>
          <TableCell class="font-mono text-xs">
            {{ resultOf(item).to_email || '-' }}
          </TableCell>
          <TableCell>
            <Badge :variant="statusVariant(item.status)">
              {{ statusLabel(item.status) }}
            </Badge>
          </TableCell>
          <TableCell
            class="whitespace-pre-wrap break-words text-muted-foreground"
            :title="item.error_message || '-'"
          >
            {{ item.error_message || '-' }}
          </TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </CardSection>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { CardSection } from '@/components/layout'
import { useResizableTableColumns, type ResizableTableColumn } from '@/composables/useResizableTableColumns'
import {
  Badge,
  Button,
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
  SortableTableHead,
} from '@/components/ui'
import { asyncTasksApi, type AsyncTaskItem, type AsyncTaskStatus } from '@/api/async-tasks'
import { parseApiError } from '@/utils/errorParser'

interface EmailDeliveryResult {
  message_type?: string | null
  to_email?: string | null
}

const items = ref<AsyncTaskItem[]>([])
const { t, locale } = useI18n()
const loading = ref(false)
const loadError = ref('')
type EmailHistoryColumnKey = 'time' | 'type' | 'recipient' | 'status' | 'error'
const emailHistoryColumns: ResizableTableColumn<EmailHistoryColumnKey>[] = [
  { key: 'time', width: '150px', minWidth: 140 },
  { key: 'type', width: '140px', minWidth: 120 },
  { key: 'recipient', width: '190px', minWidth: 160 },
  { key: 'status', width: '110px', minWidth: 100 },
  { key: 'error', width: '230px', minWidth: 200 },
]
const {
  columnWidths: emailHistoryColumnWidths,
  startResize: handleEmailHistoryColumnResizeStart,
} = useResizableTableColumns<EmailHistoryColumnKey>({
  storageKey: 'email-delivery-history-table-column-widths',
  columns: emailHistoryColumns,
  defaultMinWidth: 96,
})

onMounted(() => {
  void loadHistory()
})

async function loadHistory() {
  loading.value = true
  loadError.value = ''
  try {
    const response = await asyncTasksApi.list({
      trigger: 'auth_email',
      page_size: 10,
    })
    items.value = response.items
  } catch (err) {
    loadError.value = parseApiError(err, t('emailDeliveryHistory.loadFailed'))
  } finally {
    loading.value = false
  }
}

function resultOf(item: AsyncTaskItem): EmailDeliveryResult {
  if (!item.result || typeof item.result !== 'object') {
    return {}
  }
  return item.result as EmailDeliveryResult
}

function formatMessageType(type: string | null | undefined): string {
  if (type === 'verification') return t('emailDeliveryHistory.verification')
  if (type === 'password_reset') return t('emailDeliveryHistory.passwordReset')
  if (type === 'test') return t('emailDeliveryHistory.testEmail')
  return type || '-'
}

function statusLabel(status: AsyncTaskStatus): string {
  if (status === 'queued' || status === 'pending' || status === 'submitted') return t('emailDeliveryHistory.waiting')
  if (status === 'running' || status === 'processing') return t('emailDeliveryHistory.sending')
  if (status === 'retrying') return t('emailDeliveryHistory.waitingRetry')
  if (status === 'succeeded' || status === 'completed') return t('emailDeliveryHistory.sent')
  if (status === 'failed') return t('emailDeliveryHistory.failed')
  if (status === 'cancelled') return t('emailDeliveryHistory.cancelled')
  if (status === 'skipped') return t('emailDeliveryHistory.skipped')
  return status
}

function statusVariant(
  status: AsyncTaskStatus
): 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning' {
  if (status === 'succeeded' || status === 'completed') return 'success'
  if (status === 'failed') return 'destructive'
  if (status === 'running' || status === 'processing' || status === 'retrying') return 'warning'
  if (status === 'cancelled' || status === 'skipped') return 'outline'
  return 'secondary'
}

function formatDate(value: string | null | undefined): string {
  if (!value) return '-'
  return new Date(value).toLocaleString(locale.value, {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

defineExpose({
  refresh: loadHistory,
})
</script>
