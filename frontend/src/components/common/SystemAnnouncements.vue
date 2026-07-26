<template>
  <div class="relative ml-1">
    <button
      type="button"
      class="relative flex h-9 w-9 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted/50 hover:text-foreground"
      :title="t('systemAnnouncements.title')"
      :aria-label="t('systemAnnouncements.title')"
      :aria-expanded="menuOpen"
      @click="menuOpen = !menuOpen"
    >
      <Megaphone class="h-4 w-4" />
      <span
        v-if="unreadCount > 0"
        class="absolute right-1 top-1 h-2 w-2 rounded-full bg-[#cc785c] ring-2 ring-[#faf9f5] dark:ring-[#191714]"
      />
    </button>

    <div
      v-if="menuOpen"
      class="fixed inset-x-3 top-16 z-[70] w-auto max-w-none overflow-hidden rounded-xl border border-[#3d3929]/10 bg-[#faf9f5] shadow-xl dark:border-white/10 dark:bg-[#1e1c19] lg:absolute lg:inset-x-auto lg:right-0 lg:top-[calc(100%+0.5rem)] lg:w-[22rem] lg:max-w-[calc(100vw-2rem)]"
    >
      <div class="border-b border-[#3d3929]/10 px-4 py-3 dark:border-white/10">
        <div class="flex items-center justify-between gap-3">
          <div>
            <div class="text-sm font-semibold text-foreground">{{ t('systemAnnouncements.title') }}</div>
            <div class="mt-0.5 text-xs text-muted-foreground">{{ t('systemAnnouncements.description') }}</div>
          </div>
          <span
            v-if="unreadCount > 0"
            class="rounded-full bg-[#cc785c]/10 px-2 py-0.5 text-[10px] font-medium text-[#a8533a] dark:bg-[#d4a27f]/10 dark:text-[#d4a27f]"
          >
            {{ t('systemAnnouncements.unread', { count: unreadCount }) }}
          </span>
        </div>
      </div>

      <div class="max-h-[calc(100vh-9rem)] overflow-y-auto lg:max-h-80">
        <div
          v-if="loading"
          class="px-4 py-8 text-center text-xs text-muted-foreground"
        >
          {{ t('common.loading') }}
        </div>
        <div
          v-else-if="announcements.length === 0"
          class="px-4 py-8 text-center text-xs text-muted-foreground"
        >
          {{ t('systemAnnouncements.empty') }}
        </div>
        <template v-else>
          <button
            v-for="announcement in announcements.slice(0, 5)"
            :key="announcement.id"
            type="button"
            class="block w-full border-b border-[#3d3929]/5 px-4 py-3 text-left transition last:border-b-0 hover:bg-black/[0.03] dark:border-white/5 dark:hover:bg-white/[0.04]"
            @click="openDetail(announcement)"
          >
            <div class="flex items-start justify-between gap-3">
              <span class="line-clamp-2 text-sm font-medium text-foreground">{{ announcement.title }}</span>
              <span
                v-if="!announcement.is_read"
                class="mt-1 h-1.5 w-1.5 shrink-0 rounded-full bg-[#cc785c]"
              />
            </div>
            <p class="mt-1 line-clamp-2 text-xs leading-5 text-muted-foreground">{{ preview(announcement.content) }}</p>
            <time class="mt-1 block text-[10px] text-muted-foreground/70">{{ formatDate(announcement.created_at) }}</time>
          </button>
        </template>
      </div>

      <RouterLink
        to="/dashboard/announcements"
        class="block border-t border-[#3d3929]/10 px-4 py-2.5 text-center text-xs font-medium text-[#a8533a] transition hover:bg-[#cc785c]/10 dark:border-white/10 dark:text-[#d4a27f]"
        @click="menuOpen = false"
      >
        {{ t('systemAnnouncements.viewAll') }}
      </RouterLink>
    </div>
  </div>

  <AnnouncementDetailDialog v-model="detailOpen" :announcement="selectedAnnouncement" />
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Megaphone } from 'lucide-vue-next'
import AnnouncementDetailDialog from './AnnouncementDetailDialog.vue'
import { useAuthStore } from '@/stores/auth'
import { announcementApi, type Announcement } from '@/api/announcements'

const authStore = useAuthStore()
const { t, locale } = useI18n()
const menuOpen = ref(false)
const loading = ref(false)
const announcements = ref<Announcement[]>([])
const selectedAnnouncement = ref<Announcement | null>(null)
const detailOpen = ref(false)
const unreadCount = computed(() => announcements.value.filter(item => !item.is_read).length)

async function loadAnnouncements() {
  if (!authStore.user || !authStore.token) {
    announcements.value = []
    return
  }
  loading.value = true
  try {
    const response = await announcementApi.getAnnouncements({ active_only: true, limit: 20, offset: 0 })
    announcements.value = response.items
  } catch {
    announcements.value = []
  } finally {
    loading.value = false
  }
}

function preview(content: string): string {
  const value = content
    .replace(/```[\s\S]*?```/g, ' ')
    .replace(/`[^`]*`/g, ' ')
    .replace(/!\[[^\]]*\]\([^)]*\)/g, ' ')
    .replace(/\[[^\]]*\]\(([^)]*)\)/g, '$1')
    .replace(/[#>*_~]/g, '')
    .replace(/\s+/g, ' ')
    .trim()
  return value.length > 120 ? `${value.slice(0, 120).trim()}...` : value
}

function formatDate(value: string): string {
  return new Date(value).toLocaleString(locale.value)
}

async function openDetail(announcement: Announcement) {
  selectedAnnouncement.value = announcement
  detailOpen.value = true
  menuOpen.value = false
  // 点击后立即视为已读，不在本地保留未读状态。
  if (!announcement.is_read) {
    announcement.is_read = true
    try { await announcementApi.markAsRead(announcement.id) } catch { /* detail remains available */ }
  }
}

watch(() => [authStore.user, authStore.token] as const, loadAnnouncements, { immediate: true })
</script>
