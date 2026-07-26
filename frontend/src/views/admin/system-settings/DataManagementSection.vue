<template>
  <CardSection
    :title="t('dataManagement.title')"
    :description="t('dataManagement.description')"
  >
    <div class="space-y-6">
      <div>
        <div class="flex items-center gap-2 mb-3">
          <Database class="w-4 h-4 text-muted-foreground" />
          <h4 class="text-sm font-medium">
            {{ t('dataManagement.transfer') }}
          </h4>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
          <div
            v-for="item in dataItems"
            :key="item.key"
            class="flex flex-col gap-3 p-4 rounded-lg border border-border"
          >
            <div class="flex items-center gap-2">
              <component
                :is="item.icon"
                class="w-4 h-4 text-muted-foreground"
              />
              <span class="text-sm font-medium">{{ item.title }}</span>
            </div>
            <p class="text-xs text-muted-foreground flex-1">
              {{ item.description }}
            </p>
            <div class="grid grid-cols-2 gap-2">
              <Button
                variant="outline"
                size="sm"
                class="w-full"
                :disabled="item.exportLoading"
                @click="$emit('export', item.key)"
              >
                <Download class="w-3.5 h-3.5 mr-1.5" />
                {{ item.exportLoading ? t('dataManagement.exporting') : item.exportLabel }}
              </Button>
              <Button
                variant="outline"
                size="sm"
                class="w-full"
                :disabled="item.importLoading"
                @click="triggerDataFileSelect(item.key)"
              >
                <Upload class="w-3.5 h-3.5 mr-1.5" />
                {{ item.importLoading ? t('dataManagement.importing') : item.importLabel }}
              </Button>
            </div>
          </div>
        </div>

        <input
          ref="configFileInput"
          type="file"
          accept=".json"
          class="hidden"
          @change="$emit('fileSelect', 'config', $event)"
        >
        <input
          ref="usersFileInput"
          type="file"
          accept=".json"
          class="hidden"
          @change="$emit('fileSelect', 'users', $event)"
        >
        <input
          ref="aggregateFileInput"
          type="file"
          accept=".json"
          class="hidden"
          @change="$emit('fileSelect', 'aggregate', $event)"
        >
      </div>

      <Separator />

      <div>
        <div class="flex items-center gap-2 mb-3">
          <Trash2 class="w-4 h-4 text-muted-foreground" />
          <h4 class="text-sm font-medium">
            {{ t('dataManagement.clear') }}
          </h4>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          <div
            v-for="item in purgeItems"
            :key="item.key"
            class="flex flex-col gap-2 p-4 rounded-lg border border-border"
          >
            <div class="flex items-center gap-2">
              <component
                :is="item.icon"
                class="w-4 h-4 text-muted-foreground"
              />
              <span class="text-sm font-medium">{{ item.title }}</span>
            </div>
            <p class="text-xs text-muted-foreground flex-1">
              {{ item.description }}
            </p>
            <Button
              variant="destructive"
              size="sm"
              class="w-full mt-1"
              :disabled="loadingKey === item.key"
              @click="handlePurge(item)"
            >
              <Trash2 class="w-3.5 h-3.5 mr-1.5" />
              {{ loadingKey === item.key ? t('dataManagement.clearing') : item.buttonText }}
            </Button>
          </div>
        </div>
      </div>
    </div>
  </CardSection>
</template>

<script setup lang="ts">
import { computed, ref, markRaw, type Component } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
import {
  Download,
  Upload,
  Settings,
  Users,
  Database,
  Layers3,
  Trash2,
  BarChart3,
  Shield,
  FileText,
  PieChart,
} from 'lucide-vue-next'
import Button from '@/components/ui/button.vue'
import { Separator } from '@/components/ui'
import { CardSection } from '@/components/layout'
import { adminApi } from '@/api/admin'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { parseApiError } from '@/utils/errorParser'

type DataItemKey = 'config' | 'users' | 'aggregate'

interface DataItem {
  key: DataItemKey
  title: string
  description: string
  exportLabel: string
  importLabel: string
  icon: Component
  exportLoading: boolean
  importLoading: boolean
}

interface PurgeItem {
  key: string
  title: string
  description: string
  buttonText: string
  icon: Component
  confirmMessage: string
  action: () => Promise<{ message: string }>
}

const props = defineProps<{
  configExportLoading: boolean
  configImportLoading: boolean
  usersExportLoading: boolean
  usersImportLoading: boolean
  aggregateExportLoading: boolean
  aggregateImportLoading: boolean
}>()

