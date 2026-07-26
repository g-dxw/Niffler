import {
  ACCEPTED_INPUT_IMAGE_TYPES,
  MAX_INPUT_IMAGE_BYTES,
} from '../constants'
import { i18n } from '@/i18n'

const t = i18n.global.t

export function validateInputImages(files: File[]): File[] {
  for (const file of files) {
    if (!(ACCEPTED_INPUT_IMAGE_TYPES as readonly string[]).includes(file.type)) {
      throw new Error(t('imageTaskErrors.unsupportedImage', { name: file.name }))
    }
    if (file.size > MAX_INPUT_IMAGE_BYTES) {
      throw new Error(t('imageTaskErrors.tooLarge', { name: file.name }))
    }
  }
  return files
}

export function imageFileAccept(): string {
  return ACCEPTED_INPUT_IMAGE_TYPES.join(',')
}
