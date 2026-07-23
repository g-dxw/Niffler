<template>
  <div class="relative">
    <button
      class="flex h-9 items-center gap-1.5 rounded-lg px-2.5 text-sm text-muted-foreground transition hover:bg-muted/50 hover:text-foreground"
      :aria-label="currentLabel"
      :aria-expanded="open"
      @click="open = !open"
    >
      <Languages class="h-4 w-4" />
      <span class="hidden sm:inline">{{ shortLabel }}</span>
      <ChevronDown class="h-3 w-3 transition" :class="open ? 'rotate-180' : ''" />
    </button>
    <div
      v-if="open"
      class="absolute right-0 top-11 z-[80] min-w-36 rounded-xl border border-border bg-background p-1.5 shadow-xl ring-1 ring-black/5 dark:ring-white/10"
    >
      <button
        v-for="option in options"
        :key="option.value"
        class="flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition hover:bg-muted/60"
        :class="locale === option.value ? 'text-primary font-medium' : 'text-foreground'"
        @click="selectLocale(option.value)"
      >
        {{ option.label }}
        <Check v-if="locale === option.value" class="h-3.5 w-3.5" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { Check, ChevronDown, Languages } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { setAppLocale, type AppLocale } from '@/i18n'

const { locale } = useI18n()
const open = ref(false)
const options: { value: AppLocale; label: string; short: string }[] = [
  { value: 'zh-CN', label: '简体中文', short: '中' },
  { value: 'en-US', label: 'English', short: 'EN' },
]
const current = computed(() => options.find(option => option.value === locale.value) ?? options[0])
const currentLabel = computed(() => current.value.label)
const shortLabel = computed(() => current.value.short)

function selectLocale(value: AppLocale) {
  setAppLocale(value)
  open.value = false
}
</script>