defineEmits<{
  export: [key: DataItemKey]
  fileSelect: [key: DataItemKey, event: Event]
}>()

const { success, error } = useToast()
const { confirmDanger } = useConfirm()
const loadingKey = ref<string | null>(null)
const configFileInput = ref<HTMLInputElement | null>(null)
const usersFileInput = ref<HTMLInputElement | null>(null)
const aggregateFileInput = ref<HTMLInputElement | null>(null)

const dataItems = computed<DataItem[]>(() => [
  {
    key: 'config',
    title: t('dataManagement.items.configTitle'),
    description: t('dataManagement.items.configDescription'),
    exportLabel: t('dataManagement.items.configExport'),
    importLabel: t('dataManagement.items.configImport'),
    icon: markRaw(Settings),
    exportLoading: props.configExportLoading,
    importLoading: props.configImportLoading,
  },
  {
    key: 'users',
    title: t('dataManagement.items.usersTitle'),
    description: t('dataManagement.items.usersDescription'),
    exportLabel: t('dataManagement.items.usersExport'),
    importLabel: t('dataManagement.items.usersImport'),
    icon: markRaw(Users),
    exportLoading: props.usersExportLoading,
    importLoading: props.usersImportLoading,
  },
  {
    key: 'aggregate',
    title: t('dataManagement.items.aggregateTitle'),
    description: t('dataManagement.items.aggregateDescription'),
    exportLabel: t('dataManagement.items.aggregateExport'),
    importLabel: t('dataManagement.items.aggregateImport'),
    icon: markRaw(Layers3),
    exportLoading: props.aggregateExportLoading,
    importLoading: props.aggregateImportLoading,
  },
])

const purgeItems = computed<PurgeItem[]>(() => [
  {
    key: 'config',
    title: t('dataManagement.purge.configTitle'),
    description: t('dataManagement.purge.configDescription'),
    buttonText: t('dataManagement.purge.configButton'),
    icon: markRaw(Settings),
    confirmMessage: t('dataManagement.purge.configConfirm'),
    action: () => adminApi.purgeConfig(),
  },
  {
    key: 'users',
    title: t('dataManagement.purge.usersTitle'),
    description: t('dataManagement.purge.usersDescription'),
    buttonText: t('dataManagement.purge.usersButton'),
    icon: markRaw(Users),
    confirmMessage: t('dataManagement.purge.usersConfirm'),
    action: () => adminApi.purgeUsers(),
  },
  {
    key: 'usage',
    title: t('dataManagement.purge.usageTitle'),
    description: t('dataManagement.purge.usageDescription'),
    buttonText: t('dataManagement.purge.usageButton'),
    icon: markRaw(BarChart3),
    confirmMessage: t('dataManagement.purge.usageConfirm'),
    action: () => adminApi.purgeUsage(),
  },
  {
    key: 'audit-logs',
    title: t('dataManagement.purge.auditTitle'),
    description: t('dataManagement.purge.auditDescription'),
    buttonText: t('dataManagement.purge.auditButton'),
    icon: markRaw(Shield),
    confirmMessage: t('dataManagement.purge.auditConfirm'),
    action: () => adminApi.purgeAuditLogs(),
  },
  {
    key: 'request-bodies',
    title: t('dataManagement.purge.bodiesTitle'),
    description: t('dataManagement.purge.bodiesDescription'),
    buttonText: t('dataManagement.purge.bodiesButton'),
    icon: markRaw(FileText),
    confirmMessage: t('dataManagement.purge.bodiesConfirm'),
    action: () => adminApi.purgeRequestBodiesAsync(),
  },
  {
    key: 'stats',
    title: t('dataManagement.purge.statsTitle'),
    description: t('dataManagement.purge.statsDescription'),
    buttonText: t('dataManagement.purge.statsButton'),
    icon: markRaw(PieChart),
    confirmMessage: t('dataManagement.purge.statsConfirm'),
    action: () => adminApi.purgeStats(),
  },
])

function triggerDataFileSelect(key: DataItemKey) {
  if (key === 'config') {
    configFileInput.value?.click()
  } else if (key === 'users') {
    usersFileInput.value?.click()
  } else {
    aggregateFileInput.value?.click()
  }
}

async function handlePurge(item: PurgeItem) {
  const confirmed = await confirmDanger(item.confirmMessage, item.title)
  if (!confirmed) return

  loadingKey.value = item.key
  try {
    const result = await item.action()
    success(result.message || t('common.success'))
  } catch (e) {
    error(parseApiError(e, t('dataManagement.clearFailed')))
  } finally {
    loadingKey.value = null
  }
}
</script>
