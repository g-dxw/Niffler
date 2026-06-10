<template>
  <CardSection
    title="最近发送记录"
    description="只显示邮件类型、脱敏收件人和发送结果"
  >
    <template #actions>
      <Button
        size="sm"
        variant="outline"
        :disabled="loading"
        @click="loadHistory"
      >
        {{ loading ? '刷新中...' : '刷新' }}
      </Button>
    </template>

    <div
      v-if="loading && items.length === 0"
      class="py-8 text-center text-sm text-muted-foreground"
    >
      正在加载发送记录...
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
      暂无邮件发送记录
    </div>

    <Table v-else>
      <TableHeader>
        <TableRow>
          <TableHead>时间</TableHead>
          <TableHead>类型</TableHead>
          <TableHead>收件人</TableHead>
          <TableHead>状态</TableHead>
          <TableHead>失败原因</TableHead>
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
          <TableCell class="max-w-[360px] truncate text-muted-foreground">
            {{ item.error_message || '-' }}
          </TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </CardSection>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { CardSection } from '@/components/layout'
import {
  Badge,
  Button,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui'
import { asyncTasksApi, type AsyncTaskItem, type AsyncTaskStatus } from '@/api/async-tasks'
import { parseApiError } from '@/utils/errorParser'

interface EmailDeliveryResult {
  message_type?: string | null
  to_email?: string | null
}

const items = ref<AsyncTaskItem[]>([])
const loading = ref(false)
const loadError = ref('')

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
    loadError.value = parseApiError(err, '加载发送记录失败')
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
  if (type === 'verification') return '注册验证码'
  if (type === 'password_reset') return '找回密码'
  if (type === 'test') return '测试邮件'
  return type || '-'
}

function statusLabel(status: AsyncTaskStatus): string {
  if (status === 'queued' || status === 'pending' || status === 'submitted') return '等待发送'
  if (status === 'running' || status === 'processing') return '发送中'
  if (status === 'retrying') return '等待重试'
  if (status === 'succeeded' || status === 'completed') return '已发送'
  if (status === 'failed') return '失败'
  if (status === 'cancelled') return '已取消'
  if (status === 'skipped') return '已跳过'
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
  return new Date(value).toLocaleString('zh-CN', {
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
