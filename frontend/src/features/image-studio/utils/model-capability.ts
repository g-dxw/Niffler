const IMAGE_MODEL_FAMILY_PATTERNS = [
  /(?:^|\/)gpt-image(?:$|[/_.:-]|\d)/,
  /(?:^|\/)dall-e(?:$|[/_.:-]|\d)/,
  /(?:^|\/)dalle(?:$|[/_.:-]|\d)/,
  /(?:^|\/)flux(?:$|[/_.:-]|\d)/,
  /(?:^|\/)stable-diffusion(?:$|[/_.:-]|\d)/,
  /(?:^|\/)sdxl(?:$|[/_.:-]|\d)/,
  /(?:^|\/)imagen(?:$|[/_.:-]|\d)/,
  /(?:^|\/)seedream(?:$|[/_.:-]|\d)/,
  /(?:^|\/)nano-banana(?:$|[/_.:-]|\d)/,
]

function imageGenerationCapabilityDecision(
  value: string[] | Record<string, unknown> | null,
): boolean | undefined {
  if (Array.isArray(value)) {
    return value.some(item => item.trim().toLowerCase() === 'image_generation') || undefined
  }
  if (value && typeof value === 'object') {
    const entry = Object.entries(value).find(([key]) => key.trim().toLowerCase() === 'image_generation')
    if (entry) return entry[1] === true
  }
  return undefined
}

function declaresNonGenerationImageCapability(value: string[] | Record<string, unknown> | null) {
  const names = Array.isArray(value)
    ? value
    : value && typeof value === 'object'
      ? Object.entries(value).filter(([, enabled]) => enabled !== false).map(([key]) => key)
      : []
  return names.some(item => {
    const capability = item.trim().toLowerCase()
    return capability === 'image_input'
      || capability === 'image_understanding'
      || capability === 'image_embedding'
      || capability === 'vision'
      || capability === 'embedding'
  })
}

export function isImageGenerationModel(model: {
  name: string
  config: Record<string, unknown> | null
  supported_capabilities: string[] | Record<string, unknown> | null
}) {
  const configDecision = model.config && Object.prototype.hasOwnProperty.call(model.config, 'image_generation')
    ? model.config.image_generation === true
    : undefined
  const capabilityDecision = imageGenerationCapabilityDecision(model.supported_capabilities)
  if (configDecision === true || capabilityDecision === true) return true
  if (configDecision === false || capabilityDecision === false) return false
  if (declaresNonGenerationImageCapability(model.supported_capabilities)) return false
  const normalized = model.name.trim().toLowerCase()
  return IMAGE_MODEL_FAMILY_PATTERNS.some(pattern => pattern.test(normalized))
}
