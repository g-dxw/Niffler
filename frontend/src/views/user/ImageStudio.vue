<template>
  <div class="space-y-6 pb-8">
    <div class="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
      <div>
        <div class="flex items-center gap-2">
          <div class="flex h-10 w-10 items-center justify-center rounded-2xl bg-primary/10 text-primary">
            <WandSparkles class="h-5 w-5" />
          </div>
          <div>
            <h1 class="text-xl font-semibold tracking-tight">{{ t('imageStudio.title') }}</h1>
            <p class="text-sm text-muted-foreground">{{ t('imageStudio.description') }}</p>
          </div>
        </div>
      </div>
      <div class="flex items-center gap-2 text-xs text-muted-foreground">
        <span v-if="runningCount">{{ runningCount }} {{ t('imageStudio.running') }}</span>
        <span v-if="pendingCount">{{ pendingCount }} {{ t('imageStudio.pending') }}</span>
        <Button
          v-if="tasks.length"
          variant="outline"
          size="sm"
          @click="requestClearTasks"
        >
          <Trash2 class="mr-1.5 h-3.5 w-3.5" />{{ t('imageStudio.clearTasks') }}
        </Button>
      </div>
    </div>

    <div class="grid items-start gap-6 xl:grid-cols-[380px_minmax(0,1fr)]">
      <aside class="xl:sticky xl:top-6">
        <ImageGenerationForm
          :settings="settings"
          :form="form"
          :api-keys="apiKeys"
          :models="models"
          :loading="resourceLoading"
          @update:settings="settings = $event"
          @update:form="form = $event"
          @submit="handleSubmit"
          @refresh="loadResources"
          @error="showError($event)"
        />
      </aside>

      <section class="min-w-0 space-y-4">
        <Card class="flex flex-col gap-3 px-5 py-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 class="font-semibold">
              {{ t('imageStudio.tasks') }}
            </h2>
          </div>
          <div class="flex items-center gap-2">
            <Badge variant="secondary">
              {{ tasks.length }} {{ t('imageStudio.taskCount') }}
            </Badge>
            <RouterLink
              to="/dashboard/usage"
              class="text-xs text-primary hover:underline"
            >
              {{ t('imageStudio.usage') }}
            </RouterLink>
          </div>
        </Card>

        <div
          v-if="tasks.length"
          class="grid gap-4 sm:grid-cols-2 2xl:grid-cols-3"
        >
          <ImageTaskCard
            v-for="task in orderedTasks"
            :key="task.id"
            :task="task"
            @preview="openPreview"
            @download="handleDownload"
            @cancel="cancelTask"
            @retry="handleRetry"
            @remove="requestTaskRemoval"
          />
        </div>

        <Card
          v-else
          class="flex min-h-96 flex-col items-center justify-center border-dashed p-8 text-center"
        >
          <Images class="h-12 w-12 text-muted-foreground/40" />
          <h3 class="mt-4 font-semibold">
              {{ t('imageStudio.empty') }}
          </h3>
          <p class="mt-1 max-w-sm text-sm text-muted-foreground">
              {{ t('imageStudio.emptyHint') }}
          </p>
          <div class="mt-5 flex gap-3 text-xs">
            <RouterLink
              to="/dashboard/api-keys"
              class="text-primary hover:underline"
            >
              {{ t('imageStudio.manageKeys') }}
            </RouterLink>
            <RouterLink
              to="/dashboard/models"
              class="text-primary hover:underline"
            >
              {{ t('imageStudio.browseModels') }}
            </RouterLink>
          </div>
        </Card>
      </section>
    </div>

    <ImagePreviewDialog
      v-model:open="previewOpen"
      :task="previewTask"
      @download="handleDownload"
    />

    <AlertDialog
      v-model="deleteDialogOpen"
      type="danger"
      :title="deleteMode === 'all' ? t('imageStudio.deleteAll') : t('imageStudio.deleteTask')"
      :description="deleteDescription"
      :confirm-text="deleteMode === 'all' ? t('imageStudio.confirmClear') : t('imageStudio.delete')"
      :loading="deleting"
      @confirm="confirmDeletion"
      @cancel="resetDeleteDialog"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Images, Trash2, WandSparkles } from 'lucide-vue-next'
