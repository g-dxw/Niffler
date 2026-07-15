const RESERVED_PARAMS = new Set([
  'model',
  'prompt',
  'n',
  'image',
  'images',
  'mask',
  'response_format',
  'stream',
  'partial_images',
])

export function parseAdvancedParams(value: string): Record<string, unknown> {
  const trimmed = value.trim()
  if (!trimmed) return {}

  let parsed: unknown
  try {
    parsed = JSON.parse(trimmed)
  } catch {
    throw new Error('高级参数不是有效的 JSON')
  }

  if (!parsed || Array.isArray(parsed) || typeof parsed !== 'object') {
    throw new Error('高级参数必须是一个 JSON 对象')
  }

  const result: Record<string, unknown> = {}
  for (const [key, item] of Object.entries(parsed)) {
    if (key.startsWith('_')) continue
    if (RESERVED_PARAMS.has(key)) {
      throw new Error(`高级参数不能覆盖 ${key}`)
    }
    result[key] = item
  }
  return result
}

export function stripInternalParams(value: Record<string, unknown>): Record<string, unknown> {
  return Object.fromEntries(Object.entries(value).filter(([key]) => !key.startsWith('_')))
}
