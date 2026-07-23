<template>
  <div class="relative ml-1" @mouseenter="$emit('open-hover')" @mouseleave="$emit('close-hover')">
    <button type="button" class="flex h-9 items-center gap-2 rounded-lg bg-[#cc785c]/10 px-3 text-sm font-semibold text-[#a8533a] transition hover:bg-[#cc785c]/15 dark:bg-[#d4a27f]/10 dark:text-[#d4a27f]" :title="t('console.balanceDetails')" :aria-label="t('console.balanceDetails')" :aria-expanded="open" @click="$emit('toggle')">
      <Wallet class="h-4 w-4" /><span class="tabular-nums">{{ total }}</span>
    </button>
    <div v-if="open" class="absolute right-0 top-[calc(100%+0.5rem)] z-[70] w-64 rounded-xl border border-[#3d3929]/10 bg-[#faf9f5] p-3 text-sm shadow-xl dark:border-white/10 dark:bg-[#1e1c19]">
      <div class="flex items-center justify-between py-1.5 text-muted-foreground"><span>{{ t('console.walletBalance') }}</span><span class="font-semibold tabular-nums text-foreground">{{ wallet }}</span></div>
      <div class="flex items-center justify-between py-1.5 text-muted-foreground"><span>{{ t('console.packageQuota') }}</span><span class="font-semibold tabular-nums text-foreground">{{ packageQuota }}</span></div>
      <div class="my-1 border-t border-[#3d3929]/10 dark:border-white/10" />
      <div class="flex items-center justify-between py-1.5 text-muted-foreground"><span>{{ t('console.totalAvailable') }}</span><span class="font-semibold tabular-nums text-foreground">{{ total }}</span></div>
      <RouterLink to="/dashboard/wallet" class="mt-2 flex items-center justify-center rounded-lg px-3 py-2 text-xs font-medium text-[#a8533a] hover:bg-[#cc785c]/10 dark:text-[#d4a27f]" @click="$emit('close')">{{ t('console.viewWallet') }}</RouterLink>
    </div>
  </div>
</template>
<script setup lang="ts">
import { Wallet } from 'lucide-vue-next'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
defineProps<{ open: boolean; total: string; wallet: string; packageQuota: string }>()
defineEmits<{ toggle: []; 'open-hover': []; 'close-hover': []; close: [] }>()
</script>