import { Badge, Button, Card } from '@/components/ui'
import { AlertDialog } from '@/components/common'
import { meApi } from '@/api/me'
import { useAuthStore } from '@/stores/auth'
import { useToast } from '@/composables/useToast'
import { log } from '@/utils/logger'
import ImageGenerationForm from '@/features/image-studio/components/ImageGenerationForm.vue'
import ImagePreviewDialog from '@/features/image-studio/components/ImagePreviewDialog.vue'
import ImageTaskCard from '@/features/image-studio/components/ImageTaskCard.vue'
import { DEFAULT_IMAGE_GENERATION_FORM } from '@/features/image-studio/constants'
import { useImageTasks } from '@/features/image-studio/composables/useImageTasks'
import type {
  ImageApiKeyOption,
  ImageGenerationForm as ImageGenerationFormState,
  ImageModelOption,
  ImageStudioSettings,
  ImageTask,
} from '@/features/image-studio/types'
import { parseAdvancedParams } from '@/features/image-studio/utils/advanced-params'
import { downloadImage } from '@/features/image-studio/utils/image-download'
import { isImageGenerationModel } from '@/features/image-studio/utils/model-capability'
import { loadImageSettings, saveImageSettings } from '@/features/image-studio/utils/storage'
import { resolveImageApiBaseUrl } from '@/features/image-studio/utils/base-url'
import { createImageSubmissionSnapshot } from '@/features/image-studio/utils/submission'

const authStore = useAuthStore()
const { t } = useI18n()
const { error: showError, success, warning } = useToast()
const userId = authStore.user?.id || 'current-session'

const settings = ref<ImageStudioSettings>(loadImageSettings(userId))
const form = ref<ImageGenerationFormState>({ ...DEFAULT_IMAGE_GENERATION_FORM, inputImages: [] })
const apiKeys = ref<ImageApiKeyOption[]>([])
const models = ref<ImageModelOption[]>([])
const baseUrl = ref('')
const credentialCache = new Map<string, string>()
const resourceLoading = ref(false)
const previewOpen = ref(false)
const previewTask = ref<ImageTask | null>(null)
const deleteDialogOpen = ref(false)
const deleteMode = ref<'task' | 'all'>('task')
const deleteTargetId = ref('')
const deleting = ref(false)

const {
  tasks,
  runningCount,
  pendingCount,
  addTasks,
  cancelTask,
  retryTask,
  removeTask,
  clearTasks,
} = useImageTasks({ userId, settings })

const orderedTasks = computed(() => [...tasks.value].sort((a, b) => b.createdAt - a.createdAt))
const deleteTarget = computed(() => tasks.value.find(task => task.id === deleteTargetId.value))
const deleteDescription = computed(() => {
  if (deleteMode.value === 'all') {
    return t('imageStudioExtra.clearConfirm', { count: tasks.value.length })
  }
  const runningHint = deleteTarget.value?.status === 'running' || deleteTarget.value?.status === 'pending'
    ? t('imageStudioExtra.runningDeleteHint')
    : ''
  return t('imageStudioExtra.deleteConfirm', { runningHint })
})

watch(settings, value => saveImageSettings(userId, value), { deep: true })
watch(tasks, (value) => {
  if (!previewTask.value) return
  const current = value.find(task => task.id === previewTask.value?.id)
  if (!current || current.imageUrl !== previewTask.value.imageUrl) {
    previewOpen.value = false
    previewTask.value = null
  }
}, { deep: true })

async function loadResources() {
  resourceLoading.value = true
  credentialCache.clear()
  try {
    const [keysResponse, modelsResponse, resolvedBaseUrl] = await Promise.all([
      meApi.getApiKeys(),
      meApi.getAvailableModels({ limit: 1000 }),
      resolveImageApiBaseUrl({
        isDev: import.meta.env.DEV,
        origin: window.location.origin,
        getPublicBaseUrl: () => meApi.getPublicBaseUrl(),
        onFallback: error => log.warn('获取公开 API 地址失败，已回退到同源地址:', error),
      }),
    ])

    apiKeys.value = keysResponse
      .filter(key => key.is_active && !key.is_locked)
      .map(key => ({ id: key.id, name: key.name, display: key.key_display }))
    models.value = modelsResponse.models
      .filter(model => model.is_active && isImageGenerationModel(model))
      .map(model => ({
        id: model.id,
        name: model.name,
        displayName: model.display_name || model.name,
      }))
    baseUrl.value = resolvedBaseUrl

    if (!apiKeys.value.some(key => key.id === settings.value.selectedKeyId)) {
      settings.value.selectedKeyId = apiKeys.value[0]?.id || ''
    }
    if (!models.value.some(model => model.name === settings.value.model)) {
      settings.value.model = models.value[0]?.name || ''
    }
  } catch (error) {
    log.error('加载生图资源失败:', error)
    showError(error instanceof Error ? error.message : t('imageStudioExtra.loadResourcesFailed'), t('imageStudioExtra.loadFailed'))
  } finally {
    resourceLoading.value = false
  }
}

