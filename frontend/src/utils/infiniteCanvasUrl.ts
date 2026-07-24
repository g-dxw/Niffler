export function getInfiniteCanvasUrl(
  path = '',
  baseUrl = import.meta.env.BASE_URL,
): string {
  const normalizedBaseUrl = `${baseUrl || '/'}`.replace(/\/*$/, '/')
  const normalizedPath = path.replace(/^\/+/, '')

  return `${normalizedBaseUrl}InfiniteCanvas/${normalizedPath}`
}
