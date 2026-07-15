import { describe, expect, it } from 'vitest'
import { isImageGenerationModel } from '../utils/model-capability'

function model(
  name: string,
  supportedCapabilities: string[] | Record<string, unknown> | null,
  config: Record<string, unknown> | null = null,
) {
  return { name, supported_capabilities: supportedCapabilities, config }
}

describe('image generation model capability', () => {
  it('prefers the exact image_generation capability', () => {
    expect(isImageGenerationModel(model('custom-renderer', ['image_generation']))).toBe(true)
    expect(isImageGenerationModel(model('custom-renderer', { image_generation: true }))).toBe(true)
  })

  it('accepts an explicit image generation config', () => {
    expect(isImageGenerationModel(model('gemini-image-preview', ['vision'], { image_generation: true }))).toBe(true)
  })

  it('rejects image input, understanding, and embedding models', () => {
    expect(isImageGenerationModel(model('my-image-input-model', ['image_input']))).toBe(false)
    expect(isImageGenerationModel(model('vision-image-model', ['image_understanding']))).toBe(false)
    expect(isImageGenerationModel(model('image-embedding-v2', { image_embedding: true }))).toBe(false)
  })

  it('respects an explicit image_generation disable before using name hints', () => {
    expect(isImageGenerationModel(model('gpt-image-disabled', null, { image_generation: false }))).toBe(false)
    expect(isImageGenerationModel(model('flux-disabled', { image_generation: false }))).toBe(false)
  })

  it('uses only known generation model families as a legacy fallback', () => {
    expect(isImageGenerationModel(model('gpt-image-2', null))).toBe(true)
    expect(isImageGenerationModel(model('dall-e-3', null))).toBe(true)
    expect(isImageGenerationModel(model('google/imagen-3', null))).toBe(true)
    expect(isImageGenerationModel(model('imagenet-classifier', null))).toBe(false)
    expect(isImageGenerationModel(model('my-flux-capacitor', null))).toBe(false)
    expect(isImageGenerationModel(model('generic-image-classifier', null))).toBe(false)
  })
})
