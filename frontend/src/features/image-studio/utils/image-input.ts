import {
  ACCEPTED_INPUT_IMAGE_TYPES,
  MAX_INPUT_IMAGE_BYTES,
} from '../constants'

export function validateInputImages(files: File[]): File[] {
  for (const file of files) {
    if (!(ACCEPTED_INPUT_IMAGE_TYPES as readonly string[]).includes(file.type)) {
      throw new Error(`${file.name} 格式不支持，请使用 PNG、JPEG 或 WebP`)
    }
    if (file.size > MAX_INPUT_IMAGE_BYTES) {
      throw new Error(`${file.name} 超过 20 MB`)
    }
  }
  return files
}

export function imageFileAccept(): string {
  return ACCEPTED_INPUT_IMAGE_TYPES.join(',')
}
