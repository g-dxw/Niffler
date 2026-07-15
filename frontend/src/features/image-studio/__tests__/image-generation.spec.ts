import { afterEach, describe, expect, it, vi } from 'vitest'
import { editImage, generateImage, ImageGenerationError, normalizeImagesBaseUrl } from '../api/image-generation'

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('image generation API', () => {
  it('normalizes the public base URL and sends a single-image generation request', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({
      data: [{ b64_json: 'aGVsbG8=', output_format: 'webp' }],
    }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)

    const result = await generateImage({
      apiKey: 'secret-key',
      baseUrl: 'https://niffler.example/',
      model: 'gpt-image-2',
      prompt: 'a cat',
      size: '1024x1024',
      responseFormat: 'b64_json',
      extraParams: { quality: 'high', _taskId: 'internal' },
    })

    expect(normalizeImagesBaseUrl('https://niffler.example/v1/')).toBe('https://niffler.example/v1')
    expect(fetchMock).toHaveBeenCalledTimes(1)
    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(url).toBe('https://niffler.example/v1/images/generations')
    expect(init.headers).toEqual({
      'Content-Type': 'application/json',
      Authorization: 'Bearer secret-key',
    })
    expect(JSON.parse(String(init.body))).toEqual({
      model: 'gpt-image-2',
      prompt: 'a cat',
      n: 1,
      size: '1024x1024',
      response_format: 'b64_json',
      quality: 'high',
      stream: true,
      partial_images: 1,
    })
    expect(result).toEqual({
      imageUrl: 'data:image/webp;base64,aGVsbG8=',
      b64Json: 'aGVsbG8=',
      mimeType: 'image/webp',
    })
  })

  it('lets the browser set the multipart boundary for edits', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({ data: [{ url: 'https://example.com/image.png' }] }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    const file = new File(['image'], 'reference.png', { type: 'image/png' })

    await editImage({
      apiKey: 'secret-key',
      baseUrl: 'https://niffler.example/v1',
      model: 'gpt-image-2',
      prompt: 'edit this',
      size: '976x992',
      responseFormat: 'url',
      images: [file],
    })

    const [, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(init.headers).toEqual({ Authorization: 'Bearer secret-key' })
    expect(init.body).toBeInstanceOf(FormData)
    const body = init.body as FormData
    expect(body.get('model')).toBe('gpt-image-2')
    expect(body.get('prompt')).toBe('edit this')
    expect(body.get('n')).toBe('1')
    expect(body.get('size')).toBe('976x992')
    expect(body.get('response_format')).toBe('url')
    expect(body.getAll('image')).toHaveLength(1)
    expect(body.get('output_format')).toBeNull()
    expect(body.get('quality')).toBeNull()
    expect(body.get('background')).toBeNull()
    expect(body.get('stream')).toBeNull()
    expect(body.get('partial_images')).toBeNull()
  })

  it('does not retry an ambiguous upstream 503', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({
      error: { message: 'upstream temporarily unavailable' },
    }), { status: 503 }))
    vi.stubGlobal('fetch', fetchMock)

    const promise = editImage({
      apiKey: 'secret-key',
      baseUrl: 'https://niffler.example/v1',
      model: 'gpt-image-2',
      prompt: 'use this as reference',
      responseFormat: 'b64_json',
      images: [new File(['image'], 'reference.png', { type: 'image/png' })],
    })

    await expect(promise).rejects.toThrow('upstream temporarily unavailable')
    expect(fetchMock).toHaveBeenCalledTimes(1)
  })

  it('parses the final image from Niffler image SSE events', async () => {
    const streamBody = [
      'event: image_generation.partial_image',
      'data: {"type":"image_generation.partial_image","b64_json":"cGFydGlhbA==","partial_image_index":0}',
      '',
      'event: image_generation.completed',
      'data: {"type":"image_generation.completed","b64_json":"ZmluYWw="}',
      '',
    ].join('\n')
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(streamBody, {
      status: 200,
      headers: { 'Content-Type': 'text/event-stream' },
    })))

    const result = await generateImage({
      apiKey: 'secret-key',
      baseUrl: 'https://niffler.example',
      model: 'gpt-image-2',
      prompt: 'a cat',
      responseFormat: 'b64_json',
      extraParams: { output_format: 'webp' },
    })

    expect(result).toEqual({
      imageUrl: 'data:image/webp;base64,ZmluYWw=',
      b64Json: 'ZmluYWw=',
      mimeType: 'image/webp',
    })
  })

  it('rejects a partial preview when the stream ends without a completed event', async () => {
    const streamBody = [
      'event: image_generation.partial_image',
      'data: {"type":"image_generation.partial_image","b64_json":"aW5jb21wbGV0ZQ==","partial_image_index":0}',
      '',
      'data: [DONE]',
      '',
    ].join('\n')
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(streamBody, {
      status: 200,
      headers: { 'Content-Type': 'text/event-stream' },
    })))

    const promise = generateImage({
      apiKey: 'secret-key',
      baseUrl: 'https://niffler.example',
      model: 'gpt-image-2',
      prompt: 'a cat',
      responseFormat: 'b64_json',
    })

    await expect(promise).rejects.toThrow('图片生成未完成，已丢弃不完整的预览图')
  })

  it('surfaces the upstream error message and status', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(JSON.stringify({
      error: { message: '余额不足' },
    }), { status: 402 })))

    const promise = generateImage({
      apiKey: 'secret-key',
      baseUrl: 'https://niffler.example',
      model: 'gpt-image-2',
      prompt: 'a cat',
      responseFormat: 'b64_json',
    })
    await expect(promise).rejects.toMatchObject<ImageGenerationError>({ message: '余额不足', status: 402 })
  })

  it.each([
    [401, '当前 API 密钥无效，或没有调用图片接口的权限'],
    [403, '当前 API 密钥无权调用所选图片模型或图片接口'],
  ])('explains image permission failures for HTTP %i', async (status, message) => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('', { status })))

    const promise = generateImage({
      apiKey: 'secret-key',
      baseUrl: 'https://niffler.example',
      model: 'gpt-image-2',
      prompt: 'a cat',
      responseFormat: 'b64_json',
    })

    await expect(promise).rejects.toMatchObject<ImageGenerationError>({ message, status })
  })
})
