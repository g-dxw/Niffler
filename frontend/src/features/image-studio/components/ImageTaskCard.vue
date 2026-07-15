<template>
  <Card class="group overflow-hidden">
    <div class="relative aspect-square bg-muted/40">
      <button
        v-if="task.status === 'success' && task.imageUrl"
        type="button"
        class="h-full w-full"
        @click="emit('preview', task)"
      >
        <img
          :src="task.imageUrl"
          :alt="task.prompt"
          class="h-full w-full object-contain"
          loading="lazy"
        >
      </button>
      <div
        v-else
        class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center text-muted-foreground"
      >
        <Loader2
          v-if="task.status === 'running'"
          class="h-8 w-8 animate-spin text-primary"
        />
        <Clock3
          v-else-if="task.status === 'pending'"
          class="h-8 w-8"
        />
        <CircleX
          v-else-if="task.status === 'error'"
          class="h-8 w-8 text-destructive"
        />
        <Ban
          v-else
          class="h-8 w-8"
        />
        <p class="text-sm">
          {{ emptyLabel }}
        </p>
      </div>
      <Badge
        class="absolute left-3 top-3"
        :variant="statusVariant"
      >
        {{ statusLabel }}
      </Badge>
      <Badge
        v-if="task.mode === 'edit'"
        class="absolute right-3 top-3"
        variant="secondary"
      >
        图生图
      </Badge>
    </div>

    <div class="space-y-3 p-4">
      <div class="flex min-h-10 items-start gap-2">
        <p
          class="line-clamp-2 min-w-0 flex-1 text-sm leading-5"
          :title="task.prompt"
        >
          {{ task.prompt }}
        </p>
        <Button
          variant="ghost"
          size="sm"
          class="h-8 shrink-0 px-2 text-muted-foreground"
          title="复制提示词"
          aria-label="复制提示词"
          @click="copyToClipboard(task.prompt)"
        >
          <Copy class="h-3.5 w-3.5" />
        </Button>
      </div>
      <div class="flex flex-wrap gap-x-3 gap-y-1 text-[11px] text-muted-foreground">
        <span>{{ task.model }}</span><span>{{ task.size }}</span><span v-if="duration">耗时 {{ duration }}</span>
      </div>
      <p
        v-if="task.error"
        class="rounded-lg bg-destructive/10 px-3 py-2 text-xs text-destructive"
      >
        {{ task.error }}
      </p>
      <div class="flex items-center justify-end gap-1">
        <Button
          v-if="task.status === 'pending' || task.status === 'running'"
          variant="ghost"
          size="sm"
          @click="emit('cancel', task.id)"
        >
          <X class="mr-1 h-4 w-4" />取消
        </Button>
        <Button
          v-if="task.status === 'error' || task.status === 'cancelled'"
          variant="ghost"
          size="sm"
          @click="emit('retry', task.id)"
        >
          <RotateCcw class="mr-1 h-3.5 w-3.5" />重试
        </Button>
        <Button
          v-if="task.status === 'success' && task.imageUrl"
          variant="ghost"
          size="sm"
          @click="emit('download', task)"
        >
          <Download class="mr-1 h-3.5 w-3.5" />下载
        </Button>
        <Button
          variant="ghost"
          size="sm"
          class="text-muted-foreground hover:text-destructive"
          @click="emit('remove', task.id)"
        >
          <Trash2 class="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { Ban, CircleX, Clock3, Copy, Download, Loader2, RotateCcw, Trash2, X } from 'lucide-vue-next'
import { Badge, Button, Card } from '@/components/ui'
import { useClipboard } from '@/composables/useClipboard'
import type { ImageTask } from '../types'
import { formatTaskDuration } from '../utils/task-duration'

const props = defineProps<{ task: ImageTask }>()
const emit = defineEmits<{
  preview: [task: ImageTask]
  download: [task: ImageTask]
  cancel: [id: string]
  retry: [id: string]
  remove: [id: string]
}>()
const { copyToClipboard } = useClipboard()
const currentTime = ref(Date.now())
let timer: ReturnType<typeof setInterval> | undefined

function stopTimer() {
  if (timer === undefined) return
  clearInterval(timer)
  timer = undefined
}

watch(
  () => props.task.status,
  (status) => {
    stopTimer()
    if (status !== 'running') return
    currentTime.value = Date.now()
    timer = setInterval(() => {
      currentTime.value = Date.now()
    }, 100)
  },
  { immediate: true },
)

onBeforeUnmount(stopTimer)

const statusLabel = computed(() => ({ pending: '等待中', running: '生成中', success: '已完成', error: '失败', cancelled: '已取消' })[props.task.status])
const statusVariant = computed(() => ({ pending: 'secondary', running: 'warning', success: 'success', error: 'destructive', cancelled: 'outline' })[props.task.status])
const emptyLabel = computed(() => props.task.status === 'running' ? '正在生成图片…' : props.task.status === 'pending' ? '等待执行' : props.task.error || '暂无图片')
const duration = computed(() => formatTaskDuration(
  props.task.startedAt,
  props.task.finishedAt,
  currentTime.value,
))
</script>
