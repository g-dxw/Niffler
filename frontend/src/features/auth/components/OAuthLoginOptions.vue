<template>
  <section
    v-if="providers.length > 0"
    class="space-y-3"
  >
    <button
      v-if="providers.length === 1"
      type="button"
      class="oauth-btn"
      :disabled="disabled"
      :aria-busy="disabled"
      :data-state="disabled ? 'disabled' : 'idle'"
      @click="$emit('login', providers[0].provider_type)"
    >
      <!-- eslint-disable vue/no-v-html -->
      <span
        class="oauth-icon"
        v-html="getOAuthIcon(providers[0].provider_type, providers[0].icon_url)"
      />
      <!-- eslint-enable vue/no-v-html -->
      <span>{{ t('authCommon.signInWith', { provider: providers[0].display_name }) }}</span>
    </button>

    <div
      v-else
      class="oauth-grid"
    >
      <button
        v-for="provider in providers"
        :key="provider.provider_type"
        type="button"
        class="oauth-provider"
        :title="provider.display_name"
        :disabled="disabled"
        :aria-busy="disabled"
        :data-state="disabled ? 'disabled' : 'idle'"
        @click="$emit('login', provider.provider_type)"
      >
        <!-- eslint-disable vue/no-v-html -->
        <span
          class="oauth-icon"
          v-html="getOAuthIcon(provider.provider_type, provider.icon_url)"
        />
        <!-- eslint-enable vue/no-v-html -->
        <span>{{ provider.display_name }}</span>
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { OAuthProviderInfo } from '@/api/oauth'
import { getOAuthIcon } from '@/utils/oauth-icons'

const { t } = useI18n()

defineProps<{
  providers: OAuthProviderInfo[]
  disabled?: boolean
}>()

defineEmits<{
  login: [providerType: string]
}>()
</script>

<style scoped>
.oauth-btn,
.oauth-provider {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  min-height: 2.5rem;
  border-radius: calc(var(--radius) + 0.125rem);
  color: var(--foreground);
  background: color-mix(in oklch, var(--muted) 55%, transparent);
  border: 1px solid var(--border);
  font-size: 0.875rem;
  font-weight: 600;
  transition:
    border-color 160ms ease,
    background-color 160ms ease,
    transform 160ms ease;
}

.oauth-btn {
  width: 100%;
}

.oauth-btn:hover,
.oauth-provider:hover {
  background: var(--muted);
  border-color: var(--primary);
}

.oauth-btn:active,
.oauth-provider:active {
  transform: translateY(1px);
}

.oauth-btn:disabled,
.oauth-provider:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.oauth-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.5rem;
}

.oauth-icon {
  width: 1.25rem;
  height: 1.25rem;
  flex-shrink: 0;
}

.oauth-icon :deep(svg) {
  width: 100%;
  height: 100%;
}
</style>
