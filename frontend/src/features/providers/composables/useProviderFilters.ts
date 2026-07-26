import { ref, computed, watch } from 'vue'
import type { ProviderSummaryQuery } from '@/api/endpoints'
import { i18n } from '@/i18n'

export interface FilterOption {
  value: string
  label: string
}

export function useProviderFilters(
  globalModels: () => { id: string; name: string }[],
) {
  // 搜索与筛选
  const searchQuery = ref('')
  const filterStatus = ref('all')
  const filterApiFormat = ref('all')
  const filterModel = ref('all')

  const statusFilters = computed<FilterOption[]>(() => [
    { value: 'all', label: i18n.global.t('providerUi.allStatuses') },
    { value: 'active', label: i18n.global.t('providerUi.active') },
    { value: 'inactive', label: i18n.global.t('providerUi.inactive') },
  ])

  const apiFormatFilters = computed<FilterOption[]>(() => [
    { value: 'all', label: i18n.global.t('providerUi.allFormats') },
    { value: 'claude:messages', label: 'Claude Messages' },
    { value: 'openai:chat', label: 'OpenAI Chat' },
    { value: 'openai:responses', label: 'OpenAI Responses' },
    { value: 'openai:responses:compact', label: 'OpenAI Responses Compact' },
    { value: 'openai:embedding', label: 'OpenAI Embedding' },
    { value: 'openai:rerank', label: 'OpenAI Rerank' },
    { value: 'gemini:generate_content', label: 'Gemini Generate Content' },
    { value: 'gemini:embedding', label: 'Gemini Embedding' },
    { value: 'jina:embedding', label: 'Jina Embedding' },
    { value: 'jina:rerank', label: 'Jina Rerank' },
    { value: 'doubao:embedding', label: 'Doubao Embedding' },
  ])

  const modelFilters = computed<FilterOption[]>(() => {
    const items = globalModels()
      .map(m => ({ value: m.id, label: m.name }))
      .sort((a, b) => a.label.localeCompare(b.label))
    return [{ value: 'all', label: i18n.global.t('providerUi.allModels') }, ...items]
  })

  const hasActiveFilters = computed(() => {
    return (
      searchQuery.value !== '' ||
      filterStatus.value !== 'all' ||
      filterApiFormat.value !== 'all' ||
      filterModel.value !== 'all'
    )
  })

  // 分页
  const currentPage = ref(1)
  const pageSize = ref(20)
  const total = ref(0)

  // 服务端分页查询参数
  const queryParams = computed<ProviderSummaryQuery>(() => ({
    page: currentPage.value,
    page_size: pageSize.value,
    search: searchQuery.value.trim() || undefined,
    status: filterStatus.value !== 'all' ? filterStatus.value : undefined,
    api_format: filterApiFormat.value !== 'all' ? filterApiFormat.value : undefined,
    model_id: filterModel.value !== 'all' ? filterModel.value : undefined,
  }))

  // 搜索/筛选变化时重置分页到第1页
  watch([searchQuery, filterStatus, filterApiFormat, filterModel], () => {
    currentPage.value = 1
  })

  function resetFilters() {
    searchQuery.value = ''
    filterStatus.value = 'all'
    filterApiFormat.value = 'all'
    filterModel.value = 'all'
  }

  return {
    searchQuery,
    filterStatus,
    filterApiFormat,
    filterModel,
    statusFilters,
    apiFormatFilters,
    modelFilters,
    hasActiveFilters,
    currentPage,
    pageSize,
    total,
    queryParams,
    resetFilters,
  }
}
