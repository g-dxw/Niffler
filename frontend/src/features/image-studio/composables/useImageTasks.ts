import { computed, onBeforeUnmount, onMounted, ref, watch, type Ref } from 'vue'
import { i18n } from '@/i18n'
import { editImage, generateImage } from '../api/image-generation'
import type {
  ImageGenerationForm,
  ImageStudioSettings,
  ImageTask,
  ImageTaskConfiguration,
  ImageTaskCredential,
  PendingImageInputs,
} from '../types'
import {
  cacheTaskImage,
  clearUserImages,
  deleteTaskImage,
  getTaskImage,
  pruneUserImages,
} from './image-cache'
import { loadImageTasks, saveImageTasks, selectRetainedImageTaskIds } from '../utils/storage'
import { buildCompatibleImageRequest } from '../utils/image-sizing'

const t = i18n.global.t

interface UseImageTasksOptions {
  userId: string
  settings: Ref<ImageStudioSettings>
}

function taskId() {
  return crypto.randomUUID?.() || `${Date.now()}-${Math.random().toString(36).slice(2)}`
}

function friendlyTaskError(error: unknown) {
  if (error instanceof DOMException && error.name === 'AbortError') return t('imageTaskErrors.cancelled')
  if (error instanceof Error) return error.message
  return t('imageTaskErrors.requestFailed')
}

