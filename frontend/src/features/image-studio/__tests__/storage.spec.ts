import { beforeEach, describe, expect, it } from 'vitest'
import type { ImageTask } from '../types'
import { loadImageSettings, loadImageTasks, saveImageSettings, saveImageTasks } from '../utils/storage'

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
})
