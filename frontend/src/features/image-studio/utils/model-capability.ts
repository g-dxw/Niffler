const IMAGE_MODEL_HINTS = [
  'gpt-image',
  'image',
  'dall-e',
  'dalle',
  'flux',
  'stable-diffusion',
  'sdxl',
  'imagen',
  'seedream',
  'nano-banana',
]

function capabilityContainsImage(value: string[] | Record<string, unknown> | null) {
  if (Array.isArray(value)) {
    return value.some(item => item.toLowerCase().includes('image'))
  }
  if (value && typeof value === 'object') {
    return Object.entries(value).some(([key, enabled]) => key.toLowerCase().includes('image') && enabled !== false)
  }
  return false
}

export function isImageGenerationModel(model: {
  name: string
  config: Record<string, unknown> | null
  supported_capabilities: string[] | Record<string, unknown> | null
}) {
  if (model.config?.image_generation === true) return true
  if (capabilityContainsImage(model.supported_capabilities)) return true
  const normalized = model.name.toLowerCase()
  return IMAGE_MODEL_HINTS.some(hint => normalized.includes(hint))
}
