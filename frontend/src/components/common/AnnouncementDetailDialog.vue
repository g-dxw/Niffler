<template>
  <Dialog v-model="open" size="lg">
    <template #header>
      <div class="border-b border-border px-6 py-4">
        <div class="flex items-center gap-3">
          <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg" :class="iconClass">
            <component :is="icon" class="h-5 w-5" :class="iconColor" />
          </div>
          <div class="min-w-0 flex-1">
            <h3 class="truncate text-lg font-semibold leading-tight">{{ announcement?.title || t('announcementDetail.title') }}</h3>
            <p class="text-xs text-muted-foreground">{{ t('announcementDetail.systemAnnouncement') }}</p>
          </div>
        </div>
      </div>
    </template>
    <div v-if="announcement" class="space-y-4">
      <div class="flex items-center gap-3 text-xs text-muted-foreground">
        <span>{{ announcement.author?.username || t('announcementDetail.system') }}</span><span>·</span><span>{{ formatDate(announcement.created_at) }}</span>
      </div>
      <div class="prose prose-sm dark:prose-invert max-w-none" v-html="renderMarkdown(announcement.content)" />
    </div>
    <template #footer><Button variant="outline" type="button" class="h-10 px-5" @click="open = false">{{ t('announcementDetail.close') }}</Button></template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { AlertCircle, AlertTriangle, Info, Wrench } from 'lucide-vue-next'
import { Button, Dialog } from '@/components/ui'
import type { Announcement } from '@/api/announcements'
import { marked } from 'marked'
import { sanitizeMarkdown } from '@/utils/sanitize'

const { t, locale } = useI18n()

const props = defineProps<{ modelValue: boolean; announcement: Announcement | null }>()
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()
const open = computed({ get: () => props.modelValue, set: value => emit('update:modelValue', value) })
const icon = computed(() => props.announcement?.type === 'warning' ? AlertTriangle : props.announcement?.type === 'maintenance' ? Wrench : props.announcement?.type === 'important' ? AlertCircle : Info)
const iconClass = computed(() => props.announcement?.type === 'warning' ? 'bg-amber-100 dark:bg-amber-900/30' : props.announcement?.type === 'important' ? 'bg-red-100 dark:bg-red-900/30' : 'bg-blue-100 dark:bg-blue-900/30')
const iconColor = computed(() => props.announcement?.type === 'warning' ? 'text-amber-600' : props.announcement?.type === 'important' ? 'text-red-600' : 'text-blue-600')
function formatDate(value: string) { return new Date(value).toLocaleString(locale.value) }
function renderMarkdown(content: string) { return sanitizeMarkdown(marked.parse(content) as string) }
</script>
