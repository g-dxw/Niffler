<template>
  <!-- 聚合数据导入对话框 -->
  <Dialog
    :open="aggregateImportDialogOpen"
    :title="t('importDialogs.aggregateTitle')"
    :description="t('importDialogs.hint')"
    @update:open="$emit('update:aggregateImportDialogOpen', $event)"
  >
    <div class="space-y-4">
      <div
        v-if="aggregateImportPreview"
        class="text-sm"
      >
        <p class="font-medium mb-2">
          {{ t('importDialogs.preview') }}
        </p>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-muted-foreground">
          <div>
            <p class="font-medium text-foreground mb-1">
              {{ t('importDialogs.configData') }}
            </p>
            <ul class="space-y-1">
              <li>{{ t('importDialogs.globalModels') }}: {{ aggregateImportPreview.config_data.global_models?.length || 0 }}</li>
              <li>{{ t('importDialogs.providers') }}: {{ aggregateImportPreview.config_data.providers?.length || 0 }}</li>
              <li>
                {{ t('importDialogs.keys') }}: {{ aggregateImportPreview.config_data.providers?.reduce((sum: number, p: { api_keys?: unknown[] }) => sum + (p.api_keys?.length || 0), 0) }}
              </li>
            </ul>
          </div>
          <div>
            <p class="font-medium text-foreground mb-1">
              {{ t('importDialogs.userData') }}
            </p>
            <ul class="space-y-1">
              <li v-if="aggregateImportPreview.user_data.user_groups?.length">
                {{ t('importDialogs.groups') }}: {{ aggregateImportPreview.user_data.user_groups.length }}
              </li>
              <li>{{ t('importDialogs.users') }}: {{ aggregateImportPreview.user_data.users?.length || 0 }}</li>
              <li>
                {{ t('importDialogs.keys') }}: {{ aggregateImportPreview.user_data.users?.reduce((sum: number, u: { api_keys?: unknown[] }) => sum + (u.api_keys?.length || 0), 0) }}
              </li>
              <li v-if="aggregateImportPreview.user_data.standalone_keys?.length">
                {{ t('importDialogs.standalone') }}: {{ aggregateImportPreview.user_data.standalone_keys.length }}
              </li>
            </ul>
          </div>
        </div>
      </div>

      <div>
        <Label class="block text-sm font-medium mb-2">{{ t('importDialogs.conflict') }}</Label>
        <Select
          :model-value="aggregateMergeMode"
          :open="aggregateMergeModeSelectOpen"
          @update:model-value="$emit('update:aggregateMergeMode', $event as 'skip' | 'overwrite' | 'error')"
          @update:open="$emit('update:aggregateMergeModeSelectOpen', $event)"
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
          <template v-if="aggregateMergeMode === 'skip'">
            {{ t('importDialogs.skipHint') }}
          </template>
          <template v-else-if="aggregateMergeMode === 'overwrite'">
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
        @click="$emit('update:aggregateImportDialogOpen', false); $emit('update:aggregateMergeModeSelectOpen', false)"
      >
        {{ t('importDialogs.cancel') }}
      </Button>
      <Button
        :disabled="importAggregateLoading"
        @click="$emit('confirm')"
      >
        {{ importAggregateLoading ? t('importDialogs.importing') : t('importDialogs.confirm') }}
      </Button>
    </template>
  </Dialog>

  <!-- 聚合数据导入结果对话框 -->
  <Dialog
    :open="aggregateImportResultDialogOpen"
    :title="t('importDialogs.complete')"
    @update:open="$emit('update:aggregateImportResultDialogOpen', $event)"
  >
    <div
      v-if="aggregateImportResult"
      class="space-y-4"
    >
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
        <div>
          <p class="font-medium">
            {{ t('importDialogs.configData') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.globalModelsCreated', { count: aggregateImportResult.config.stats.global_models.created }) }}，
            {{ t('importDialogs.providersCreated', { count: aggregateImportResult.config.stats.providers.created }) }}，
            {{ t('importDialogs.keysCreated', { count: aggregateImportResult.config.stats.keys.created }) }}
          </p>
        </div>
        <div>
          <p class="font-medium">
            {{ t('importDialogs.userData') }}
          </p>
          <p class="text-muted-foreground">
            {{ t('importDialogs.usersCreated', { count: aggregateImportResult.users.stats.users.created }) }}，
            {{ t('importDialogs.keysCreated', { count: aggregateImportResult.users.stats.api_keys.created }) }}，
            {{ t('importDialogs.usersSkipped', { count: aggregateImportResult.users.stats.users.skipped }) }}
          </p>
        </div>
      </div>

      <div
        v-if="warningMessages.length > 0"
        class="p-3 bg-destructive/10 rounded-lg"
      >
        <p class="font-medium text-destructive mb-2">
          {{ t('importDialogs.warnings') }}
        </p>
        <ul class="text-sm text-destructive space-y-1">
          <li
            v-for="(message, index) in warningMessages"
            :key="index"
          >
            {{ message }}
          </li>
        </ul>
      </div>
    </div>

    <template #footer>
      <Button @click="$emit('update:aggregateImportResultDialogOpen', false)">
        {{ t('importDialogs.confirmDone') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
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
import type { AggregateExportData, AggregateImportResponse } from '@/api/admin'

const props = defineProps<{
  aggregateImportDialogOpen: boolean
  aggregateImportResultDialogOpen: boolean
  aggregateImportPreview: AggregateExportData | null
  aggregateImportResult: AggregateImportResponse | null
  aggregateMergeMode: 'skip' | 'overwrite' | 'error'
  aggregateMergeModeSelectOpen: boolean
  importAggregateLoading: boolean
}>()

defineEmits<{
  confirm: []
  'update:aggregateImportDialogOpen': [value: boolean]
  'update:aggregateImportResultDialogOpen': [value: boolean]
  'update:aggregateMergeMode': [value: 'skip' | 'overwrite' | 'error']
  'update:aggregateMergeModeSelectOpen': [value: boolean]
}>()

const warningMessages = computed(() => {
  if (!props.aggregateImportResult) return []
  const configErrors = props.aggregateImportResult.config.stats.errors.map((message) => `${t('importDialogs.configData')}: ${message}`)
  const userErrors = props.aggregateImportResult.users.stats.errors.map((message) => `${t('importDialogs.userData')}: ${message}`)
  return [...configErrors, ...userErrors]
})
</script>
