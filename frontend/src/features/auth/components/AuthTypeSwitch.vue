<template>
  <div
    v-if="showTabs"
    class="auth-type-tabs"
  >
    <button
      type="button"
      class="auth-tab"
      :class="[modelValue === 'local' && 'active']"
      :aria-pressed="modelValue === 'local'"
      :disabled="disabled || modelValue === 'local'"
      @click="$emit('update:modelValue', 'local')"
    >
      账号密码
    </button>
    <button
      type="button"
      class="auth-tab"
      :class="[modelValue === 'ldap' && 'active']"
      :aria-pressed="modelValue === 'ldap'"
      :disabled="disabled || modelValue === 'ldap'"
      @click="$emit('update:modelValue', 'ldap')"
    >
      LDAP
    </button>
  </div>

  <div
    v-else-if="showExclusiveSwitch"
    class="exclusive-switch"
  >
    <span>{{ modelValue === 'ldap' ? '企业账号登录' : '管理员本地登录' }}</span>
    <button
      type="button"
      class="exclusive-switch__button"
      :disabled="disabled"
      @click="$emit('update:modelValue', modelValue === 'ldap' ? 'local' : 'ldap')"
    >
      {{ modelValue === 'ldap' ? '管理员入口' : '返回 LDAP' }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  modelValue: 'local' | 'ldap'
  localEnabled: boolean
  ldapEnabled: boolean
  ldapExclusive: boolean
  disabled?: boolean
}>()

defineEmits<{
  'update:modelValue': [value: 'local' | 'ldap']
}>()

const showTabs = computed(() => {
  return props.localEnabled && props.ldapEnabled && !props.ldapExclusive
})

const showExclusiveSwitch = computed(() => {
  return props.ldapEnabled && props.ldapExclusive
})
</script>

<style scoped>
.auth-type-tabs {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.25rem;
  padding: 0.25rem;
  border-radius: calc(var(--radius) + 0.125rem);
  background: var(--muted);
}

.auth-tab {
  height: 2.25rem;
  border-radius: var(--radius);
  color: var(--muted-foreground);
  font-size: 0.875rem;
  font-weight: 600;
  transition:
    color 160ms ease,
    background-color 160ms ease,
    transform 160ms ease;
}

.auth-tab:hover {
  color: var(--foreground);
}

.auth-tab.active {
  color: var(--foreground);
  background: var(--background);
}

.auth-tab:active {
  transform: translateY(1px);
}

.auth-tab:disabled,
.exclusive-switch__button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.exclusive-switch {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  color: var(--muted-foreground);
  font-size: 0.8125rem;
}

.exclusive-switch__button {
  color: var(--primary);
  font-weight: 600;
  transition:
    color 160ms ease,
    transform 160ms ease;
}

.exclusive-switch__button:hover {
  color: var(--foreground);
}

.exclusive-switch__button:active {
  transform: translateY(1px);
}
</style>
