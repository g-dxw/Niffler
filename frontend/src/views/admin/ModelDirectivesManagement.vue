<template>
  <PageContainer>
    <PageHeader
      :title="t('admin.modelDirectives.title')"
      :description="t('admin.modelDirectives.description')"
    >
      <template #actions>
        <Button
          variant="outline"
          :disabled="loading"
          @click="loadConfig"
        >
          <RefreshCw
            class="w-4 h-4 mr-2"
            :class="{ 'animate-spin': loading }"
          />
          {{ t('admin.modelDirectives.refresh') }}
        </Button>
      </template>
    </PageHeader>

    <div class="mt-6 space-y-5">
      <Card
        variant="default"
        class="p-6"
      >
        <ModelDirectivesPanel
          :config="modelDirectivesConfig"
          :loading="loading || saving"
          @save="saveConfig"
          @update:config="modelDirectivesConfig = $event"
        />
      </Card>
    </div>
  </PageContainer>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { RefreshCw } from 'lucide-vue-next'
import Button from '@/components/ui/button.vue'
import Card from '@/components/ui/card.vue'
import { PageContainer, PageHeader } from '@/components/layout'
import { adminApi } from '@/api/admin'
import { useToast } from '@/composables/useToast'
import { log } from '@/utils/logger'
import { getErrorMessage } from '@/types/api-error'
import ModelDirectivesPanel from './module-management/ModelDirectivesPanel.vue'
import {
  createDefaultModelDirectivesConfig,
  normalizeModelDirectivesConfig,
  type ModelDirectivesConfig,
} from './module-management/modelDirectivesConfig'

const { t } = useI18n()
const { success, error } = useToast()

const modelDirectivesConfig = ref<ModelDirectivesConfig>(createDefaultModelDirectivesConfig())
const loading = ref(false)
const saving = ref(false)

async function loadConfig() {
  loading.value = true
  try {
    const response = await adminApi.getSystemConfig('model_directives')
    const normalized = normalizeModelDirectivesConfig(response.value)
    modelDirectivesConfig.value = normalized
  } catch (err) {
    error(t('admin.modelDirectives.loadFailed'))
    log.error('获取模型后缀参数配置失败:', err)
  } finally {
    loading.value = false
  }
}

async function saveConfig() {
  saving.value = true
  try {
    const normalized = normalizeModelDirectivesConfig(modelDirectivesConfig.value)
    modelDirectivesConfig.value = normalized
    await adminApi.updateSystemConfig(
      'model_directives',
      normalized,
      t('admin.modelDirectives.configName')
    )
    success(t('admin.modelDirectives.saved'))
  } catch (err) {
    error(getErrorMessage(err, t('admin.modelDirectives.saveFailed')))
    log.error('保存模型后缀参数配置失败:', err)
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadConfig()
})
</script>
