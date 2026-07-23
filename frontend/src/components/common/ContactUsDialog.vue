<template>
  <slot name="trigger" :open="openDialog" />
  <Dialog v-model:open="open">
    <template #header>
      <div class="flex items-start justify-between gap-4 border-b border-border px-5 py-4 sm:px-6">
        <div>
          <DialogTitle>{{ t('publicContent.contactUs') }}</DialogTitle>
          <p class="mt-1 text-sm text-muted-foreground">{{ t('publicContent.contactDialogHint') }}</p>
        </div>
        <button type="button" class="rounded-md p-1 text-muted-foreground transition hover:bg-muted hover:text-foreground" :aria-label="t('common.close')" @click="open = false">
          <X class="h-5 w-5" />
        </button>
      </div>
    </template>
    <div class="max-h-[70vh] overflow-y-auto px-5 py-5 sm:px-6">
      <div v-if="loading" class="py-8 text-center text-sm text-muted-foreground">{{ t('common.loading') }}</div>
      <div v-else-if="loadError" class="rounded-lg border border-destructive/20 bg-destructive/5 p-4 text-sm text-destructive">{{ loadError }}</div>
      <!-- eslint-disable-next-line vue/no-v-html -->
      <article v-else class="contact-dialog-content prose prose-sm max-w-none dark:prose-invert" v-html="renderedContent" />
    </div>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { marked } from 'marked'
import { X } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { Dialog, DialogTitle } from '@/components/ui'
import { authApi, type ContactUsSettings } from '@/api/auth'
import { sanitizeHtml, sanitizeMarkdown } from '@/utils/sanitize'

const { t } = useI18n()
const open = ref(false)
const loading = ref(false)
const loadError = ref('')
const loaded = ref(false)
const contactUs = ref<ContactUsSettings>({ format: 'markdown', content: '' })

const renderedContent = computed(() => {
  if (!contactUs.value.content) return `<p>${t('publicContent.noContactContent')}</p>`
  return contactUs.value.format === 'html'
    ? sanitizeHtml(contactUs.value.content)
    : sanitizeMarkdown(marked(contactUs.value.content) as string)
})

function openDialog() {
  open.value = true
}

watch(open, async (isOpen) => {
  if (!isOpen || loaded.value || loading.value) return
  loading.value = true
  loadError.value = ''
  try {
    const settings = await authApi.getRegistrationSettings()
    contactUs.value = settings.contact_us ?? contactUs.value
    loaded.value = true
  } catch {
    loadError.value = t('publicContent.contactLoadFailed')
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
.contact-dialog-content :deep(img) { max-width: 100%; height: auto; border-radius: 0.5rem; }
.contact-dialog-content :deep(a) { color: hsl(var(--primary)); text-decoration: underline; text-underline-offset: 3px; }
.contact-dialog-content :deep(p) { margin: 0.75rem 0; }
</style>
