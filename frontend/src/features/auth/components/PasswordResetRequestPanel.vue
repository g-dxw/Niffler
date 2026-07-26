<template>
  <form
    class="space-y-4"
    @submit.prevent="$emit('submit')"
  >
    <p class="text-sm text-muted-foreground">
      {{ t('passwordResetRequest.description') }}
    </p>
    <div class="space-y-1.5">
      <Label
        for="password-reset-email"
        class="text-sm"
      >
        {{ t('passwordResetRequest.email') }}
      </Label>
      <Input
        id="password-reset-email"
        :model-value="email"
        type="email"
        name="email"
        required
        placeholder="you@example.com"
        autocomplete="email"
        autocapitalize="none"
        spellcheck="false"
        @update:model-value="$emit('update:email', String($event))"
      />
    </div>
    <div
      v-if="notice"
      class="rounded-lg border border-border bg-muted/40 px-3 py-2 text-sm text-foreground"
      role="status"
    >
      {{ notice }}
    </div>
    <Button
      type="submit"
      :disabled="loading"
      :aria-busy="loading"
      class="h-12 w-full"
    >
      {{ loading ? t('passwordResetRequest.sending') : t('passwordResetRequest.send') }}
    </Button>
    <Button
      type="button"
      variant="ghost"
      class="w-full"
      :disabled="loading"
      :data-state="loading ? 'disabled' : 'idle'"
      @click="$emit('back')"
    >
      {{ t('passwordResetRequest.backToLogin') }}
    </Button>
  </form>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'

const { t } = useI18n()

defineProps<{
  email: string
  loading: boolean
  notice: string
}>()

defineEmits<{
  'update:email': [value: string]
  submit: []
  back: []
}>()
</script>