export function useImageTasks(options: UseImageTasksOptions) {
  const tasks = ref<ImageTask[]>(loadImageTasks(options.userId))
  const controllers = new Map<string, AbortController>()
  const pendingInputs = new Map<string, PendingImageInputs>()
  const taskCredentials = new Map<string, ImageTaskCredential>()
  const objectUrls = new Set<string>()
  let scheduling = false
  let cachePruneRunning = false
  let cachePruneRequested = false
  let disposed = false
  let lastCreatedAt = tasks.value.reduce((latest, task) => Math.max(latest, task.createdAt), 0)

  const runningCount = computed(() => tasks.value.filter(task => task.status === 'running').length)
  const pendingCount = computed(() => tasks.value.filter(task => task.status === 'pending').length)

  function reserveCreatedAtSequence(count = 1) {
    const first = Math.max(Date.now(), lastCreatedAt + 1)
    lastCreatedAt = first + count - 1
    return first
  }

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
      if (!cached) {
        replaceTask(task.id, { imageCached: false, imageSize: undefined })
        return
      }
      if (!tasks.value.some(item => item.id === task.id)) return
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
    const credential = taskCredentials.get(task.id)
    if (!credential?.apiKey.trim()) {
      pendingInputs.delete(task.id)
      taskCredentials.delete(task.id)
      replaceTask(task.id, { status: 'error', error: t('imageTaskErrors.keyNotLoaded'), finishedAt: Date.now() })
      schedule()
      return
    }

    const controller = new AbortController()
    controllers.set(task.id, controller)
    replaceTask(task.id, { status: 'running', startedAt: Date.now(), error: undefined })

    try {
      const inputs = pendingInputs.get(task.id)
      const baseParams = {
        apiKey: credential.apiKey,
        baseUrl: credential.baseUrl,
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
      pendingInputs.delete(task.id)
      taskCredentials.delete(task.id)
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

  function addTasks(
    form: ImageGenerationForm,
    extraParams: Record<string, unknown>,
    credential: ImageTaskCredential,
    configuration: ImageTaskConfiguration,
  ) {
    const count = Math.min(8, Math.max(1, Math.floor(form.count || 1)))
    const now = reserveCreatedAtSequence(count)
    const isEdit = form.inputImages.length > 0
    const compatibleRequest = buildCompatibleImageRequest({
      model: configuration.model,
      prompt: form.prompt,
      size: form.size,
      extraParams,
    })
    const newTasks = Array.from({ length: count }, (_, index): ImageTask => {
      const id = taskId()
      taskCredentials.set(id, { ...credential })
      if (isEdit) {
        pendingInputs.set(id, { images: [...form.inputImages], mask: form.maskImage })
      }
      return {
        id,
        apiKeyId: credential.apiKeyId,
        mode: isEdit ? 'edit' : 'generate',
        prompt: compatibleRequest.prompt,
        model: configuration.model.trim(),
        size: compatibleRequest.size,
        responseFormat: configuration.responseFormat,
        status: 'pending',
        createdAt: now + index,
        extraParams: compatibleRequest.extraParams,
        inputImageCount: isEdit ? form.inputImages.length : undefined,
        hasMask: isEdit && Boolean(form.maskImage),
      }
    })
    tasks.value.push(...newTasks)
    trimTaskHistory()
    schedule()
  }

  function releaseTaskResources(task: ImageTask, deleteImage = true) {
    controllers.get(task.id)?.abort()
    controllers.delete(task.id)
    pendingInputs.delete(task.id)
    taskCredentials.delete(task.id)
    revokeObjectUrl(task.imageUrl)
    if (deleteImage) void deleteTaskImage(options.userId, task.id).catch(() => undefined)
  }

  function trimTaskHistory() {
    const retainedIds = selectRetainedImageTaskIds(tasks.value)
    if (retainedIds.size === tasks.value.length) return
    const removed = tasks.value.filter(task => !retainedIds.has(task.id))
    removed.forEach(task => releaseTaskResources(task))
    tasks.value = tasks.value.filter(task => retainedIds.has(task.id))
  }

  function cancelTask(id: string) {
    const controller = controllers.get(id)
    if (controller) {
      controller.abort()
      return
    }
    const task = tasks.value.find(item => item.id === id)
    if (task?.status === 'pending') {
      pendingInputs.delete(id)
      taskCredentials.delete(id)
      replaceTask(id, { status: 'cancelled', error: t('imageTaskErrors.cancelled'), finishedAt: Date.now() })
    }
  }

  function retryTask(id: string, credential?: ImageTaskCredential) {
    const task = tasks.value.find(item => item.id === id)
    if (!task || task.status === 'running' || task.status === 'pending') return
    if (task.mode === 'edit' && !pendingInputs.has(id)) {
      replaceTask(id, { status: 'error', error: t('imageTaskErrors.referenceReleased') })
      return
    }
    if (credential) taskCredentials.set(id, { ...credential })
    if (!taskCredentials.has(id)) {
      replaceTask(id, { status: 'error', error: t('imageTaskErrors.originalKeyNotLoaded') })
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
      createdAt: reserveCreatedAtSequence(),
    })
    schedule()
  }

  function removeTask(id: string) {
    const task = tasks.value.find(item => item.id === id)
    if (task) releaseTaskResources(task)
    tasks.value = tasks.value.filter(item => item.id !== id)
  }

  async function clearTasks() {
    controllers.forEach(controller => controller.abort())
    controllers.clear()
    pendingInputs.clear()
    taskCredentials.clear()
    objectUrls.forEach(url => URL.revokeObjectURL(url))
    objectUrls.clear()
    tasks.value = []
    await clearUserImages(options.userId).catch(() => undefined)
  }

  function persistTasksAndPruneCache(value: ImageTask[]) {
    saveImageTasks(options.userId, value)
    cachePruneRequested = true
    if (cachePruneRunning) return
    cachePruneRunning = true
    queueMicrotask(async () => {
      try {
        while (cachePruneRequested && !disposed) {
          cachePruneRequested = false
          const retainedTaskIds = saveImageTasks(options.userId, tasks.value)
          const deletedTaskIds = await pruneUserImages(options.userId, retainedTaskIds)
          if (disposed) return
          const deleted = new Set(deletedTaskIds)
          for (const task of tasks.value) {
            if (deleted.has(task.id) && task.imageCached) {
              revokeObjectUrl(task.imageUrl)
              replaceTask(task.id, { imageUrl: undefined, imageCached: false, imageSize: undefined })
            }
          }
        }
      } catch {
        // IndexedDB may be unavailable; task history still works in localStorage.
      } finally {
        cachePruneRunning = false
        if (cachePruneRequested && !disposed) persistTasksAndPruneCache(tasks.value)
      }
    })
  }

  watch(tasks, persistTasksAndPruneCache, { deep: true })
  watch(() => options.settings.value.concurrency, schedule)

  onMounted(() => {
    trimTaskHistory()
    tasks.value.forEach(task => void attachCachedImage(task))
    persistTasksAndPruneCache(tasks.value)
    schedule()
  })

  onBeforeUnmount(() => {
    disposed = true
    controllers.forEach(controller => controller.abort())
    pendingInputs.clear()
    taskCredentials.clear()
    objectUrls.forEach(url => URL.revokeObjectURL(url))
    objectUrls.clear()
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
