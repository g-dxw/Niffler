import { beforeEach, describe, expect, it } from 'vitest'
import type { ImageTask } from '../types'
import { IMAGE_TASK_LIMIT, loadImageSettings, loadImageTasks, saveImageSettings, saveImageTasks } from '../utils/storage'
import { selectImageCacheRecordsForDeletion, type CachedImageRecord } from '../composables/image-cache'

beforeEach(() => localStorage.clear())

describe('image studio storage', () => {
  it('uses the source workbench response default for a new user', () => {
    expect(loadImageSettings('new-user').responseFormat).toBe('url')
  })

  it('isolates settings by user and never includes an API credential field', () => {
    saveImageSettings('user-a', { selectedKeyId: 'key-a', model: 'image-a', responseFormat: 'b64_json', concurrency: 3 })
    saveImageSettings('user-b', { selectedKeyId: 'key-b', model: 'image-b', responseFormat: 'url', concurrency: 2 })

    expect(loadImageSettings('user-a').selectedKeyId).toBe('key-a')
    expect(loadImageSettings('user-b').selectedKeyId).toBe('key-b')
    expect(JSON.stringify(localStorage)).not.toContain('secret-key')
  })

  it('marks unfinished persisted tasks as interrupted and removes embedded image data', () => {
    const task: ImageTask = {
      id: 'task-1',
      apiKeyId: 'key-a',
      mode: 'generate',
      prompt: 'cat',
      model: 'gpt-image-2',
      size: '1024x1024',
      responseFormat: 'b64_json',
      status: 'running',
      createdAt: 1,
      imageUrl: 'data:image/png;base64,aGVsbG8=',
      b64Json: 'aGVsbG8=',
    }
    saveImageTasks('user-a', [task])

    const raw = localStorage.getItem('niffler:image-studio:user-a:tasks:v1') || ''
    expect(raw).not.toContain('aGVsbG8=')
    expect(loadImageTasks('user-a')[0]).toMatchObject({
      id: 'task-1',
      status: 'cancelled',
      error: '页面刷新，任务已中断',
    })
    expect(loadImageTasks('user-b')).toEqual([])
  })

  it('returns exactly the retained task ids when history exceeds the record limit', () => {
    const tasks = Array.from({ length: IMAGE_TASK_LIMIT + 2 }, (_, index): ImageTask => ({
      id: `task-${index}`,
      apiKeyId: 'key-a',
      mode: 'generate',
      prompt: 'cat',
      model: 'gpt-image-2',
      size: '1024x1024',
      responseFormat: 'url',
      status: 'success',
      createdAt: index,
    }))

    const retained = saveImageTasks('user-a', tasks)
    expect(retained.size).toBe(IMAGE_TASK_LIMIT)
    expect(retained.has('task-0')).toBe(false)
    expect(retained.has('task-1')).toBe(false)
    expect(retained.has('task-2')).toBe(true)
    expect(loadImageTasks('user-a')).toHaveLength(IMAGE_TASK_LIMIT)
  })

  it('limits oversized persisted history while loading', () => {
    const tasks = Array.from({ length: IMAGE_TASK_LIMIT + 2 }, (_, index): ImageTask => ({
      id: `task-${index}`,
      apiKeyId: 'key-a',
      mode: 'generate',
      prompt: 'cat',
      model: 'gpt-image-2',
      size: '1024x1024',
      responseFormat: 'url',
      status: 'success',
      createdAt: index,
    }))
    localStorage.setItem('niffler:image-studio:user-a:tasks:v1', JSON.stringify(tasks))

    const loaded = loadImageTasks('user-a')
    expect(loaded).toHaveLength(IMAGE_TASK_LIMIT)
    expect(loaded.some(task => task.id === 'task-0')).toBe(false)
    expect(loaded.some(task => task.id === 'task-301')).toBe(true)
  })

  it('retains a retried old task according to its refreshed creation time', () => {
    const tasks = Array.from({ length: IMAGE_TASK_LIMIT + 2 }, (_, index): ImageTask => ({
      id: `task-${index}`,
      apiKeyId: 'key-a',
      mode: 'generate',
      prompt: 'cat',
      model: 'gpt-image-2',
      size: '1024x1024',
      responseFormat: 'url',
      status: 'success',
      createdAt: index,
    }))
    tasks[0].createdAt = IMAGE_TASK_LIMIT + 10

    const retained = saveImageTasks('user-a', tasks)

    expect(retained.has('task-0')).toBe(true)
    expect(retained.has('task-1')).toBe(false)
    expect(retained.has('task-2')).toBe(false)
  })

  it('evicts orphaned images first and then the oldest retained images over capacity', () => {
    const record = (taskId: string, size: number, cachedAt: number): CachedImageRecord => ({
      id: `user-a:${taskId}`,
      userId: 'user-a',
      taskId,
      blob: new Blob([new Uint8Array(size)]),
      mimeType: 'image/png',
      size,
      cachedAt,
    })
    const records = [
      record('orphan', 2, 1),
      record('old-retained', 6, 2),
      record('new-retained', 6, 3),
    ]

    const deleted = selectImageCacheRecordsForDeletion(
      records,
      'user-a',
      new Set(['old-retained', 'new-retained']),
      8,
    )
    expect(deleted.map(item => item.taskId)).toEqual(['orphan', 'old-retained'])
  })

  it('enforces the byte limit across users without treating other users as orphaned', () => {
    const metadata = [
      { id: 'user-b:old', userId: 'user-b', taskId: 'old', size: 6, cachedAt: 1 },
      { id: 'user-a:current', userId: 'user-a', taskId: 'current', size: 6, cachedAt: 2 },
    ]

    const deleted = selectImageCacheRecordsForDeletion(
      metadata,
      'user-a',
      new Set(['current']),
      8,
    )

    expect(deleted.map(item => item.id)).toEqual(['user-b:old'])
  })
})
