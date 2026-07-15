export type ImageResponseFormat = 'url' | 'b64_json'

export type ImageTaskStatus = 'pending' | 'running' | 'success' | 'error' | 'cancelled'

export type ImageTaskMode = 'generate' | 'edit'

export interface ImageStudioSettings {
  selectedKeyId: string
  model: string
  responseFormat: ImageResponseFormat
  concurrency: number
}

export interface ImageGenerationForm {
  prompt: string
  count: number
  size: string
  quality: string
  background: string
  outputFormat: string
  advancedJson: string
  inputImages: File[]
  maskImage: File | null
}

export interface ImageTask {
  id: string
  apiKeyId: string
  mode: ImageTaskMode
  prompt: string
  model: string
  size: string
  responseFormat: ImageResponseFormat
  status: ImageTaskStatus
  createdAt: number
  startedAt?: number
  finishedAt?: number
  imageUrl?: string
  b64Json?: string
  imageMimeType?: string
  imageSize?: number
  imageCached?: boolean
  error?: string
  extraParams?: Record<string, unknown>
  inputImageCount?: number
  hasMask?: boolean
}

export interface ImageTaskCredential {
  apiKeyId: string
  apiKey: string
  baseUrl: string
}

export interface ImageTaskConfiguration {
  model: string
  responseFormat: ImageResponseFormat
}

export interface GenerateImageParams {
  apiKey: string
  baseUrl: string
  model: string
  prompt: string
  size?: string
  responseFormat: ImageResponseFormat
  extraParams?: Record<string, unknown>
  signal?: AbortSignal
}

export interface EditImageParams extends GenerateImageParams {
  images: File[]
  mask?: File | null
}

export interface GenerateImageResult {
  imageUrl: string
  b64Json?: string
  mimeType: string
}

export interface PendingImageInputs {
  images: File[]
  mask?: File | null
}

export interface ImageModelOption {
  id: string
  name: string
  displayName: string
}

export interface ImageApiKeyOption {
  id: string
  name: string
  display: string
}
