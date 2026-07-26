import { ref } from 'vue'
import { useToast } from '@/composables/useToast'
import {
  adminApi,
  type AggregateExportData,
  type AggregateImportResponse,
  type ConfigExportData,
  type ConfigImportResponse,
  type UsersExportData,
  type UsersImportResponse,
} from '@/api/admin'
import { parseApiError } from '@/utils/errorParser'
import { log } from '@/utils/logger'
import { i18n } from '@/i18n'

// 文件大小限制：导出文件可能包含大量 Provider Key、模型和用户数据。
const BYTES_PER_MB = 1024 * 1024
const MAX_FILE_SIZE_MB = 500
const MAX_AGGREGATE_FILE_SIZE_MB = 500
const MAX_FILE_SIZE = MAX_FILE_SIZE_MB * BYTES_PER_MB
const MAX_AGGREGATE_FILE_SIZE = MAX_AGGREGATE_FILE_SIZE_MB * BYTES_PER_MB

type JsonObject = Record<string, unknown>

function asJsonObject(value: unknown): JsonObject | null {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? value as JsonObject
    : null
}

function hasArrayField(value: JsonObject, key: string): boolean {
  return Array.isArray(value[key])
}

function looksLikeConfigExport(value: JsonObject): boolean {
  return hasArrayField(value, 'global_models')
    || hasArrayField(value, 'providers')
    || hasArrayField(value, 'proxy_nodes')
    || hasArrayField(value, 'oauth_providers')
    || hasArrayField(value, 'system_configs')
    || Object.prototype.hasOwnProperty.call(value, 'ldap_config')
}

function looksLikeUsersExport(value: JsonObject): boolean {
  return hasArrayField(value, 'users')
    || hasArrayField(value, 'standalone_keys')
    || hasArrayField(value, 'user_groups')
}

function looksLikeAggregateExport(value: JsonObject): boolean {
  return asJsonObject(value.config_data) != null
    && asJsonObject(value.user_data) != null
}

