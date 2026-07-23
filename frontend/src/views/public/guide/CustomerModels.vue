<template>
  <div>
    <div class="guide-eyebrow">{{ t('guide.models.eyebrow') }}</div>
    <h1 class="mt-4">{{ t('guide.models.title') }}</h1>
    <p class="mt-5 max-w-3xl text-lg">{{ t('guide.models.subtitle') }}</p>
    <RouterLink to="/models" class="mt-7 inline-flex items-center gap-2 bg-primary px-5 py-3 text-sm font-semibold text-primary-foreground">{{ t('guide.models.openMarketplace') }}<ArrowRight class="h-4 w-4" /></RouterLink>

    <section>
      <h2>{{ t('guide.models.chooseTitle') }}</h2>
      <div class="grid gap-3 sm:grid-cols-2">
        <div v-for="item in choices" :key="item.title" class="border border-border/80 bg-background/70 p-5">
          <component :is="item.icon" class="h-5 w-5 text-primary" />
          <h3>{{ item.title }}</h3>
          <p class="text-sm">{{ item.description }}</p>
        </div>
      </div>
    </section>

    <section>
      <h2>{{ t('guide.models.pricingTitle') }}</h2>
      <p>{{ t('guide.models.pricingDesc') }}</p>
      <div class="mt-5 flex gap-3 border-l-4 border-primary bg-primary/5 p-5 text-sm leading-7 text-muted-foreground">
        <CircleDollarSign class="mt-1 h-4 w-4 shrink-0 text-primary" aria-hidden="true" />
        <div>
          <strong class="text-foreground">{{ t('guide.models.exchangeTitle') }}</strong><br>
          {{ t('guide.models.exchangeDesc') }}
        </div>
      </div>
      <div class="mt-5 overflow-hidden border border-border/80">
        <div class="grid grid-cols-3 bg-muted/40 px-4 py-3 text-xs font-bold uppercase tracking-wider text-muted-foreground"><span>{{ t('guide.models.item') }}</span><span>{{ t('guide.models.unit') }}</span><span>{{ t('guide.models.meaning') }}</span></div>
        <div v-for="row in pricingRows" :key="row.item" class="grid grid-cols-3 border-t border-border/60 px-4 py-4 text-sm"><strong>{{ row.item }}</strong><code>{{ row.unit }}</code><span class="text-muted-foreground">{{ row.meaning }}</span></div>
      </div>
    </section>

    <section>
      <h2>{{ t('guide.models.modelIdTitle') }}</h2>
      <p>{{ t('guide.models.modelIdDesc') }}</p>
      <CustomerCodeBlock label="Request body" :code="modelExample" />
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { ArrowRight, Brain, CircleDollarSign, Image, Rabbit, Scale } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import CustomerCodeBlock from './components/CustomerCodeBlock.vue'
const { t } = useI18n()
const choices = computed(() => [
  { icon: Rabbit, title: t('guide.models.fastTitle'), description: t('guide.models.fastDesc') },
  { icon: Brain, title: t('guide.models.reasoningTitle'), description: t('guide.models.reasoningDesc') },
  { icon: Image, title: t('guide.models.imageTitle'), description: t('guide.models.imageDesc') },
  { icon: Scale, title: t('guide.models.costTitle'), description: t('guide.models.costDesc') },
])
const pricingRows = computed(() => [
  { item: t('guide.models.input'), unit: '/ 1M tokens', meaning: t('guide.models.inputDesc') },
  { item: t('guide.models.output'), unit: '/ 1M tokens', meaning: t('guide.models.outputDesc') },
  { item: t('guide.models.perRequest'), unit: t('guide.models.requestUnit'), meaning: t('guide.models.perRequestDesc') },
])
const modelExample = `{
  "model": "claude-sonnet-4-6",
  "messages": [
    { "role": "user", "content": "Explain this code" }
  ]
}`
</script>

<style scoped>.guide-eyebrow { color: hsl(var(--primary)); font-size: 11px; font-weight: 700; letter-spacing: 0.2em; text-transform: uppercase; }</style>
