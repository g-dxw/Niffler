import type {
  ImageGenerationForm,
  ImageResponseFormat,
  ImageStudioSettings,
} from '../types'

export interface ImageSubmissionSnapshot {
  apiKeyId: string
  model: string
  responseFormat: ImageResponseFormat
  form: ImageGenerationForm
}

export function createImageSubmissionSnapshot(
  settings: ImageStudioSettings,
  form: ImageGenerationForm,
): ImageSubmissionSnapshot {
  return {
    apiKeyId: settings.selectedKeyId,
    model: settings.model.trim(),
    responseFormat: settings.responseFormat,
    form: {
      ...form,
      inputImages: [...form.inputImages],
      maskImage: form.maskImage,
    },
  }
}
