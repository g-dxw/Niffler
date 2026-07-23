<template>
  <div class="flex items-center gap-1">
    <ContactUsButton />
    <button type="button" class="flex h-9 w-9 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted/50 hover:text-foreground" :title="themeTitle" @click="toggleDarkMode">
      <SunMoon v-if="themeMode === 'system'" class="h-4 w-4" />
      <Sun v-else-if="themeMode === 'light'" class="h-4 w-4" />
      <Moon v-else class="h-4 w-4" />
    </button>
    <LanguageSwitcher />
    <a v-if="showGithub" href="https://github.com/ryfineZ/Niffler" target="_blank" rel="noopener noreferrer" class="hidden h-9 w-9 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted/50 hover:text-foreground sm:flex" :title="t('common.github')">
      <GithubIcon class="h-4 w-4" />
    </a>
    <SystemAnnouncements v-if="showAnnouncements && authStore.isAuthenticated" />
    <template v-if="showPublicAccount">
      <AccountBalanceBadge v-if="authStore.isAuthenticated" />
      <AuthenticatedUserMenu v-if="authStore.isAuthenticated" />
      <button v-else class="ml-1 rounded-lg bg-primary px-3.5 py-2 text-xs font-semibold text-primary-foreground shadow-sm sm:text-sm" @click="emit('login')">{{ t('nav.login') }}</button>
    </template>
    <slot name="after" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Moon, Sun, SunMoon } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import ContactUsButton from '@/components/common/ContactUsButton.vue'
import LanguageSwitcher from '@/components/common/LanguageSwitcher.vue'
import { useDarkMode } from '@/composables/useDarkMode'
import GithubIcon from '@/components/icons/GithubIcon.vue'
import AuthenticatedUserMenu from '@/components/common/AuthenticatedUserMenu.vue'
import AccountBalanceBadge from '@/components/common/AccountBalanceBadge.vue'
import SystemAnnouncements from '@/components/common/SystemAnnouncements.vue'
import { useAuthStore } from '@/stores/auth'

withDefaults(defineProps<{ showGithub?: boolean; showAnnouncements?: boolean; showPublicAccount?: boolean }>(), {
  showGithub: false,
  showAnnouncements: true,
  showPublicAccount: false,
})
const emit = defineEmits<{ login: [] }>()
const { t } = useI18n()
const authStore = useAuthStore()
const { themeMode, toggleDarkMode } = useDarkMode()
const themeTitle = computed(() => themeMode.value === 'system' ? t('common.themeSystem') : themeMode.value === 'dark' ? t('common.themeDark') : t('common.themeLight'))
</script>
