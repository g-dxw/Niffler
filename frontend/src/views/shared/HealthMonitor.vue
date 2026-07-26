<template>
  <div class="space-y-6 pb-8">
    <HealthMonitorCard
      :title="t('healthMonitor.title')"
      :is-admin="isAdminPage"
      :show-provider-info="isAdminPage"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import HealthMonitorCard from '@/features/providers/components/HealthMonitorCard.vue'

const route = useRoute()
const { t } = useI18n()
const isAdminPage = computed(() => route.path.startsWith('/admin'))
</script>
