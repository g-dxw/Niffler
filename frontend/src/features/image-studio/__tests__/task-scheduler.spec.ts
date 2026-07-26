import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, nextTick, ref, type App } from '@/test/vue'
import { useImageTasks } from '../composables/useImageTasks'
import type { ImageGenerationForm, ImageStudioSettings } from '../types'
import { IMAGE_TASK_LIMIT } from '../utils/storage'

const apiMocks = vi.hoisted(() => ({
  generateImage: vi.fn(),
  editImage: vi.fn(),
}))

const cacheMocks = vi.hoisted(() => ({
  cacheTaskImage: vi.fn(),
  clearUserImages: vi.fn(),
  deleteTaskImage: vi.fn(),
  getTaskImage: vi.fn(),
  pruneUserImages: vi.fn(),
}))

vi.mock('../api/image-generation', () => apiMocks)
vi.mock('../composables/image-cache', () => cacheMocks)

interface Deferred<T> {
  promise: Promise<T>
  resolve: (value: T) => void
  reject: (reason?: unknown) => void
}

interface GeneratedImageResult {
  imageUrl: string
  mimeType: string
}

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void
  let reject!: (reason?: unknown) => void
  const promise = new Promise<T>((onResolve, onReject) => {
    resolve = onResolve
    reject = onReject
  })
  return { promise, resolve, reject }
}

const form: ImageGenerationForm = {
  prompt: 'cat',
  count: 1,
  size: '1024x1024',
  quality: 'auto',
  background: 'auto',
  outputFormat: 'auto',
  advancedJson: '',
  inputImages: [],
  maskImage: null,
}

const mountedApps: App[] = []

function mountScheduler(concurrency = 1) {
  const settings = ref<ImageStudioSettings>({
    selectedKeyId: 'key-a',
    model: 'gpt-image-2',
    responseFormat: 'url',
    concurrency,
  })
  let scheduler!: ReturnType<typeof useImageTasks>
  const app = createApp(defineComponent({
    setup() {
      scheduler = useImageTasks({ userId: 'scheduler-user', settings })
      return () => h('div')
    },
  }))
  const root = document.createElement('div')
  document.body.appendChild(root)
  app.mount(root)
  mountedApps.push(app)
  return { scheduler, settings, root }
}

async function settle() {
  for (let index = 0; index < 10; index += 1) {
    await Promise.resolve()
    await nextTick()
  }
}

beforeEach(() => {
  localStorage.clear()
  apiMocks.generateImage.mockReset()
  apiMocks.editImage.mockReset()
  cacheMocks.cacheTaskImage.mockReset().mockResolvedValue({
    blob: new Blob(['image'], { type: 'image/png' }),
    mimeType: 'image/png',
    size: 5,
  })
  cacheMocks.clearUserImages.mockReset().mockResolvedValue(undefined)
  cacheMocks.deleteTaskImage.mockReset().mockResolvedValue(undefined)
  cacheMocks.getTaskImage.mockReset().mockResolvedValue(undefined)
  cacheMocks.pruneUserImages.mockReset().mockResolvedValue([])
  vi.stubGlobal('URL', {
    ...URL,
    createObjectURL: vi.fn(() => 'blob:cached-image'),
    revokeObjectURL: vi.fn(),
  })
})

afterEach(() => {
  mountedApps.splice(0).forEach(app => app.unmount())
  document.body.innerHTML = ''
  vi.unstubAllGlobals()
})

