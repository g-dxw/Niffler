import type { ImageGenerationForm, ImageStudioSettings } from './types'

export const DEFAULT_IMAGE_STUDIO_SETTINGS: ImageStudioSettings = {
  selectedKeyId: '',
  model: '',
  responseFormat: 'url',
  concurrency: 3,
}

export const DEFAULT_IMAGE_GENERATION_FORM: ImageGenerationForm = {
  prompt: '',
  count: 1,
  size: '1024x1024',
  quality: 'auto',
  background: 'auto',
  outputFormat: 'auto',
  advancedJson: '',
  inputImages: [],
  maskImage: null,
}

export const IMAGE_SIZE_PRESETS = [
  '1024x1024',
  '1536x1024',
  '1024x1536',
  '1792x1024',
  '1024x1792',
] as const

export const IMAGE_ASPECT_RATIO_PRESETS = [
  { ratio: '1:1', size: '1024x1024' },
  { ratio: '4:3', size: '1024x768' },
  { ratio: '3:4', size: '768x1024' },
  { ratio: '3:2', size: '1536x1024' },
  { ratio: '2:3', size: '1024x1536' },
  { ratio: '16:9', size: '1024x576' },
  { ratio: '9:16', size: '576x1024' },
  { ratio: '5:4', size: '1280x1024' },
  { ratio: '4:5', size: '1024x1280' },
  { ratio: '21:9', size: '1792x768' },
] as const

export const MAX_INPUT_IMAGE_BYTES = 20 * 1024 * 1024
export const ACCEPTED_INPUT_IMAGE_TYPES = ['image/png', 'image/jpeg', 'image/webp'] as const
