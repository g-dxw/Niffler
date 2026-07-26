<template>
  <div class="my-5 overflow-hidden border border-border/80 bg-[#1c1a17] text-[#eee8dc] shadow-sm">
    <div class="flex items-center justify-between border-b border-white/10 px-4 py-2.5 text-xs text-[#b9b1a3]">
      <span>{{ label }}</span>
      <button class="flex items-center gap-1.5 hover:text-white" @click="copy">
        <Check v-if="copied" class="h-3.5 w-3.5 text-emerald-400" />
        <Copy v-else class="h-3.5 w-3.5" />
        {{ copied ? t('common.copied') : t('common.copy') }}
      </button>
    </div>
    <pre class="overflow-x-auto p-5 text-[12px] leading-7 sm:text-[13px]"><code>{{ code }}</code></pre>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Check, Copy } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { useClipboard } from '@/composables/useClipboard'

const props = withDefaults(defineProps<{ code: string; label?: string }>(), { label: 'Code' })
const { t } = useI18n()
const { copyToClipboard } = useClipboard()
const copied = ref(false)

async function copy() {
  await copyToClipboard(props.code)
  copied.value = true
  setTimeout(() => { copied.value = false }, 1500)
}
</script>
