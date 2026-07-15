interface ImageSize {
  width: number
  height: number
}

interface CompatibleImageRequest {
  model: string
  prompt: string
  size: string
  extraParams?: Record<string, unknown>
}

const MIN_SIZE = 256
const GPT_IMAGE_2_MAX_SIZE = 3840
const GPT_IMAGE_2_STEP = 16
const DALLE_2_ALLOWED = ['256x256', '512x512', '1024x1024'] as const
const DALLE_3_ALLOWED = ['1024x1024', '1792x1024', '1024x1792'] as const
const GPT_IMAGE_1_ALLOWED = ['1024x1024', '1536x1024', '1024x1536'] as const
const COMMON_ASPECT_RATIOS = ['1:1', '5:4', '4:5', '4:3', '3:4', '3:2', '2:3', '16:9', '9:16', '21:9'] as const

const GEMINI_25_1K_SIZES: Record<string, string> = {
  '1:1': '1024x1024',
  '5:4': '1152x928',
  '4:5': '928x1152',
  '4:3': '1184x864',
  '3:4': '864x1184',
  '3:2': '1248x832',
  '2:3': '832x1248',
  '16:9': '1344x768',
  '9:16': '768x1344',
  '21:9': '1536x672',
}

const GEMINI_3_1K_SIZES: Record<string, string> = {
  '1:1': '1024x1024',
  '5:4': '1152x928',
  '4:5': '928x1152',
  '4:3': '1200x896',
  '3:4': '896x1200',
  '3:2': '1264x848',
  '2:3': '848x1264',
  '16:9': '1376x768',
  '9:16': '768x1376',
  '21:9': '1584x672',
  '4:1': '2048x512',
  '1:4': '512x2048',
  '8:1': '3072x384',
  '1:8': '384x3072',
}

function modelNeedle(model: string) {
  return model.trim().toLowerCase().replace(/[\s_]+/g, '-').replace(/-+/g, '-')
}

function includesAll(value: string, tokens: readonly string[]) {
  return tokens.every(token => value.includes(token))
}

function isDalle2(model: string) {
  const value = modelNeedle(model)
  return value.includes('dall-e-2') || value.includes('dalle-2')
}

function isDalle3(model: string) {
  const value = modelNeedle(model)
  return value.includes('dall-e-3') || value.includes('dalle-3')
}

function isGptImage2(model: string) {
  const value = modelNeedle(model)
  return value.includes('gpt-image-2') || includesAll(value, ['gpt', 'image', '2'])
}

function isGptImage1(model: string) {
  const value = modelNeedle(model)
  if (value.includes('gpt-image-1') || includesAll(value, ['gpt', 'image', '1'])) return true
  return includesAll(value, ['gpt', 'image']) && !isGptImage2(model)
}

function isGeminiImage(model: string) {
  const value = modelNeedle(model)
  return value.includes('gemini') || value.includes('nano-banana') || value.includes('banana')
}

function isGemini31Image(model: string) {
  const value = modelNeedle(model)
  return value.includes('3.1') || value.includes('banana-2') || value.includes('nano-banana-2')
}

function isGeminiProImage(model: string) {
  const value = modelNeedle(model)
  return value.includes('pro-image') || value.includes('banana-pro') || value.includes('nano-banana-pro')
}

function normalizeSize(value: string) {
  const cleaned = value.trim().replace(/×/g, 'x').replace(/\s+/g, '')
  return /^\d+x\d+$/i.test(cleaned) ? cleaned.toLowerCase() : ''
}

function parseSize(value: string): ImageSize | null {
  const normalized = normalizeSize(value)
  if (!normalized) return null
  const [width, height] = normalized.split('x').map(Number)
  return Number.isFinite(width) && Number.isFinite(height) && width > 0 && height > 0
    ? { width, height }
    : null
}

function closestSize(size: ImageSize | null, allowed: readonly string[]) {
  if (!size) return allowed[0]
  const targetRatio = size.width / size.height
  const targetArea = size.width * size.height
  return allowed.reduce((best, candidate) => {
    const current = parseSize(candidate)
    const previous = parseSize(best)
    if (!current || !previous) return best
    const score = (value: ImageSize) => Math.abs(Math.log(targetRatio / (value.width / value.height))) * 2
      + Math.abs(Math.log(targetArea / (value.width * value.height))) * 0.2
    return score(current) < score(previous) ? candidate : best
  }, allowed[0])
}

function normalizeGptImage2Size(size: ImageSize | null) {
  const fallback = size ?? { width: 1024, height: 1024 }
  const round = (value: number) => Math.round(value / GPT_IMAGE_2_STEP) * GPT_IMAGE_2_STEP
  const clamp = (value: number) => Math.min(GPT_IMAGE_2_MAX_SIZE, Math.max(MIN_SIZE, value))
  let width = round(clamp(fallback.width))
  let height = round(clamp(fallback.height))
  if (width / height > 3) width = round(height * 3)
  else if (height / width > 3) height = round(width * 3)
  return `${clamp(width)}x${clamp(height)}`
}