function downloadJson(data: unknown, filename: string) {
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

export function useConfigExportImport(systemConfig: { value: { site_name: string } }) {
  const { success, error } = useToast()
  const t = i18n.global.t

  // 配置导出/导入相关
  const exportLoading = ref(false)
  const importLoading = ref(false)
  const importDialogOpen = ref(false)
  const importResultDialogOpen = ref(false)
  const configFileInput = ref<HTMLInputElement | null>(null)
  const importPreview = ref<ConfigExportData | null>(null)
  const importResult = ref<ConfigImportResponse | null>(null)
  const mergeMode = ref<'skip' | 'overwrite' | 'error'>('skip')
  const mergeModeSelectOpen = ref(false)

  // 用户数据导出/导入相关
  const exportUsersLoading = ref(false)
  const importUsersLoading = ref(false)
  const importUsersDialogOpen = ref(false)
  const importUsersResultDialogOpen = ref(false)
  const usersFileInput = ref<HTMLInputElement | null>(null)
  const importUsersPreview = ref<UsersExportData | null>(null)
  const importUsersResult = ref<UsersImportResponse | null>(null)
  const usersMergeMode = ref<'skip' | 'overwrite' | 'error'>('skip')
  const usersMergeModeSelectOpen = ref(false)

  // 聚合数据导出/导入相关
  const exportAggregateLoading = ref(false)
  const importAggregateLoading = ref(false)
  const aggregateImportDialogOpen = ref(false)
  const aggregateImportResultDialogOpen = ref(false)
  const aggregateImportPreview = ref<AggregateExportData | null>(null)
  const aggregateImportResult = ref<AggregateImportResponse | null>(null)
  const aggregateMergeMode = ref<'skip' | 'overwrite' | 'error'>('skip')
  const aggregateMergeModeSelectOpen = ref(false)

  // 导出配置
  async function handleExportConfig() {
    exportLoading.value = true
    try {
      const data = await adminApi.exportConfig()
      downloadJson(
        data,
        `${systemConfig.value.site_name.toLowerCase()}-config-${new Date().toISOString().slice(0, 10)}.json`,
      )
      success(t('configTransfer.configExported'))
    } catch (err) {
      error(t('configTransfer.configExportFailed'))
      log.error('导出配置失败:', err)
    } finally {
      exportLoading.value = false
    }
  }

  // 触发文件选择
  function triggerConfigFileSelect() {
    configFileInput.value?.click()
  }

  // 处理文件选择
  function handleConfigFileSelect(event: Event) {
    const input = event.target as HTMLInputElement
    const file = input.files?.[0]
    if (!file) return

    if (file.size > MAX_FILE_SIZE) {
      error(t('configTransfer.fileTooLarge', { size: MAX_FILE_SIZE_MB }))
      input.value = ''
      return
    }

    const reader = new FileReader()
    reader.onload = (e) => {
      try {
        const content = e.target?.result as string
        const root = asJsonObject(JSON.parse(content))
        if (!root) {
          error(t('configTransfer.invalidConfigObject'))
          return
        }

        if (looksLikeUsersExport(root) && !looksLikeConfigExport(root)) {
          error(t('configTransfer.useUsersImport'))
          return
        }

        if (!root.version) {
          error(t('configTransfer.configVersionMissing'))
          return
        }

        if (!looksLikeConfigExport(root)) {
          error(t('configTransfer.configContentMissing'))
          return
        }

        const data = root as unknown as ConfigExportData
        importPreview.value = data
        mergeMode.value = 'skip'
        importDialogOpen.value = true
      } catch (err) {
        error(t('configTransfer.configParseFailed'))
        log.error('解析配置文件失败:', err)
      }
    }
    reader.readAsText(file)

    input.value = ''
  }

  // 确认导入
  async function confirmImport() {
    if (!importPreview.value) return

    importLoading.value = true
    try {
      const result = await adminApi.importConfig({
        ...importPreview.value,
        merge_mode: mergeMode.value,
      })
      importResult.value = result
      importDialogOpen.value = false
      mergeModeSelectOpen.value = false
      importResultDialogOpen.value = true
      success(t('configTransfer.configImported'))
    } catch (err: unknown) {
      error(parseApiError(err, t('configTransfer.configImportFailed')))
      log.error('导入配置失败:', err)
    } finally {
      importLoading.value = false
    }
  }

  // 导出用户数据
  async function handleExportUsers() {
    exportUsersLoading.value = true
    try {
      const data = await adminApi.exportUsers()
      downloadJson(
        data,
        `${systemConfig.value.site_name.toLowerCase()}-users-${new Date().toISOString().slice(0, 10)}.json`,
      )
      success(t('configTransfer.usersExported'))
    } catch (err) {
      error(t('configTransfer.usersExportFailed'))
      log.error('导出用户数据失败:', err)
    } finally {
      exportUsersLoading.value = false
    }
  }

  // 触发用户数据文件选择
  function triggerUsersFileSelect() {
    usersFileInput.value?.click()
  }

  // 处理用户数据文件选择
  function handleUsersFileSelect(event: Event) {
    const input = event.target as HTMLInputElement
    const file = input.files?.[0]
    if (!file) return

    if (file.size > MAX_FILE_SIZE) {
      error(t('configTransfer.fileTooLarge', { size: MAX_FILE_SIZE_MB }))
      input.value = ''
      return
    }

    const reader = new FileReader()
    reader.onload = (e) => {
      try {
        const content = e.target?.result as string
        const root = asJsonObject(JSON.parse(content))
        if (!root) {
          error(t('configTransfer.invalidUsersObject'))
          return
        }

        if (looksLikeConfigExport(root) && !looksLikeUsersExport(root)) {
          error(t('configTransfer.useConfigImport'))
          return
        }

        if (!root.version) {
          error(t('configTransfer.usersVersionMissing'))
          return
        }

        if (!Array.isArray(root.users)) {
          error(t('configTransfer.usersArrayMissing'))
          return
        }

        if (root.user_groups != null && !Array.isArray(root.user_groups)) {
          error(t('configTransfer.userGroupsInvalid'))
          return
        }

        if (root.standalone_keys != null && !Array.isArray(root.standalone_keys)) {
          error(t('configTransfer.standaloneKeysInvalid'))
          return
        }

        const data = root as unknown as UsersExportData
        importUsersPreview.value = data
        usersMergeMode.value = 'skip'
        importUsersDialogOpen.value = true
      } catch (err) {
        error(t('configTransfer.usersParseFailed'))
        log.error('解析用户数据文件失败:', err)
      }
    }
    reader.readAsText(file)

    input.value = ''
  }

  // 确认导入用户数据
  async function confirmImportUsers() {
    if (!importUsersPreview.value) return

    importUsersLoading.value = true
    try {
      const result = await adminApi.importUsers({
        ...importUsersPreview.value,
        merge_mode: usersMergeMode.value,
      })
      importUsersResult.value = result
      importUsersDialogOpen.value = false
      usersMergeModeSelectOpen.value = false
      importUsersResultDialogOpen.value = true
      success(t('configTransfer.usersImported'))
    } catch (err: unknown) {
      error(parseApiError(err, t('configTransfer.usersImportFailed')))
      log.error('导入用户数据失败:', err)
    } finally {
      importUsersLoading.value = false
    }
  }

  // 导出聚合数据
  async function handleExportAggregate() {
    exportAggregateLoading.value = true
    try {
      const data = await adminApi.exportAggregateData()
      downloadJson(
        data,
        `${systemConfig.value.site_name.toLowerCase()}-data-${new Date().toISOString().slice(0, 10)}.json`,
      )
      success(t('configTransfer.aggregateExported'))
    } catch (err) {
      error(t('configTransfer.aggregateExportFailed'))
      log.error('导出聚合数据失败:', err)
    } finally {
      exportAggregateLoading.value = false
    }
  }

  // 处理聚合数据文件选择
  function handleAggregateFileSelect(event: Event) {
    const input = event.target as HTMLInputElement
    const file = input.files?.[0]
    if (!file) return

    if (file.size > MAX_AGGREGATE_FILE_SIZE) {
      error(t('configTransfer.fileTooLarge', { size: MAX_AGGREGATE_FILE_SIZE_MB }))
      input.value = ''
      return
    }

    const reader = new FileReader()
    reader.onload = (e) => {
      try {
        const content = e.target?.result as string
        const root = asJsonObject(JSON.parse(content))
        if (!root) {
          error(t('configTransfer.invalidAggregateObject'))
          return
        }

        if (!looksLikeAggregateExport(root)) {
          if (looksLikeConfigExport(root)) {
            error(t('configTransfer.useConfigDataImport'))
          } else if (looksLikeUsersExport(root)) {
            error(t('configTransfer.useUsersImport'))
          } else {
            error(t('configTransfer.aggregateContentMissing'))
          }
          return
        }

        if (!root.version) {
          error(t('configTransfer.aggregateVersionMissing'))
          return
        }

        const configData = asJsonObject(root.config_data)
        const userData = asJsonObject(root.user_data)
        if (!configData || !looksLikeConfigExport(configData)) {
          error(t('configTransfer.configDataInvalid'))
          return
        }
        if (!userData || !looksLikeUsersExport(userData)) {
          error(t('configTransfer.userDataInvalid'))
          return
        }

        const data = root as unknown as AggregateExportData
        aggregateImportPreview.value = data
        aggregateMergeMode.value = 'skip'
        aggregateImportDialogOpen.value = true
      } catch (err) {
        error(t('configTransfer.aggregateParseFailed'))
        log.error('解析聚合数据文件失败:', err)
      }
    }
    reader.readAsText(file)

    input.value = ''
  }

  // 确认导入聚合数据
  async function confirmImportAggregate() {
    if (!aggregateImportPreview.value) return

    importAggregateLoading.value = true
    try {
      const result = await adminApi.importAggregateData({
        ...aggregateImportPreview.value,
        merge_mode: aggregateMergeMode.value,
      })
      aggregateImportResult.value = result
      aggregateImportDialogOpen.value = false
      aggregateMergeModeSelectOpen.value = false
      aggregateImportResultDialogOpen.value = true
      success(t('configTransfer.aggregateImported'))
    } catch (err: unknown) {
      error(parseApiError(err, t('configTransfer.aggregateImportFailed')))
      log.error('导入聚合数据失败:', err)
    } finally {
      importAggregateLoading.value = false
    }
  }

  return {
    // 配置导出/导入
    exportLoading,
    importLoading,
    importDialogOpen,
    importResultDialogOpen,
    configFileInput,
    importPreview,
    importResult,
    mergeMode,
    mergeModeSelectOpen,
    handleExportConfig,
    triggerConfigFileSelect,
    handleConfigFileSelect,
    confirmImport,
    // 用户数据导出/导入
    exportUsersLoading,
    importUsersLoading,
    importUsersDialogOpen,
    importUsersResultDialogOpen,
    usersFileInput,
    importUsersPreview,
    importUsersResult,
    usersMergeMode,
    usersMergeModeSelectOpen,
    handleExportUsers,
    triggerUsersFileSelect,
    handleUsersFileSelect,
    confirmImportUsers,
    // 聚合数据导出/导入
    exportAggregateLoading,
    importAggregateLoading,
    aggregateImportDialogOpen,
    aggregateImportResultDialogOpen,
    aggregateImportPreview,
    aggregateImportResult,
    aggregateMergeMode,
    aggregateMergeModeSelectOpen,
    handleExportAggregate,
    handleAggregateFileSelect,
    confirmImportAggregate,
  }
}
