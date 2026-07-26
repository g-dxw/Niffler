<template>
  <div ref="root" class="relative ml-1">
    <button
      type="button"
      class="flex h-9 items-center gap-2 rounded-lg bg-primary/10 px-3 text-sm font-semibold text-primary transition hover:bg-primary/15"
      :title="t('accountBalance.details')"
      :aria-label="t('accountBalance.details')"
      :aria-expanded="open"
      @click="open = !open"
    >
      <Wallet class="h-4 w-4" />
      <span class="tabular-nums">{{ totalLabel }}</span>
    </button>
    <div v-if="open" class="absolute right-0 top-[calc(100%+0.5rem)] z-[70] w-64 rounded-xl border border-border bg-card p-3 text-sm shadow-xl">
      <div class="flex items-center justify-between py-1.5 text-muted-foreground"><span>{{ t('accountBalance.wallet') }}</span><span class="font-semibold tabular-nums text-foreground">{{ walletLabel }}</span></div>
      <div class="flex items-center justify-between py-1.5 text-muted-foreground"><span>{{ t('accountBalance.package') }}</span><span class="font-semibold tabular-nums text-foreground">{{ packageLabel }}</span></div>
      <div class="my-1 border-t border-border" />
      <div class="flex items-center justify-between py-1.5 text-muted-foreground"><span>{{ t('accountBalance.total') }}</span><span class="font-semibold tabular-nums text-foreground">{{ totalLabel }}</span></div>
      <RouterLink to="/dashboard/wallet" class="mt-2 flex items-center justify-center rounded-lg px-3 py-2 text-xs font-medium text-primary hover:bg-primary/10" @click="open = false">{{ t('accountBalance.viewWallet') }}</RouterLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Wallet } from 'lucide-vue-next'
import { useBillingSummary } from '@/composables/useBillingSummary'

const { t } = useI18n()
const root = ref<HTMLElement | null>(null)
const open = ref(false)
const { totalLabel, walletLabel, packageLabel } = useBillingSummary()

function onPointerDown(event: PointerEvent) {
  if (open.value && root.value && !root.value.contains(event.target as Node)) open.value = false
}

onMounted(() => document.addEventListener('pointerdown', onPointerDown))
onBeforeUnmount(() => document.removeEventListener('pointerdown', onPointerDown))
</script>
