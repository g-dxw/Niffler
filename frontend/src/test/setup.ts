function createMemoryStorage(): Storage {
  const values = new Map<string, string>()

  return {
    get length() {
      return values.size
    },
    clear() {
      values.clear()
    },
    getItem(key: string) {
      return values.has(key) ? values.get(key)! : null
    },
    key(index: number) {
      return Array.from(values.keys())[index] ?? null
    },
    removeItem(key: string) {
      values.delete(key)
    },
    setItem(key: string, value: string) {
      values.set(key, String(value))
    },
  }
}

function ensureStorage(name: 'localStorage' | 'sessionStorage') {
  const current = globalThis[name]
  if (
    current
    && typeof current.clear === 'function'
    && typeof current.getItem === 'function'
    && typeof current.setItem === 'function'
    && typeof current.removeItem === 'function'
  ) {
    return
  }

  Object.defineProperty(globalThis, name, {
    value: createMemoryStorage(),
    configurable: true,
    writable: true,
  })
}

ensureStorage('localStorage')
ensureStorage('sessionStorage')
