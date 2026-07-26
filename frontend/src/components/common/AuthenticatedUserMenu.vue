<template>
  <div ref="root" class="relative ml-1">
    <button
      type="button"
      class="flex h-9 w-9 items-center justify-center overflow-hidden rounded-lg border border-border/70 bg-muted/40 text-xs font-bold text-foreground shadow-sm transition hover:border-primary/40 hover:bg-muted"
      :title="t('authenticatedMenu.title')"
      :aria-label="t('authenticatedMenu.title')"
      :aria-expanded="open"
      @click="open = !open"
    >
      {{ initials }}
    </button>

    <div
      v-if="open"
      class="absolute right-0 top-[calc(100%+0.5rem)] z-[70] w-64 rounded-xl border border-border bg-card p-2 text-foreground shadow-xl"
    >
      <div class="border-b border-border px-3 pb-2.5 pt-1">
        <div class="truncate text-sm font-semibold">{{ authStore.user?.username || t('authenticatedMenu.user') }}</div>
        <div class="mt-0.5 text-xs text-muted-foreground">{{ roleLabel }}</div>
      </div>
      <RouterLink
        :to="dashboardPath"
        class="mt-1 flex items-center gap-2 rounded-lg px-3 py-2 text-sm text-muted-foreground hover:bg-muted/50 hover:text-foreground"
        @click="open = false"
      >
        <LayoutDashboard class="h-4 w-4" />
        {{ t('authenticatedMenu.dashboard') }}
      </RouterLink>
      <RouterLink
        to="/dashboard/settings"
        class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm text-muted-foreground hover:bg-muted/50 hover:text-foreground"
        @click="open = false"
      >
        <Settings class="h-4 w-4" />
        {{ t('authenticatedMenu.settings') }}
      </RouterLink>
      <button
        type="button"
        class="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-muted-foreground hover:bg-red-500/10 hover:text-red-500"
        @click="handleLogout"
      >
        <LogOut class="h-4 w-4" />
        {{ t('authenticatedMenu.logout') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { LayoutDashboard, LogOut, Settings } from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth'
import router from '@/router'

const authStore = useAuthStore()
const { t } = useI18n()
const root = ref<HTMLElement | null>(null)
const open = ref(false)
const dashboardPath = computed(() => authStore.canAccessAdmin ? '/admin/dashboard' : '/dashboard')
const initials = computed(() => authStore.user?.username?.slice(0, 2).toUpperCase() || 'U')
const roleLabel = computed(() => {
  if (authStore.isAdmin) return t('authenticatedMenu.admin')
  if (authStore.isAuditAdmin) return t('authenticatedMenu.auditAdmin')
  return t('authenticatedMenu.user')
})

function handlePointerDown(event: PointerEvent) {
  if (open.value && root.value && !root.value.contains(event.target as Node)) open.value = false
}

async function handleLogout() {
  open.value = false
  await authStore.logout()
  await router.replace('/')
}

onMounted(() => document.addEventListener('pointerdown', handlePointerDown))
onBeforeUnmount(() => document.removeEventListener('pointerdown', handlePointerDown))
</script>
