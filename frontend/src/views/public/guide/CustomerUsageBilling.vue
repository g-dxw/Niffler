<template>
  <div>
    <div class="guide-eyebrow">{{ t('guide.billing.eyebrow') }}</div>
    <h1 class="mt-4">{{ t('guide.billing.title') }}</h1>
    <p class="mt-5 max-w-3xl text-lg">{{ t('guide.billing.subtitle') }}</p>

    <section>
      <h2>{{ t('guide.billing.balanceTitle') }}</h2>
      <div class="grid gap-3 sm:grid-cols-3">
        <RouterLink v-for="item in balanceItems" :key="item.title" :to="item.to" class="border border-border/80 bg-background/70 p-5 transition hover:border-primary/50">
          <component :is="item.icon" class="h-5 w-5 text-primary" />
          <h3>{{ item.title }}</h3><p class="text-sm">{{ item.description }}</p>
        </RouterLink>
      </div>
    </section>

    <section>
      <h2>{{ t('guide.billing.usageTitle') }}</h2>
      <p>{{ t('guide.billing.usageDesc') }}</p>
      <div class="mt-5 space-y-3">
        <div v-for="(item, index) in usageItems" :key="item.title" class="flex gap-4 border border-border/70 bg-background/65 p-5">
          <span class="flex h-8 w-8 shrink-0 items-center justify-center bg-primary font-mono text-xs font-bold text-primary-foreground">{{ index + 1 }}</span>
          <div><strong>{{ item.title }}</strong><p class="mt-1 text-sm">{{ item.description }}</p></div>
        </div>
      </div>
    </section>

    <section>
      <h2>{{ t('guide.billing.tipsTitle') }}</h2>
      <ul class="space-y-3">
        <li v-for="tip in tips" :key="tip" class="flex gap-3 border-b border-border/60 pb-3 text-sm text-muted-foreground"><CheckCircle2 class="mt-0.5 h-4 w-4 shrink-0 text-emerald-500" />{{ tip }}</li>
      </ul>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { BarChart3, CheckCircle2, Package, Wallet } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
const balanceItems = computed(() => [
  { icon: Wallet, title: t('guide.billing.wallet'), description: t('guide.billing.walletDesc'), to: '/dashboard/wallet' },
  { icon: Package, title: t('guide.billing.plan'), description: t('guide.billing.planDesc'), to: '/dashboard/billing' },
  { icon: BarChart3, title: t('guide.billing.records'), description: t('guide.billing.recordsDesc'), to: '/dashboard/usage' },
])
const usageItems = computed(() => [
  { title: t('guide.billing.usageStep1'), description: t('guide.billing.usageStep1Desc') },
  { title: t('guide.billing.usageStep2'), description: t('guide.billing.usageStep2Desc') },
  { title: t('guide.billing.usageStep3'), description: t('guide.billing.usageStep3Desc') },
])
const tips = computed(() => [t('guide.billing.tip1'), t('guide.billing.tip2'), t('guide.billing.tip3')])
</script>

<style scoped>.guide-eyebrow { color: hsl(var(--primary)); font-size: 11px; font-weight: 700; letter-spacing: 0.2em; text-transform: uppercase; }</style>
