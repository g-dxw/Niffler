interface CachedImageRecord {
  id: string
  userId: string
  taskId: string
  blob: Blob
  mimeType: string
  size: number
  cachedAt: number
}

const DB_NAME = 'niffler-image-studio-cache'
const DB_VERSION = 1
const STORE_NAME = 'images'

function recordId(userId: string, taskId: string) {
  return `${userId}:${taskId}`
}

function openDatabase(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION)
    request.onupgradeneeded = () => {
      const db = request.result
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        const store = db.createObjectStore(STORE_NAME, { keyPath: 'id' })
        store.createIndex('userId', 'userId')
      }
    }
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error || new Error('无法打开图片缓存'))
  })
}

async function withStore<T>(mode: IDBTransactionMode, action: (store: IDBObjectStore) => IDBRequest<T>): Promise<T> {
  const db = await openDatabase()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, mode)
    const request = action(transaction.objectStore(STORE_NAME))
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error || new Error('图片缓存操作失败'))
    transaction.oncomplete = () => db.close()
    transaction.onerror = () => reject(transaction.error || new Error('图片缓存事务失败'))
  })
}

function dataUrlToBlob(dataUrl: string) {
  const commaIndex = dataUrl.indexOf(',')
  const header = dataUrl.slice(0, commaIndex)
  const mimeType = header.match(/^data:([^;,]+)/)?.[1] || 'image/png'
  const bytes = Uint8Array.from(atob(dataUrl.slice(commaIndex + 1)), char => char.charCodeAt(0))
  return new Blob([bytes], { type: mimeType })
}

export async function cacheTaskImage(userId: string, taskId: string, imageUrl: string, mimeType: string) {
  const blob = imageUrl.startsWith('data:')
    ? dataUrlToBlob(imageUrl)
    : await fetch(imageUrl).then(response => {
      if (!response.ok) throw new Error('无法下载图片用于缓存')
      return response.blob()
    })
  const record: CachedImageRecord = {
    id: recordId(userId, taskId),
    userId,
    taskId,
    blob,
    mimeType: blob.type || mimeType,
    size: blob.size,
    cachedAt: Date.now(),
  }
  await withStore('readwrite', store => store.put(record))
  return record
}

export function getTaskImage(userId: string, taskId: string) {
  return withStore<CachedImageRecord | undefined>('readonly', store => store.get(recordId(userId, taskId)))
}

export function deleteTaskImage(userId: string, taskId: string) {
  return withStore<undefined>('readwrite', store => store.delete(recordId(userId, taskId)))
}

export async function clearUserImages(userId: string) {
  const db = await openDatabase()
  await new Promise<void>((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, 'readwrite')
    const index = transaction.objectStore(STORE_NAME).index('userId')
    const request = index.openCursor(IDBKeyRange.only(userId))
    request.onsuccess = () => {
      const cursor = request.result
      if (!cursor) return
      cursor.delete()
      cursor.continue()
    }
    request.onerror = () => reject(request.error || new Error('清理图片缓存失败'))
    transaction.oncomplete = () => resolve()
    transaction.onerror = () => reject(transaction.error || new Error('清理图片缓存失败'))
  })
  db.close()
}
