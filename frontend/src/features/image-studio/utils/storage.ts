import { DEFAULT_IMAGE_STUDIO_SETTINGS } from '../constants'
import type { ImageStudioSettings, ImageTask, ImageTaskStatus } from '../types'
import { i18n } from '@/i18n'

const t = i18n.global.t

const VALID_STATUSES = new Set<ImageTaskStatus>(['pending', 'running', 'success', 'error', 'cancelled'])
export const IMAGE_TASK_LIMIT = 300

function settingsKey(userId: string) {
  return `niffler:image-studio:${userId}:settings:v1`
}

function tasksKey(userId: string) {
  return `niffler:image-studio:${userId}:tasks:v1`
}

export function loadImageSettings(userId: string): ImageStudioSettings {
  try {
    const value = JSON.parse(localStorage.getItem(settingsKey(userId)) || '{}') as Partial<ImageStudioSettings>
    return {
      selectedKeyId: typeof value.selectedKeyId === 'string' ? value.selectedKeyId : '',
      model: typeof value.model === 'string' ? value.model : '',
      responseFormat: value.responseFormat === 'url' || value.responseFormat === 'b64_json'
        ? value.responseFormat
        : DEFAULT_IMAGE_STUDIO_SETTINGS.responseFormat,
      concurrency: typeof value.concurrency === 'number'
        ? Math.min(8, Math.max(1, Math.floor(value.concurrency)))
        : DEFAULT_IMAGE_STUDIO_SETTINGS.concurrency,
    }
  } catch {
    return { ...DEFAULT_IMAGE_STUDIO_SETTINGS }
  }
}

export function saveImageSettings(userId: string, settings: ImageStudioSettings) {
  const safeSettings = { ...settings, concurrency: Math.min(8, Math.max(1, settings.concurrency)) }
  try {
    localStorage.setItem(settingsKey(userId), JSON.stringify(safeSettings))
  } catch {
    // Storage can be unavailable in privacy mode; the page still works in memory.
  }
}

export function loadImageTasks(userId: string): ImageTask[] {
  try {
    const values = JSON.parse(localStorage.getItem(tasksKey(userId)) || '[]') as Partial<ImageTask>[]
    if (!Array.isArray(values)) return []
    const tasks = values.flatMap((value): ImageTask[] => {
      if (
        typeof value.id !== 'string'
        || typeof value.prompt !== 'string'
        || typeof value.model !== 'string'
        || typeof value.size !== 'string'
        || typeof value.createdAt !== 'number'
        || !value.status
        || !VALID_STATUSES.has(value.status)
      ) return []

      const task = {
        ...value,
        apiKeyId: typeof value.apiKeyId === 'string' ? value.apiKeyId : '',
      } as ImageTask
      if (task.status === 'pending' || task.status === 'running') {
        return [{ ...task, status: 'cancelled', error: t('imageTaskErrors.pageReloaded'), finishedAt: Date.now() }]
      }
      return [task]
    })
    const retainedIds = selectRetainedImageTaskIds(tasks)
    return tasks.filter(task => retainedIds.has(task.id))
  } catch {
    return []
  }
}

export function selectRetainedImageTaskIds(tasks: ImageTask[]) {
  return new Set(
    [...tasks]
      .sort((left, right) => right.createdAt - left.createdAt)
      .slice(0, IMAGE_TASK_LIMIT)
      .map(task => task.id),
  )
}

export function saveImageTasks(userId: string, tasks: ImageTask[]) {
  const retainedIds = selectRetainedImageTaskIds(tasks)
  const retainedTasks = tasks.filter(task => retainedIds.has(task.id))
  const serializable = retainedTasks.map(task => ({
    ...task,
    imageUrl: task.imageUrl?.startsWith('blob:') || task.imageUrl?.startsWith('data:')
      ? undefined
      : task.imageUrl,
    b64Json: undefined,
  }))
  try {
    localStorage.setItem(tasksKey(userId), JSON.stringify(serializable))
  } catch {
    // Ignore quota/privacy failures. Image blobs are managed separately in IndexedDB.
  }
  return retainedIds
}