async function getTaskCredential(apiKeyId = settings.value.selectedKeyId) {
  if (!apiKeyId) throw new Error(t('imageStudioExtra.selectKey'))
  if (!apiKeys.value.some(key => key.id === apiKeyId)) {
    throw new Error(t('imageStudioExtra.originalKeyUnavailable'))
  }
  let apiKey = credentialCache.get(apiKeyId)
  if (!apiKey) {
    const response = await meApi.getFullApiKey(apiKeyId)
    apiKey = response.key?.trim()
  }
  if (!apiKey) throw new Error(t('imageStudioExtra.readKeyFailed'))
  credentialCache.set(apiKeyId, apiKey)
  return { apiKeyId, apiKey, baseUrl: baseUrl.value }
}

function ensureSelectedModelAvailable(model: string) {
  if (!models.value.some(item => item.name === model)) {
    throw new Error(t('imageStudioExtra.modelUnavailable'))
  }
}

async function handleSubmit() {
  try {
    const submission = createImageSubmissionSnapshot(settings.value, form.value)
    if (!submission.form.prompt.trim()) throw new Error(t('imageStudioExtra.promptRequired'))
    if (!submission.model) throw new Error(t('imageStudioExtra.selectModel'))
    if (!baseUrl.value) throw new Error(t('imageStudioExtra.apiUrlNotLoaded'))
    ensureSelectedModelAvailable(submission.model)
    const credential = await getTaskCredential(submission.apiKeyId)

    const advancedParams = parseAdvancedParams(submission.form.advancedJson)
    const extraParams: Record<string, unknown> = {
      ...advancedParams,
      ...(submission.form.quality !== 'auto' ? { quality: submission.form.quality } : {}),
      ...(submission.form.background !== 'auto' ? { background: submission.form.background } : {}),
      ...(submission.form.outputFormat !== 'auto' ? { output_format: submission.form.outputFormat } : {}),
    }
    addTasks(submission.form, extraParams, credential, {
      model: submission.model,
      responseFormat: submission.responseFormat,
    })
    success(t('imageStudioExtra.tasksAdded', { count: Math.min(8, Math.max(1, submission.form.count)) }))
  } catch (error) {
    showError(error instanceof Error ? error.message : t('imageStudioExtra.createFailed'))
  }
}

async function handleRetry(id: string) {
  try {
    const task = tasks.value.find(item => item.id === id)
    if (!task) return
    const credential = await getTaskCredential(task.apiKeyId || settings.value.selectedKeyId)
    retryTask(id, credential)
  } catch (error) {
    showError(error instanceof Error ? error.message : t('imageStudioExtra.retryFailed'))
  }
}

function openPreview(task: ImageTask) {
  previewTask.value = task
  previewOpen.value = true
}

function handleDownload(task: ImageTask) {
  if (!task.imageUrl) return
  void downloadImage(task.imageUrl, `niffler-image-${task.id.slice(0, 8)}`, task.imageMimeType)
}

function requestTaskRemoval(id: string) {
  deleteMode.value = 'task'
  deleteTargetId.value = id
  deleteDialogOpen.value = true
}

function requestClearTasks() {
  deleteMode.value = 'all'
  deleteTargetId.value = ''
  deleteDialogOpen.value = true
}

function resetDeleteDialog() {
  if (deleting.value) return
  deleteDialogOpen.value = false
  deleteTargetId.value = ''
}

async function confirmDeletion() {
  deleting.value = true
  try {
    if (deleteMode.value === 'all') {
      await clearTasks()
      previewOpen.value = false
      previewTask.value = null
      warning(t('imageStudioExtra.cleared'))
    } else if (deleteTargetId.value) {
      const id = deleteTargetId.value
      removeTask(id)
      if (previewTask.value?.id === id) {
        previewOpen.value = false
        previewTask.value = null
      }
      success(t('imageStudioExtra.deleted'))
    }
    deleteDialogOpen.value = false
    deleteTargetId.value = ''
  } finally {
    deleting.value = false
  }
}

onMounted(loadResources)
</script>
