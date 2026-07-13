import { computed, onBeforeUnmount, onMounted, ref, watch, type Ref } from 'vue'
import { editImage, generateImage } from '../api/image-generation'
import type {
  ImageGenerationForm,
  ImageStudioSettings,
  ImageTask,
  PendingImageInputs,
} from '../types'
import { cacheTaskImage, clearUserImages, deleteTaskImage, getTaskImage } from './image-cache'
import { loadImageTasks, saveImageTasks } from '../utils/storage'
import { buildCompatibleImageRequest } from '../utils/image-sizing'

interface UseImageTasksOptions {
  userId: string
  baseUrl: Ref<string>
  apiKey: Ref<string>
  settings: Ref<ImageStudioSettings>
}

function taskId() {
  return crypto.randomUUID?.() || `${Date.now()}-${Math.random().toString(36).slice(2)}`
}

function friendlyTaskError(error: unknown) {
  if (error instanceof DOMException && error.name === 'AbortError') return '任务已取消'
  if (error instanceof Error) return error.message
  return '生图请求失败'
}

export function useImageTasks(options: UseImageTasksOptions) {
  const tasks = ref<ImageTask[]>(loadImageTasks(options.userId))
  const controllers = new Map<string, AbortController>()
  const pendingInputs = new Map<string, PendingImageInputs>()
  const objectUrls = new Set<string>()
  let scheduling = false

  const runningCount = computed(() => tasks.value.filter(task => task.status === 'running').length)
  const pendingCount = computed(() => tasks.value.filter(task => task.status === 'pending').length)

  function replaceTask(id: string, change: Partial<ImageTask>) {
    const index = tasks.value.findIndex(task => task.id === id)
    if (index < 0) return
    tasks.value[index] = { ...tasks.value[index], ...change }
  }

  function revokeObjectUrl(url?: string) {
    if (!url?.startsWith('blob:')) return
    URL.revokeObjectURL(url)
    objectUrls.delete(url)
  }

  async function attachCachedImage(task: ImageTask) {
    if (task.status !== 'success' || !task.imageCached || task.imageUrl) return
    try {
      const cached = await getTaskImage(options.userId, task.id)
      if (!cached) return
      const url = URL.createObjectURL(cached.blob)
      objectUrls.add(url)
      replaceTask(task.id, {
        imageUrl: url,
        imageMimeType: cached.mimeType,
        imageSize: cached.size,
      })
    } catch {
      // IndexedDB may be unavailable in private browsing; history still works.
    }
  }

  async function runTask(task: ImageTask) {
    if (!options.apiKey.value.trim()) {
      replaceTask(task.id, { status: 'error', error: 'API 密钥未加载', finishedAt: Date.now() })
      return
    }

    const controller = new AbortController()
    controllers.set(task.id, controller)
    replaceTask(task.id, { status: 'running', startedAt: Date.now(), error: undefined })

    try {
      const inputs = pendingInputs.get(task.id)
      const baseParams = {
        apiKey: options.apiKey.value,
        baseUrl: options.baseUrl.value,
        model: task.model,
        prompt: task.prompt,
        size: task.size,
        responseFormat: task.responseFormat,
        extraParams: task.extraParams,
        signal: controller.signal,
      }
      const result = task.mode === 'edit'
        ? await editImage({ ...baseParams, images: inputs?.images || [], mask: inputs?.mask })
        : await generateImage(baseParams)

      replaceTask(task.id, {
        status: 'success',
        imageUrl: result.imageUrl,
        b64Json: result.b64Json,
        imageMimeType: result.mimeType,
        finishedAt: Date.now(),
      })

      try {
        const cached = await cacheTaskImage(options.userId, task.id, result.imageUrl, result.mimeType)
        const current = tasks.value.find(item => item.id === task.id)
        if (!current || current.status !== 'success') {
          await deleteTaskImage(options.userId, task.id).catch(() => undefined)
          return
        }
        const objectUrl = URL.createObjectURL(cached.blob)
        objectUrls.add(objectUrl)
        replaceTask(task.id, {
          imageUrl: objectUrl,
          b64Json: undefined,
          imageCached: true,
          imageMimeType: cached.mimeType,
          imageSize: cached.size,
        })
      } catch {
        // Keep the original response URL/data URL when local caching fails.
      }
    } catch (error) {
      const cancelled = controller.signal.aborted
      const currentTask = tasks.value.find(item => item.id === task.id)
      revokeObjectUrl(currentTask?.imageUrl)
      void deleteTaskImage(options.userId, task.id).catch(() => undefined)
      replaceTask(task.id, {
        status: cancelled ? 'cancelled' : 'error',
        error: friendlyTaskError(error),
        imageUrl: undefined,
        b64Json: undefined,
        imageCached: false,
        imageMimeType: undefined,
        imageSize: undefined,
        finishedAt: Date.now(),
      })
    } finally {
      controllers.delete(task.id)
      schedule()
    }
  }

  function schedule() {
    if (scheduling) return
    scheduling = true
    queueMicrotask(() => {
      scheduling = false
      const limit = Math.min(8, Math.max(1, Math.floor(options.settings.value.concurrency || 1)))
      let available = limit - runningCount.value
      if (available <= 0) return
      const next = tasks.value.filter(task => task.status === 'pending').slice(0, available)
      for (const task of next) {
        available -= 1
        void runTask(task)
      }
    })
  }

  function addTasks(form: ImageGenerationForm, extraParams: Record<string, unknown>) {
    const count = Math.min(8, Math.max(1, Math.floor(form.count || 1)))
    const now = Date.now()
    const isEdit = form.inputImages.length > 0
    const compatibleRequest = buildCompatibleImageRequest({
      model: options.settings.value.model,
      prompt: form.prompt,
      size: form.size,
      extraParams,
    })
    const newTasks = Array.from({ length: count }, (_, index): ImageTask => {
      const id = taskId()
      if (isEdit) {
        pendingInputs.set(id, { images: [...form.inputImages], mask: form.maskImage })
      }
      return {
        id,
        mode: isEdit ? 'edit' : 'generate',
        prompt: compatibleRequest.prompt,
        model: options.settings.value.model.trim(),
        size: compatibleRequest.size,
        responseFormat: options.settings.value.responseFormat,
        status: 'pending',
        createdAt: now + index,
        extraParams: compatibleRequest.extraParams,
        inputImageCount: isEdit ? form.inputImages.length : undefined,
        hasMask: isEdit && Boolean(form.maskImage),
      }
    })
    tasks.value.push(...newTasks)
    schedule()
  }

  function cancelTask(id: string) {
    const controller = controllers.get(id)
    if (controller) {
      controller.abort()
      return
    }
    const task = tasks.value.find(item => item.id === id)
    if (task?.status === 'pending') {
      replaceTask(id, { status: 'cancelled', error: '任务已取消', finishedAt: Date.now() })
    }
  }

  function retryTask(id: string) {
    const task = tasks.value.find(item => item.id === id)
    if (!task || task.status === 'running' || task.status === 'pending') return
    if (task.mode === 'edit' && !pendingInputs.has(id)) {
      replaceTask(id, { status: 'error', error: '参考图已释放，请重新提交图生图任务' })
      return
    }
    revokeObjectUrl(task.imageUrl)
    void deleteTaskImage(options.userId, id).catch(() => undefined)
    replaceTask(id, {
      status: 'pending',
      error: undefined,
      imageUrl: undefined,
      b64Json: undefined,
      imageCached: false,
      imageSize: undefined,
      startedAt: undefined,
      finishedAt: undefined,
      createdAt: Date.now(),
    })
    schedule()
  }

  function removeTask(id: string) {
    controllers.get(id)?.abort()
    controllers.delete(id)
    pendingInputs.delete(id)
    const task = tasks.value.find(item => item.id === id)
    revokeObjectUrl(task?.imageUrl)
    tasks.value = tasks.value.filter(item => item.id !== id)
    void deleteTaskImage(options.userId, id).catch(() => undefined)
  }

  async function clearTasks() {
    controllers.forEach(controller => controller.abort())
    controllers.clear()
    pendingInputs.clear()
    objectUrls.forEach(url => URL.revokeObjectURL(url))
    objectUrls.clear()
    tasks.value = []
    await clearUserImages(options.userId).catch(() => undefined)
  }

  watch(tasks, value => saveImageTasks(options.userId, value), { deep: true })
  watch(() => options.settings.value.concurrency, schedule)
  watch(() => options.apiKey.value, schedule)

  onMounted(() => {
    tasks.value.forEach(task => void attachCachedImage(task))
    schedule()
  })

  onBeforeUnmount(() => {
    controllers.forEach(controller => controller.abort())
    objectUrls.forEach(url => URL.revokeObjectURL(url))
  })

  return {
    tasks,
    runningCount,
    pendingCount,
    addTasks,
    cancelTask,
    retryTask,
    removeTask,
    clearTasks,
  }
}
