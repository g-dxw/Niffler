import { i18n } from '@/i18n'

const t = i18n.global.t

export interface CachedImageRecord {
  id: string
  userId: string
  taskId: string
  blob: Blob
  mimeType: string
  size: number
  cachedAt: number
}

export type CachedImageMetadata = Omit<CachedImageRecord, 'blob' | 'mimeType'>

const DB_NAME = 'niffler-image-studio-cache'
const DB_VERSION = 1
const STORE_NAME = 'images'
export const MAX_IMAGE_CACHE_BYTES = 250 * 1024 * 1024

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
    request.onerror = () => reject(request.error || new Error(t('imageCacheErrors.open')))
  })
}

async function withStore<T>(mode: IDBTransactionMode, action: (store: IDBObjectStore) => IDBRequest<T>): Promise<T> {
  const db = await openDatabase()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, mode)
    const request = action(transaction.objectStore(STORE_NAME))
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error || new Error(t('imageCacheErrors.operation')))
    transaction.oncomplete = () => db.close()
    transaction.onerror = () => reject(transaction.error || new Error(t('imageCacheErrors.transaction')))
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
      if (!response.ok) throw new Error(t('imageCacheErrors.download'))
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
    request.onerror = () => reject(request.error || new Error(t('imageCacheErrors.clear')))
    transaction.oncomplete = () => resolve()
    transaction.onerror = () => reject(transaction.error || new Error(t('imageCacheErrors.clear')))
  })
  db.close()
}

export function selectImageCacheRecordsForDeletion(
  records: CachedImageMetadata[],
  currentUserId: string,
  retainedTaskIds: ReadonlySet<string>,
  maxBytes = MAX_IMAGE_CACHE_BYTES,
) {
  const orphaned = records.filter(record => (
    record.userId === currentUserId && !retainedTaskIds.has(record.taskId)
  ))
  const orphanedIds = new Set(orphaned.map(record => record.id))
  const retained = records
    .filter(record => !orphanedIds.has(record.id))
    .sort((a, b) => a.cachedAt - b.cachedAt)
  let retainedBytes = retained.reduce((total, record) => total + record.size, 0)
  const overCapacity: CachedImageMetadata[] = []
  for (const record of retained) {
    if (retainedBytes <= maxBytes) break
    overCapacity.push(record)
    retainedBytes -= record.size
  }
  return [...orphaned, ...overCapacity]
}

export async function pruneUserImages(
  userId: string,
  retainedTaskIds: ReadonlySet<string>,
  maxBytes = MAX_IMAGE_CACHE_BYTES,
) {
  const db = await openDatabase()
  return new Promise<string[]>((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, 'readwrite')
    const store = transaction.objectStore(STORE_NAME)
    const request = store.openCursor()
    const records: CachedImageMetadata[] = []
    let deletedTaskIds: string[] = []
    request.onsuccess = () => {
      const cursor = request.result
      if (cursor) {
        const record = cursor.value as CachedImageRecord
        records.push({
          id: record.id,
          userId: record.userId,
          taskId: record.taskId,
          size: record.size,
          cachedAt: record.cachedAt,
        })
        cursor.continue()
        return
      }
      const deletions = selectImageCacheRecordsForDeletion(records, userId, retainedTaskIds, maxBytes)
      deletedTaskIds = deletions
        .filter(record => record.userId === userId)
        .map(record => record.taskId)
      deletions.forEach(record => store.delete(record.id))
    }
    request.onerror = () => {
      db.close()
      reject(request.error || new Error(t('imageCacheErrors.read')))
    }
    transaction.oncomplete = () => {
      db.close()
      resolve(deletedTaskIds)
    }
    transaction.onerror = () => {
      db.close()
      reject(transaction.error || new Error(t('imageCacheErrors.clear')))
    }
    transaction.onabort = () => {
      db.close()
      reject(transaction.error || new Error(t('imageCacheErrors.aborted')))
    }
  })
}