function geminiAspectRatios(model: string) {
  const ratios = [...COMMON_ASPECT_RATIOS]
  return isGemini31Image(model) ? [...ratios, '4:1', '1:4', '8:1', '1:8'] : ratios
}

function closestAspectRatio(size: ImageSize | null, ratios: readonly string[]) {
  if (!size) return ratios[0]
  const target = size.width / size.height
  return ratios.reduce((best, ratio) => {
    const score = (value: string) => {
      const [width, height] = value.split(':').map(Number)
      return Math.abs(Math.log(target / (width / height)))
    }
    return score(ratio) < score(best) ? ratio : best
  }, ratios[0])
}

function inferGeminiImageSize(model: string, size: ImageSize | null) {
  if (!isGemini31Image(model) && !isGeminiProImage(model)) return undefined
  const maxSide = size ? Math.max(size.width, size.height) : 1024
  if (isGemini31Image(model) && maxSide <= 768) return '512'
  if (maxSide <= 1600) return '1K'
  if (maxSide <= 3200) return '2K'
  return '4K'
}

function geminiCanonicalSize(model: string, ratio: string, imageSize?: string) {
  const sizes = isGeminiProImage(model) || isGemini31Image(model) ? GEMINI_3_1K_SIZES : GEMINI_25_1K_SIZES
  const base = parseSize(sizes[ratio] || GEMINI_25_1K_SIZES[ratio] || '1024x1024')
    ?? { width: 1024, height: 1024 }
  const scale = imageSize === '512' ? 0.5 : imageSize === '2K' ? 2 : imageSize === '4K' ? 4 : 1
  return `${Math.round(base.width * scale)}x${Math.round(base.height * scale)}`
}

function hasPromptAspectRatio(prompt: string) {
  return /(?:^|\s)--(?:ar|aspect(?:-ratio)?)\s+\d+(?:\s*[:/]\s*\d+)?\b/i.test(prompt)
}

export function resizeByWidthForAspectRatio(
  currentSize: string,
  aspectRatio: string,
  fallbackSize: string,
) {
  const width = Number(currentSize.match(/\d+/)?.[0])
  const ratioMatch = aspectRatio.match(/^(\d+):(\d+)$/)
  if (!Number.isFinite(width) || width <= 0 || !ratioMatch) return fallbackSize

  const ratioWidth = Number(ratioMatch[1])
  const ratioHeight = Number(ratioMatch[2])
  if (!ratioWidth || !ratioHeight) return fallbackSize

  const normalizedWidth = Math.round(width)
  const height = Math.max(1, Math.round(normalizedWidth * ratioHeight / ratioWidth))
  return `${normalizedWidth}x${height}`
}

export function buildCompatibleImageRequest(request: CompatibleImageRequest) {
  const prompt = request.prompt.trim()
  const parsedSize = parseSize(request.size)
  const extraParams = { ...request.extraParams }

  if (isDalle2(request.model)) return { prompt, size: closestSize(parsedSize, DALLE_2_ALLOWED), extraParams }
  if (isDalle3(request.model)) return { prompt, size: closestSize(parsedSize, DALLE_3_ALLOWED), extraParams }
  if (isGptImage1(request.model)) {
    return {
      prompt,
      size: request.size.trim().toLowerCase() === 'auto' ? 'auto' : closestSize(parsedSize, GPT_IMAGE_1_ALLOWED),
      extraParams,
    }
  }
  if (isGptImage2(request.model)) {
    if (!Object.hasOwn(extraParams, 'aspect_ratio') && !Object.hasOwn(extraParams, 'aspectRatio')) {
      extraParams.aspect_ratio = closestAspectRatio(parsedSize, COMMON_ASPECT_RATIOS)
    }
    return { prompt, size: normalizeGptImage2Size(parsedSize), extraParams }
  }

  if (isGeminiImage(request.model)) {
    const ratio = String(extraParams.aspect_ratio ?? extraParams.aspectRatio ?? closestAspectRatio(parsedSize, geminiAspectRatios(request.model)))
    const imageSize = String(extraParams.image_size ?? extraParams.imageSize ?? inferGeminiImageSize(request.model, parsedSize) ?? '')
    const appendPromptRatio = extraParams.append_prompt_ar !== false
    delete extraParams.append_prompt_ar
    if (!Object.hasOwn(extraParams, 'aspect_ratio') && !Object.hasOwn(extraParams, 'aspectRatio')) extraParams.aspect_ratio = ratio
    if (imageSize && !Object.hasOwn(extraParams, 'image_size') && !Object.hasOwn(extraParams, 'imageSize')) extraParams.image_size = imageSize
    return {
      prompt: appendPromptRatio && !hasPromptAspectRatio(prompt) ? `${prompt} --ar ${ratio}` : prompt,
      size: geminiCanonicalSize(request.model, ratio, imageSize),
      extraParams,
    }
  }

  return { prompt, size: normalizeSize(request.size) || request.size.trim(), extraParams }
}
