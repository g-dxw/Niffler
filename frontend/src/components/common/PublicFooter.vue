<template>
  <footer class="border-t border-border/70 bg-background/80">
    <div class="mx-auto flex max-w-[1480px] flex-col gap-4 px-4 py-8 text-xs text-muted-foreground sm:flex-row sm:items-center sm:justify-between sm:px-6 lg:px-8">
      <div class="flex items-center gap-3">
        <HeaderLogo size="h-6 w-6" class-name="text-foreground" />
        <span>{{ brandName }} · {{ t('home.footerTagline') }}</span>
      </div>
      <nav class="flex items-center gap-5" :aria-label="t('publicFooter.navigation')">
        <RouterLink to="/models" class="hover:text-foreground">{{ t('nav.models') }}</RouterLink>
        <RouterLink to="/guide" class="hover:text-foreground">{{ t('nav.docs') }}</RouterLink>
        <RouterLink to="/privacy-policy" class="hover:text-foreground">{{ t('home.privacy') }}</RouterLink>
      </nav>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import HeaderLogo from '@/components/HeaderLogo.vue'
import { useSiteInfo } from '@/composables/useSiteInfo'

const { t } = useI18n()
const { siteName } = useSiteInfo()
const brandName = computed(() => siteName.value || 'Niffler')
</script>
