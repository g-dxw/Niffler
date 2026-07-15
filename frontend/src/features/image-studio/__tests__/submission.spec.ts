import { describe, expect, it } from 'vitest'
import { createImageSubmissionSnapshot } from '../utils/submission'
import type { ImageGenerationForm, ImageStudioSettings } from '../types'

describe('image submission snapshot', () => {
  it('keeps the key, model and form selected before credential loading starts', () => {
    const settings: ImageStudioSettings = {
      selectedKeyId: 'key-a',
      model: 'image-a',
      responseFormat: 'url',
      concurrency: 1,
    }
    const reference = new File(['a'], 'reference.png', { type: 'image/png' })
    const form: ImageGenerationForm = {
      prompt: 'cat',
      count: 2,
      size: '1024x1024',
      quality: 'auto',
      background: 'auto',
      outputFormat: 'auto',
      advancedJson: '',
      inputImages: [reference],
      maskImage: null,
    }

    const snapshot = createImageSubmissionSnapshot(settings, form)
    settings.selectedKeyId = 'key-b'
    settings.model = 'image-b'
    settings.responseFormat = 'b64_json'
    form.prompt = 'dog'
    form.inputImages.length = 0

    expect(snapshot).toMatchObject({
      apiKeyId: 'key-a',
      model: 'image-a',
      responseFormat: 'url',
      form: { prompt: 'cat', count: 2 },
    })
    expect(snapshot.form.inputImages).toEqual([reference])
  })
})