describe('image task scheduler', () => {
  it('binds queued tasks to the API key selected when they were created', async () => {
    const first = deferred<{ imageUrl: string, mimeType: string }>()
    const second = deferred<{ imageUrl: string, mimeType: string }>()
    apiMocks.generateImage.mockImplementationOnce(() => first.promise).mockImplementationOnce(() => second.promise)
    const { scheduler, settings } = mountScheduler(1)

    scheduler.addTasks({ ...form, count: 2 }, {}, {
      apiKeyId: 'key-a',
      apiKey: 'secret-a',
      baseUrl: 'https://a.example',
    }, { model: 'gpt-image-2', responseFormat: 'url' })
    await settle()
    expect(apiMocks.generateImage).toHaveBeenCalledTimes(1)

    settings.value.selectedKeyId = 'key-b'
    first.resolve({ imageUrl: 'data:image/png;base64,YQ==', mimeType: 'image/png' })
    await settle()

    expect(apiMocks.generateImage).toHaveBeenCalledTimes(2)
    expect(apiMocks.generateImage.mock.calls[1][0]).toMatchObject({
      apiKey: 'secret-a',
      baseUrl: 'https://a.example',
    })
    expect(scheduler.tasks.value[1].apiKeyId).toBe('key-a')
    second.resolve({ imageUrl: 'data:image/png;base64,Yg==', mimeType: 'image/png' })
    await settle()
  })

  it('honors concurrency and starts the next queued task after a slot is released', async () => {
    const requests = [
      deferred<GeneratedImageResult>(),
      deferred<GeneratedImageResult>(),
      deferred<GeneratedImageResult>(),
    ]
    requests.forEach(request => apiMocks.generateImage.mockImplementationOnce(() => request.promise))
    const { scheduler } = mountScheduler(2)

    scheduler.addTasks({ ...form, count: 3 }, {}, {
      apiKeyId: 'key-a', apiKey: 'secret-a', baseUrl: 'https://a.example',
    }, { model: 'gpt-image-2', responseFormat: 'url' })
    await settle()
    expect(apiMocks.generateImage).toHaveBeenCalledTimes(2)
    expect(scheduler.runningCount.value).toBe(2)

    requests[0].resolve({ imageUrl: 'data:image/png;base64,YQ==', mimeType: 'image/png' })
    await settle()
    expect(apiMocks.generateImage).toHaveBeenCalledTimes(3)
    requests[1].resolve({ imageUrl: 'data:image/png;base64,Yg==', mimeType: 'image/png' })
    requests[2].resolve({ imageUrl: 'data:image/png;base64,Yw==', mimeType: 'image/png' })
    await settle()
  })

  it('cancels a running request and does not execute a cancelled queued task', async () => {
    apiMocks.generateImage.mockImplementation(({ signal }: { signal: AbortSignal }) => new Promise((_, reject) => {
      signal.addEventListener('abort', () => reject(new DOMException('cancelled', 'AbortError')))
    }))
    const { scheduler } = mountScheduler(1)
    scheduler.addTasks({ ...form, count: 2 }, {}, {
      apiKeyId: 'key-a', apiKey: 'secret-a', baseUrl: 'https://a.example',
    }, { model: 'gpt-image-2', responseFormat: 'url' })
    await settle()

    const [running, queued] = scheduler.tasks.value
    scheduler.cancelTask(queued.id)
    scheduler.cancelTask(running.id)
    await settle()

    expect(apiMocks.generateImage).toHaveBeenCalledTimes(1)
    expect(scheduler.tasks.value.map(task => task.status)).toEqual(['cancelled', 'cancelled'])
  })

  it('keeps only the newest task records in session memory', async () => {
    apiMocks.generateImage.mockImplementation(({ signal }: { signal: AbortSignal }) => new Promise((_, reject) => {
      signal.addEventListener('abort', () => reject(new DOMException('cancelled', 'AbortError')))
    }))
    const { scheduler } = mountScheduler(1)
    scheduler.addTasks({ ...form, count: 8 }, {}, {
      apiKeyId: 'key-a', apiKey: 'secret-a', baseUrl: 'https://a.example',
    }, { model: 'gpt-image-2', responseFormat: 'url' })
    const oldestTaskId = scheduler.tasks.value[0].id

    for (let index = 1; index < 38; index += 1) {
      scheduler.addTasks({ ...form, count: 8 }, {}, {
        apiKeyId: 'key-a', apiKey: 'secret-a', baseUrl: 'https://a.example',
      }, { model: 'gpt-image-2', responseFormat: 'url' })
    }

    expect(scheduler.tasks.value).toHaveLength(IMAGE_TASK_LIMIT)
    expect(scheduler.tasks.value.some(task => task.id === oldestTaskId)).toBe(false)
    const createdTimes = scheduler.tasks.value.map(task => task.createdAt)
    expect(new Set(createdTimes).size).toBe(IMAGE_TASK_LIMIT)
    await settle()
  })

  it('coalesces cache cleanup requests while a scan is already running', async () => {
    const firstPrune = deferred<string[]>()
    cacheMocks.pruneUserImages.mockReset()
      .mockReturnValueOnce(firstPrune.promise)
      .mockResolvedValue([])
    apiMocks.generateImage.mockImplementation(({ signal }: { signal: AbortSignal }) => new Promise((_, reject) => {
      signal.addEventListener('abort', () => reject(new DOMException('cancelled', 'AbortError')))
    }))
    const { scheduler } = mountScheduler(1)
    await settle()
    expect(cacheMocks.pruneUserImages).toHaveBeenCalledTimes(1)

    scheduler.addTasks({ ...form, count: 3 }, {}, {
      apiKeyId: 'key-a', apiKey: 'secret-a', baseUrl: 'https://a.example',
    }, { model: 'gpt-image-2', responseFormat: 'url' })
    await settle()
    expect(cacheMocks.pruneUserImages).toHaveBeenCalledTimes(1)

    firstPrune.resolve([])
    await settle()
    expect(cacheMocks.pruneUserImages).toHaveBeenCalledTimes(2)
  })
})
