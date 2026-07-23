<template>
  <header class="sticky top-0 z-50 border-b border-border/60 bg-background/90 backdrop-blur-xl">
    <div class="mx-auto flex h-16 max-w-[1480px] items-center px-4 sm:px-6 lg:px-8">
      <div class="flex items-center gap-2">
      <RouterLink to="/" class="hidden items-center gap-3 sm:flex">
        <HeaderLogo size="h-8 w-8 sm:h-9 sm:w-9" class-name="text-foreground" />
        <div class="hidden leading-none sm:block">
          <div class="font-semibold">{{ brandName }}</div>
          <div class="mt-1 text-[9px] uppercase tracking-[0.16em] text-muted-foreground">{{ brandSubtitle }}</div>
        </div>
      </RouterLink>

      <button
        class="flex h-9 w-9 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted/50 hover:text-foreground sm:hidden"
        :aria-expanded="mobileMenuOpen"
        :aria-label="mobileMenuOpen ? 'Close navigation' : 'Open navigation'"
        @click="mobileMenuOpen = !mobileMenuOpen"
      >
        <X v-if="mobileMenuOpen" class="h-5 w-5" />
        <Menu v-else class="h-5 w-5" />
      </button>
      </div>

      <nav class="ml-10 hidden items-center gap-1 sm:flex">
        <RouterLink
          v-for="item in navItems"
          :key="item.to"
          :to="item.to"
          class="nav-link"
          :class="isActive(item.to) ? 'nav-link-active' : ''"
          :aria-current="isActive(item.to) ? 'page' : undefined"
        >
          {{ item.label }}
        </RouterLink>
      </nav>

      <TopBarActions class="ml-auto" show-github show-public-account @login="$emit('login')" />
    </div>
    <nav v-if="mobileMenuOpen" class="border-t border-border/60 bg-background px-4 py-3 sm:hidden">
      <RouterLink
        v-for="item in navItems"
        :key="item.to"
        :to="item.to"
        class="block px-3 py-2.5 text-sm font-medium"
        :class="isActive(item.to) ? 'text-primary' : 'text-muted-foreground'"
        @click="mobileMenuOpen = false"
      >
        {{ item.label }}
      </RouterLink>
    </nav>
  </header>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { Menu, X } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import HeaderLogo from '@/components/HeaderLogo.vue'
import TopBarActions from '@/components/common/TopBarActions.vue'
import { useAuthStore } from '@/stores/auth'
import { useSiteInfo } from '@/composables/useSiteInfo'

defineEmits<{ login: [] }>()
const route = useRoute()
const { t } = useI18n()
const authStore = useAuthStore()
const { siteName, siteSubtitle } = useSiteInfo()
const mobileMenuOpen = ref(false)

const brandName = computed(() => siteName.value || 'Niffler')
const brandSubtitle = computed(() => siteSubtitle.value || 'AI Gateway')
const dashboardPath = computed(() => authStore.canAccessAdmin ? '/admin/dashboard' : '/dashboard')
const navItems = computed(() => {
  const items = [
    { to: '/', label: t('nav.home') },
    { to: '/models', label: t('nav.models') },
    { to: '/guide', label: t('nav.docs') },
  ]
  if (authStore.isAuthenticated) {
    items.push({ to: dashboardPath.value, label: t('nav.dashboard') })
  }
  return items
})

function isActive(path: string) {
  return path === '/' ? route.path === '/' : route.path === path || route.path.startsWith(`${path}/`)
}
</script>

<style scoped>
.nav-link { position: relative; border-radius: 0.5rem; padding: 0.5rem 0.75rem; color: hsl(var(--muted-foreground)); font-size: 0.875rem; font-weight: 500; transition: color 150ms ease, background-color 150ms ease; }
.nav-link:hover { color: hsl(var(--foreground)); background: hsl(var(--muted) / 0.5); }
.nav-link-active { color: hsl(var(--primary)); background: hsl(var(--primary) / 0.1); font-weight: 600; }
.nav-link-active::after { content: ''; position: absolute; left: 0.75rem; right: 0.75rem; bottom: 0.2rem; height: 2px; border-radius: 999px; background: hsl(var(--primary)); }
</style>
