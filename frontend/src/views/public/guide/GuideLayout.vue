<template>
  <div class="min-h-screen bg-background text-foreground literary-grid literary-paper">
    <div class="mx-auto grid max-w-[1480px] lg:grid-cols-[250px_minmax(0,1fr)]">
      <aside class="hidden min-h-[calc(100vh-4rem)] border-r border-border/70 bg-background/55 p-5 lg:block">
        <div class="sticky top-24">
          <div class="px-3 text-[10px] font-bold uppercase tracking-[0.18em] text-primary">{{ t('guide.customerDocs') }}</div>
          <nav class="mt-4 space-y-1">
            <RouterLink v-for="item in navItems" :key="item.id" :to="item.path" class="block border px-3 py-3 transition" :class="isActive(item.path) ? 'border-primary bg-primary/10 text-primary' : 'border-transparent text-muted-foreground hover:border-border hover:bg-muted/30 hover:text-foreground'">
              <div class="flex items-center gap-2.5 text-sm font-medium"><component :is="item.icon" class="h-4 w-4" />{{ item.name }}</div>
              <div class="mt-1 pl-6 text-[11px] leading-4 opacity-75">{{ item.description }}</div>
            </RouterLink>
          </nav>
        </div>
      </aside>

      <main class="min-w-0 px-4 pb-20 pt-8 sm:px-8 lg:px-12 lg:pt-12 xl:px-16">
        <div class="mb-7 flex gap-2 overflow-x-auto pb-2 lg:hidden">
          <RouterLink v-for="item in navItems" :key="item.id" :to="item.path" class="shrink-0 border px-3 py-2 text-xs font-medium" :class="isActive(item.path) ? 'border-primary bg-primary/10 text-primary' : 'border-border bg-background/70'">{{ item.name }}</RouterLink>
        </div>
        <article class="mx-auto max-w-4xl customer-guide">
          <RouterView />
        </article>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { createGuideNavItems } from './guide-config'

const route = useRoute()
const { t } = useI18n()
const navItems = computed(() => createGuideNavItems(t))

function isActive(path: string) {
  return path === '/guide' ? route.path === path : route.path.startsWith(path)
}
</script>

<style scoped>
:deep(.customer-guide h1) { font-family: var(--serif); font-size: clamp(2.35rem, 6vw, 4rem); line-height: 1.05; font-weight: 600; letter-spacing: -0.03em; }
:deep(.customer-guide h2) { margin-top: 3rem; margin-bottom: 1rem; font-family: var(--serif); font-size: 1.75rem; font-weight: 600; }
:deep(.customer-guide h3) { margin-top: 1.75rem; margin-bottom: 0.75rem; font-size: 1rem; font-weight: 650; }
:deep(.customer-guide p) { color: hsl(var(--muted-foreground)); line-height: 1.8; }
:deep(.customer-guide code) { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
</style>
