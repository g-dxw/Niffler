<template>
  <p class="text-center text-sm text-muted-foreground">
    <template v-if="allowRegistration">
      没有账户？
      <button
        type="button"
        class="register-action font-medium text-primary transition-colors hover:text-foreground active:translate-y-px disabled:pointer-events-none disabled:opacity-50"
        :disabled="disabled"
        :data-state="disabled ? 'disabled' : 'idle'"
        @click="$emit('register')"
      >
        注册
      </button>
    </template>
    <template v-else-if="showContactAdmin">
      如需开通账户，请联系管理员
    </template>
  </p>
</template>

<script setup lang="ts">
defineProps<{
  allowRegistration: boolean
  showContactAdmin: boolean
  disabled?: boolean
}>()

defineEmits<{
  register: []
}>()
</script>
