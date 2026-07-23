<template>
  <p class="text-center text-sm text-muted-foreground">
    <template v-if="allowRegistration">
      {{ t('authCommon.noAccount') }}
      <button
        type="button"
        class="register-action font-medium text-primary transition-colors hover:text-foreground active:translate-y-px disabled:pointer-events-none disabled:opacity-50"
        :disabled="disabled"
        :data-state="disabled ? 'disabled' : 'idle'"
        @click="$emit('register')"
      >
        {{ t('authCommon.register') }}
      </button>
    </template>
    <template v-else-if="showContactAdmin">
      {{ t('authCommon.contactAdmin') }}
    </template>
  </p>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

defineProps<{
  allowRegistration: boolean
  showContactAdmin: boolean
  disabled?: boolean
}>()

defineEmits<{
  register: []
}>()
</script>
