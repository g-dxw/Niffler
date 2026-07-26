<template>
  <ContactUsDialog>
    <template #trigger="{ open }">
      <button
        type="button"
        class="flex h-9 w-9 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted/50 hover:text-foreground"
        :title="t('publicContent.contactUs')"
        :aria-label="t('publicContent.contactUs')"
        @click="open"
      >
        <UsersRound class="h-4 w-4" />
      </button>
    </template>
  </ContactUsDialog>
</template>

<script setup lang="ts">
import { UsersRound } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import ContactUsDialog from '@/components/common/ContactUsDialog.vue'

const { t } = useI18n()
</script>
