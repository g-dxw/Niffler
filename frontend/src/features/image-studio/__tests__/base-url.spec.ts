import { describe, expect, it, vi } from 'vitest'
import { resolveImageApiBaseUrl } from '../utils/base-url'

describe('image API base URL resolution', () => {
  it('uses the browser origin in development without calling the support endpoint', async () => {
    const getPublicBaseUrl = vi.fn()
    await expect(resolveImageApiBaseUrl({
      isDev: true,
      origin: 'http://localhost:5173/',
      getPublicBaseUrl,
    })).resolves.toBe('http://localhost:5173')
    expect(getPublicBaseUrl).not.toHaveBeenCalled()
  })

  it('falls back to the browser origin when the production support endpoint rejects', async () => {
    const onFallback = vi.fn()
    await expect(resolveImageApiBaseUrl({
      isDev: false,
      origin: 'https://app.example.com/',
      getPublicBaseUrl: vi.fn().mockRejectedValue(new Error('401')),
      onFallback,
    })).resolves.toBe('https://app.example.com')
    expect(onFallback).toHaveBeenCalledTimes(1)
  })

  it('uses the configured public API address in production', async () => {
    await expect(resolveImageApiBaseUrl({
      isDev: false,
      origin: 'https://app.example.com',
      getPublicBaseUrl: vi.fn().mockResolvedValue({ public_base_url: 'https://api.example.com/' }),
    })).resolves.toBe('https://api.example.com')
  })
})
