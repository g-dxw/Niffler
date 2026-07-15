function extensionFromMime(mimeType?: string) {
  if (mimeType?.includes('jpeg')) return 'jpg'
  if (mimeType?.includes('webp')) return 'webp'
  return 'png'
}

function clickDownload(url: string, filename: string) {
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  link.rel = 'noopener noreferrer'
  document.body.appendChild(link)
  link.click()
  link.remove()
}

export async function downloadImage(url: string, filename: string, mimeType?: string) {
  const extension = extensionFromMime(mimeType)
  if (url.startsWith('data:') || url.startsWith('blob:')) {
    clickDownload(url, `${filename}.${extension}`)
    return
  }

  try {
    const response = await fetch(url)
    if (!response.ok) throw new Error('download failed')
    const blob = await response.blob()
    const objectUrl = URL.createObjectURL(blob)
    clickDownload(objectUrl, `${filename}.${extensionFromMime(blob.type || mimeType)}`)
    URL.revokeObjectURL(objectUrl)
  } catch {
    clickDownload(url, `${filename}.${extension}`)
  }
}
