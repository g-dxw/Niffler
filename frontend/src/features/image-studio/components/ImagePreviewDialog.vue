<template>
  <Dialog
    :open="open"
    size="6xl"
    no-padding
    @update:open="emit('update:open', $event)"
  >
    <template #header>
      <div class="flex items-center justify-between border-b border-border/60 px-5 py-4">
        <div class="min-w-0">
          <h3 class="font-semibold">
            图片预览
          </h3>
          <p class="truncate text-xs text-muted-foreground">
            {{ task?.prompt }}
          </p>
        </div>
        <Button
          variant="ghost"
          size="icon"
          aria-label="关闭预览"
          @click="emit('update:open', false)"
        >
          <X class="h-4 w-4" />
        </Button>
      </div>
    </template>
    <div class="flex max-h-[78vh] min-h-72 items-center justify-center bg-black/90 p-3">
      <img
        v-if="task?.imageUrl"
        :src="task.imageUrl"
        :alt="task.prompt"
        class="max-h-[74vh] max-w-full object-contain"
      >
    </div>
    <template #footer>
      <Button
        v-if="task?.imageUrl"
        @click="emit('download', task)"
      >
        <Download class="mr-2 h-4 w-4" />下载图片
      </Button>
      <Button
        variant="outline"
        @click="emit('update:open', false)"
      >
        关闭
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { Download, X } from 'lucide-vue-next'
import { Button, Dialog } from '@/components/ui'
import type { ImageTask } from '../types'

defineProps<{ open: boolean, task: ImageTask | null }>()
const emit = defineEmits<{
  'update:open': [value: boolean]
  download: [task: ImageTask]
}>()
</script>
