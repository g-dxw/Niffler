interface ResolveImageApiBaseUrlOptions {
  isDev: boolean
  origin: string
  getPublicBaseUrl: () => Promise<{ public_base_url: string }>
  onFallback?: (error: unknown) => void
}

export async function resolveImageApiBaseUrl(options: ResolveImageApiBaseUrlOptions) {
  const fallback = options.origin.replace(/\/+$/, '')
  if (options.isDev) return fallback

  try {
    const response = await options.getPublicBaseUrl()
    return response.public_base_url?.trim().replace(/\/+$/, '') || fallback
  } catch (error) {
    options.onFallback?.(error)
    return fallback
  }
}
