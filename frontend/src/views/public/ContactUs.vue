<template>
  <main class="min-h-screen bg-[#faf9f5] text-[#3d3929] dark:bg-[#191714] dark:text-[#e3e0d3]">
    <header class="border-b border-[#3d3929]/10 dark:border-white/10">
      <div class="mx-auto flex max-w-4xl items-center justify-between px-5 py-4">
        <RouterLink
          to="/"
          class="flex items-center gap-3"
        >
          <HeaderLogo
            size="h-9 w-9"
            class-name="text-[#191919] dark:text-white"
          />
          <div>
            <div class="text-sm font-semibold">
              {{ siteName }}
            </div>
            <div class="text-xs text-muted-foreground">{{ t('publicContent.contactUs') }}</div>
          </div>
        </RouterLink>
        <RouterLink
          to="/"
          class="rounded-lg border border-border px-3 py-1.5 text-sm text-muted-foreground transition hover:text-foreground"
        >
          {{ t('publicContent.backHome') }}
        </RouterLink>
      </div>
    </header>

    <section class="mx-auto max-w-4xl px-5 py-8">
      <div class="mb-6">
        <h1 class="text-2xl font-semibold">
          {{ t('publicContent.contactUs') }}
        </h1>
      </div>

      <div
        v-if="loading"
        class="rounded-lg border border-border bg-background/70 p-6 text-sm text-muted-foreground"
      >
        {{ t('common.loading') }}
      </div>
      <div
        v-else-if="loadError"
        class="rounded-lg border border-destructive/20 bg-destructive/5 p-6 text-sm text-destructive"
      >
        {{ loadError }}
      </div>
      <!-- eslint-disable vue/no-v-html -->
      <article
        v-else
        class="contact-content prose prose-sm dark:prose-invert max-w-none rounded-lg border border-border bg-background/70 p-6"
        v-html="renderedContent"
      />
      <!-- eslint-enable vue/no-v-html -->
    </section>
  </main>
  <PublicFooter />
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { marked } from 'marked'
import { authApi, type ContactUsSettings } from '@/api/auth'
import HeaderLogo from '@/components/HeaderLogo.vue'
import PublicFooter from '@/components/common/PublicFooter.vue'
import { useSiteInfo } from '@/composables/useSiteInfo'
import { sanitizeHtml, sanitizeMarkdown } from '@/utils/sanitize'

const { siteName } = useSiteInfo()
const { t } = useI18n()
const loading = ref(true)
const loadError = ref('')
const contactUs = ref<ContactUsSettings>({
  format: 'markdown',
  content: '',
})

const renderedContent = computed(() => {
  if (!contactUs.value.content) return `<p>${t('publicContent.noContactContent')}</p>`
  if (contactUs.value.format === 'html') return sanitizeHtml(contactUs.value.content)
  return sanitizeMarkdown(marked(contactUs.value.content) as string)
})

onMounted(async () => {
  try {
    const settings = await authApi.getRegistrationSettings()
    contactUs.value = settings.contact_us ?? contactUs.value
  } catch {
    loadError.value = t('publicContent.contactLoadFailed')
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
.contact-content :deep(h1),
.contact-content :deep(h2),
.contact-content :deep(h3),
.contact-content :deep(h4) {
  color: hsl(var(--foreground));
  font-weight: 650;
  letter-spacing: -0.015em;
  line-height: 1.3;
}

.contact-content :deep(h1) { margin: 0 0 1.5rem; font-size: 1.75rem; }
.contact-content :deep(h2) { margin: 2.25rem 0 0.85rem; padding-bottom: 0.5rem; border-bottom: 1px solid hsl(var(--border)); font-size: 1.35rem; }
.contact-content :deep(h3) { margin: 1.5rem 0 0.6rem; font-size: 1.1rem; }
.contact-content :deep(p) { margin: 0.9rem 0; color: hsl(var(--muted-foreground)); line-height: 1.9; }
.contact-content :deep(ul), .contact-content :deep(ol) { margin: 1rem 0; padding-left: 1.5rem; color: hsl(var(--muted-foreground)); }
.contact-content :deep(li) { margin: 0.45rem 0; padding-left: 0.25rem; line-height: 1.8; }
.contact-content :deep(blockquote) { margin: 1.25rem 0; border-left: 3px solid hsl(var(--primary)); background: hsl(var(--muted) / 0.35); padding: 0.75rem 1rem; color: hsl(var(--muted-foreground)); }
.contact-content :deep(a) { color: hsl(var(--primary)); text-decoration: underline; text-underline-offset: 3px; }
.contact-content :deep(code) { overflow-wrap: anywhere; }
.contact-content :deep(pre) { max-width: 100%; overflow-x: auto; border: 1px solid hsl(var(--border)); border-radius: 0.6rem; padding: 1rem; }
.contact-content :deep(table) { display: block; max-width: 100%; overflow-x: auto; white-space: nowrap; }
</style>
