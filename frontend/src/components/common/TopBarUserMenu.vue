<template>
  <div class="relative ml-1">
    <button type="button" class="flex h-9 w-9 items-center justify-center overflow-hidden rounded-lg border border-[#3d3929]/10 bg-[#f0f0eb] text-xs font-semibold text-foreground shadow-sm transition hover:border-[#cc785c]/30 hover:bg-[#e8e6df] dark:border-white/10 dark:bg-white/[0.08]" :title="t('console.userMenu')" :aria-label="t('console.userMenu')" :aria-expanded="open" @click="$emit('toggle')">
      <span class="font-bold text-[#3d3929] dark:text-[#d4a27f]">{{ initials }}</span>
    </button>
    <div v-if="open" class="absolute right-0 top-[calc(100%+0.5rem)] z-[70] w-64 rounded-xl border border-[#3d3929]/10 bg-[#faf9f5] p-2 shadow-xl dark:border-white/10 dark:bg-[#1e1c19]">
      <div class="border-b border-[#3d3929]/10 px-3 pb-2.5 pt-1 dark:border-white/10"><div class="truncate text-sm font-semibold text-foreground">{{ username }}</div><div class="mt-0.5 text-xs text-muted-foreground">{{ role }}</div></div>
      <RouterLink to="/dashboard/settings" class="mt-1 flex items-center gap-2 rounded-lg px-3 py-2 text-sm text-muted-foreground hover:bg-muted/50 hover:text-foreground" @click="$emit('close')">{{ t('console.personalSettings') }}</RouterLink>
      <button type="button" class="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-muted-foreground hover:bg-red-500/10 hover:text-red-500" @click="$emit('logout')">{{ t('console.logout') }}</button>
    </div>
  </div>
</template>
<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
defineProps<{ open: boolean; username: string; initials: string; role: string }>()
defineEmits<{ toggle: []; close: []; logout: [] }>()
</script>
