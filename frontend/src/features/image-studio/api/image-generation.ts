import type {
  EditImageParams,
  GenerateImageParams,
  GenerateImageResult,
} from '../types'
import { stripInternalParams } from '../utils/advanced-params'

export class ImageGenerationError extends Error {
  status?: number

  constructor(message: string, status?: number) {
    super(message)
    this.name = 'ImageGenerationError'
    this.status = status
  }
}

export function normalizeImagesBaseUrl(baseUrl: string) {
  const normalized = baseUrl.trim().replace(/\/+$/, '')
  if (!normalized) throw new Error('API 地址为空')
  return normalized.endsWith('/v1') ? normalized : `${normalized}/v1`
}

function buildEndpoint(baseUrl: string, path: string) {
  return `${normalizeImagesBaseUrl(baseUrl)}${path}`
}

function assertParams(params: GenerateImageParams) {
  if (!params.apiKey.trim()) throw new Error('请选择 API 密钥')
  if (!params.model.trim()) throw new Error('请选择图片模型')
  if (!params.prompt.trim()) throw new Error('请输入提示词')
}

function responseErrorMessage(status: number, fallback: string) {
  if (status === 401) {
    return `当前 API 密钥无效，或没有调用图片接口的权限${fallback ? `：${fallback}` : ''}`
  }
  if (status === 403) {
    return `当前 API 密钥无权调用所选图片模型或图片接口${fallback ? `：${fallback}` : ''}`
  }
  if (status === 524) {
    return '生图请求在边缘网关等待超时（524），请稍后重试或降低图片质量/尺寸'
  }
  return fallback || `请求失败 (${status})`
}

function parseSseImageResponse(text: string, outputFormat: string): unknown {
  let completedImage = ''
  let sawPartialImage = false

  for (const block of text.split(/\r?\n\r?\n/)) {
    const data = block
      .split(/\r?\n/)
      .filter(line => line.startsWith('data:'))
      .map(line => line.slice(5).trim())
      .join('\n')
    if (!data || data === '[DONE]') continue

    let event: {
      type?: string
      b64_json?: string
      error?: { message?: string } | string
      message?: string
    }
    try {
      event = JSON.parse(data) as typeof event
    } catch {
      continue
    }

    if (event.type?.endsWith('.failed') || event.type === 'error') {
      const errorMessage = typeof event.error === 'string' ? event.error : event.error?.message
      throw new ImageGenerationError(errorMessage || event.message || '图片生成失败')
    }
    if (typeof event.b64_json === 'string' && event.b64_json) {
      if (event.type?.endsWith('.completed')) {
        completedImage = event.b64_json
      } else if (event.type?.endsWith('.partial_image')) {
        sawPartialImage = true
      }
    }
  }

  if (!completedImage) {
    throw new ImageGenerationError(
      sawPartialImage
        ? '图片生成未完成，已丢弃不完整的预览图'
        : '图片流已结束，但没有返回完整图片',
    )
  }
  return { data: [{ b64_json: completedImage, output_format: outputFormat }] }
}

async function readResponse(response: Response, outputFormat: string): Promise<unknown> {
  const text = await response.text().catch(() => '')
  let parsed: unknown
  const isSse = response.headers.get('content-type')?.toLowerCase().includes('text/event-stream')
    || /^\s*(event|data):/m.test(text)

  if (!response.ok) {
    try {
      parsed = text ? JSON.parse(text) : undefined
    } catch {
      parsed = undefined
    }
    const errorData = parsed as { error?: { message?: string }, message?: string } | undefined
    throw new ImageGenerationError(
      responseErrorMessage(response.status, errorData?.error?.message || errorData?.message || text),
      response.status,
    )
  }

  if (isSse) return parseSseImageResponse(text, outputFormat)
  try {
    parsed = text ? JSON.parse(text) : undefined
  } catch {
    parsed = undefined
  }
  if (!parsed) throw new ImageGenerationError('接口返回了空响应或无效 JSON', response.status)
  return parsed
}

function toImageResult(value: unknown, fallbackOutputFormat = 'png'): GenerateImageResult {
  if (!value || typeof value !== 'object' || !('data' in value) || !Array.isArray(value.data)) {
    throw new ImageGenerationError('响应中没有图片数据')
  }
  const item = value.data[0] as { url?: unknown, b64_json?: unknown, output_format?: unknown } | undefined
  if (!item) throw new ImageGenerationError('响应中没有图片数据')

  const format = typeof item.output_format === 'string' ? item.output_format.toLowerCase() : fallbackOutputFormat
  const mimeType = format === 'jpg' || format === 'jpeg' ? 'image/jpeg' : format === 'webp' ? 'image/webp' : 'image/png'
  if (typeof item.url === 'string' && item.url) {
    return { imageUrl: item.url, mimeType }
  }
  if (typeof item.b64_json === 'string' && item.b64_json) {
    return {
      imageUrl: `data:${mimeType};base64,${item.b64_json}`,
      b64Json: item.b64_json,
      mimeType,
    }
  }
  throw new ImageGenerationError('图片响应格式不受支持')
}

export async function generateImage(params: GenerateImageParams): Promise<GenerateImageResult> {
  assertParams(params)
  const extraParams = stripInternalParams(params.extraParams || {})
  const outputFormat = typeof extraParams.output_format === 'string' ? extraParams.output_format : 'png'
  const body = {
    model: params.model.trim(),
    prompt: params.prompt.trim(),
    n: 1,
    ...(params.size?.trim() ? { size: params.size.trim() } : {}),
    response_format: params.responseFormat,
    ...extraParams,
    stream: true,
    partial_images: 1,
  }
  const response = await fetch(buildEndpoint(params.baseUrl, '/images/generations'), {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${params.apiKey}`,
    },
    body: JSON.stringify(body),
    signal: params.signal,
  })
  return toImageResult(await readResponse(response, outputFormat), outputFormat)
}

export async function editImage(params: EditImageParams): Promise<GenerateImageResult> {
  assertParams(params)
  if (!params.images.length) throw new Error('图生图至少需要一张参考图')

  const extraParams = stripInternalParams(params.extraParams || {})
  const outputFormat = typeof extraParams.output_format === 'string' ? extraParams.output_format : 'png'
  const form = buildEditForm(params, extraParams)
  const response = await fetch(buildEndpoint(params.baseUrl, '/images/edits'), {
    method: 'POST',
    headers: { Authorization: `Bearer ${params.apiKey}` },
    body: form,
    signal: params.signal,
  })
  return toImageResult(await readResponse(response, outputFormat), outputFormat)
}

function buildEditForm(
  params: EditImageParams,
  extraParams: Record<string, unknown>,
) {
  const form = new FormData()
  form.append('model', params.model.trim())
  form.append('prompt', params.prompt.trim())
  form.append('n', '1')
  if (params.size?.trim()) form.append('size', params.size.trim())
  form.append('response_format', params.responseFormat)
  params.images.forEach(file => form.append('image', file, file.name))
  if (params.mask) form.append('mask', params.mask, params.mask.name)

  for (const [key, value] of Object.entries(extraParams)) {
    if (value === undefined || value === null) continue
    form.append(key, typeof value === 'object' ? JSON.stringify(value) : String(value))
  }
  return form
}
