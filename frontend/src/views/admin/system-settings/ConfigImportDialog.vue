<template>
  <!-- 导入配置对话框 -->
  <Dialog
    :open="importDialogOpen"
    :title="t('importDialogs.configTitle')"
    :description="t('importDialogs.hint')"
    @update:open="$emit('update:importDialogOpen', $event)"
  >
    <div class="space-y-4">
      <div
        v-if="importPreview"
        class="text-sm"
      >
        <p class="font-medium mb-2">
          {{ t('importDialogs.preview') }}
        </p>
        <ul class="space-y-1 text-muted-foreground">
          <li>{{ t('importDialogs.globalModels') }}: {{ importPreview.global_models?.length || 0 }}</li>
          <li>{{ t('importDialogs.providers') }}: {{ importPreview.providers?.length || 0 }}</li>
          <li>
            {{ t('importDialogs.endpoints') }}: {{ importPreview.providers?.reduce((sum: number, p: { endpoints?: unknown[] }) => sum + (p.endpoints?.length || 0), 0) }}
          </li>
          <li>
            {{ t('importDialogs.keys') }}: {{ importPreview.providers?.reduce((sum: number, p: { api_keys?: unknown[] }) => sum + (p.api_keys?.length || 0), 0) }}
          </li>
          <li v-if="importPreview.proxy_nodes?.length">
            {{ t('importDialogs.proxyNodes') }}: {{ importPreview.proxy_nodes.length }}
          </li>
          <li v-if="importPreview.ldap_config">
            {{ t('importDialogs.ldap') }}: 1
          </li>
          <li v-if="importPreview.oauth_providers?.length">
            {{ t('importDialogs.oauth') }}: {{ importPreview.oauth_providers.length }}
          </li>
        </ul>
      </div>

      <div>
        <Label class="block text-sm font-medium mb-2">{{ t('importDialogs.conflict') }}</Label>
        <Select
          :model-value="mergeMode"
          :open="mergeModeSelectOpen"
          @update:model-value="$emit('update:mergeMode', $event as 'skip' | 'overwrite' | 'error')"
          @update:open="$emit('update:mergeModeSelectOpen', $event)"
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="skip">
              {{ t('importDialogs.skip') }}
            </SelectItem>
            <SelectItem value="overwrite">
              {{ t('importDialogs.overwrite') }}
            </SelectItem>
            <SelectItem value="error">
              {{ t('importDialogs.abort') }}
            </SelectItem>
          </SelectContent>
        </Select>
        <p class="mt-1 text-xs text-muted-foreground">
          <template v-if="mergeMode === 'skip'">
            {{ t('importDialogs.skipHint') }}
          </template>
          <template v-else-if="mergeMode === 'overwrite'">
            {{ t('importDialogs.overwriteHint') }}
          </template>
          <template v-else>
            {{ t('importDialogs.abortHint') }}
          </template>
        </p>
      </div>

      <p class="text-xs text-muted-foreground">
        {{ t('importDialogs.warning') }}
      </p>
    </div>

    <template #footer>
      <Button
        variant="outline"
        @click="$emit('update:importDialogOpen', false); $emit('update:mergeModeSelectOpen', false)"
      >
        {{ t('importDialogs.cancel') }}
      </Button>
      <Button
        :disabled="importLoading"
        @click="$emit('confirm')"
      >
        {{ importLoading ? t('importDialogs.importing') : t('importDialogs.confirm') }}
      </Button>
    </template>
  </Dialog>

  <!-- 导入结果对话框 -->
  <Dialog
    :open="importResultDialogOpen"
    :title="t('importDialogs.complete')"
    @update:open="$emit('update:importResultDialogOpen', $event)"
  >
    <div
      v-if="importResult"
      class="space-y-4"
    >
      <div class="grid grid-cols-2 gap-4 text-sm">
        <div>
          <p class="font-medium">
            {{ t('importDialogs.globalModels') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.created') }}: {{ importResult.stats.global_models.created }},
            {{ t('importDialogs.updated') }}: {{ importResult.stats.global_models.updated }},
            {{ t('importDialogs.skipped') }}: {{ importResult.stats.global_models.skipped }}
          </p>
        </div>
        <div>
          <p class="font-medium">
            {{ t('importDialogs.providers') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.created') }}: {{ importResult.stats.providers.created }},
            {{ t('importDialogs.updated') }}: {{ importResult.stats.providers.updated }},
            {{ t('importDialogs.skipped') }}: {{ importResult.stats.providers.skipped }}
          </p>
        </div>
        <div>
          <p class="font-medium">
            {{ t('importDialogs.endpoints') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.created') }}: {{ importResult.stats.endpoints.created }},
            {{ t('importDialogs.updated') }}: {{ importResult.stats.endpoints.updated }},
            {{ t('importDialogs.skipped') }}: {{ importResult.stats.endpoints.skipped }}
          </p>
        </div>
        <div>
          <p class="font-medium">
            API Keys
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.created') }}: {{ importResult.stats.keys.created }},
            {{ t('importDialogs.skipped') }}: {{ importResult.stats.keys.skipped }}
          </p>
        </div>
        <div class="col-span-2">
          <p class="font-medium">
            {{ t('importDialogs.modelConfig') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.created') }}: {{ importResult.stats.models.created }},
            {{ t('importDialogs.updated') }}: {{ importResult.stats.models.updated }},
            {{ t('importDialogs.skipped') }}: {{ importResult.stats.models.skipped }}
          </p>
        </div>
        <div v-if="importResult.stats.ldap">
          <p class="font-medium">
            {{ t('importDialogs.ldap') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.created') }}: {{ importResult.stats.ldap.created }},
            {{ t('importDialogs.updated') }}: {{ importResult.stats.ldap.updated }},
            {{ t('importDialogs.skipped') }}: {{ importResult.stats.ldap.skipped }}
          </p>
        </div>
        <div v-if="importResult.stats.oauth">
          <p class="font-medium">
            OAuth Providers
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.created') }}: {{ importResult.stats.oauth.created }},
            {{ t('importDialogs.updated') }}: {{ importResult.stats.oauth.updated }},
            {{ t('importDialogs.skipped') }}: {{ importResult.stats.oauth.skipped }}
          </p>
        </div>
        <div v-if="importResult.stats.proxy_nodes">
          <p class="font-medium">
            {{ t('importDialogs.proxyNodes') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.created') }}: {{ importResult.stats.proxy_nodes.created }},
            {{ t('importDialogs.updated') }}: {{ importResult.stats.proxy_nodes.updated }},
            {{ t('importDialogs.skipped') }}: {{ importResult.stats.proxy_nodes.skipped }}
          </p>
        </div>
      </div>

      <div
        v-if="importResult.stats.errors.length > 0"
        class="p-3 bg-destructive/10 rounded-lg"
      >
        <p class="font-medium text-destructive mb-2">
          {{ t('importDialogs.warnings') }}
        </p>
        <ul class="text-sm text-destructive space-y-1">
          <li
            v-for="(err, index) in importResult.stats.errors"
            :key="index"
          >
            {{ err }}
          </li>
        </ul>
      </div>
    </div>

    <template #footer>
      <Button @click="$emit('update:importResultDialogOpen', false)">
        {{ t('importDialogs.confirmDone') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
import Button from '@/components/ui/button.vue'
import Label from '@/components/ui/label.vue'
import Select from '@/components/ui/select.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectValue from '@/components/ui/select-value.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import { Dialog } from '@/components/ui'
import type { ConfigExportData, ConfigImportResponse } from '@/api/admin'

defineProps<{
  importDialogOpen: boolean
  importResultDialogOpen: boolean
  importPreview: ConfigExportData | null
  importResult: ConfigImportResponse | null
  mergeMode: 'skip' | 'overwrite' | 'error'
  mergeModeSelectOpen: boolean
  importLoading: boolean
}>()

defineEmits<{
  confirm: []
  'update:importDialogOpen': [value: boolean]
  'update:importResultDialogOpen': [value: boolean]
  'update:mergeMode': [value: 'skip' | 'overwrite' | 'error']
  'update:mergeModeSelectOpen': [value: boolean]
}>()
</script>
