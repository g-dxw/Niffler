import { describe, expect, it } from 'vitest'
import { buildCompatibleImageRequest, resizeByWidthForAspectRatio } from '../utils/image-sizing'

describe('image request compatibility', () => {
  it('changes the height from the current width when selecting an aspect ratio', () => {
    expect(resizeByWidthForAspectRatio('976x992', '4:3', '1024x768')).toBe('976x732')
    expect(resizeByWidthForAspectRatio('宽度 1200', '3:4', '768x1024')).toBe('1200x1600')
    expect(resizeByWidthForAspectRatio('auto', '16:9', '1024x576')).toBe('1024x576')
  })

  it('keeps an already compatible gpt-image-2 reference size and supplies its ratio', () => {
    expect(buildCompatibleImageRequest({
      model: 'gpt-image-2',
      prompt: ' 多只小猫 ',
      size: '976x992',
      extraParams: {},
    })).toEqual({
      prompt: '多只小猫',
      size: '976x992',
      extraParams: { aspect_ratio: '1:1' },
    })
  })

  it('rounds gpt-image-2 dimensions to the supported 16 pixel step', () => {
    const result = buildCompatibleImageRequest({
      model: 'gpt-image-2',
      prompt: 'cat',
      size: '977x991',
    })
    expect(result.size).toBe('976x992')
    expect(result.extraParams).toEqual({ aspect_ratio: '1:1' })
  })

  it('sends the selected ratio for a custom gpt-image-2 resolution', () => {
    expect(buildCompatibleImageRequest({
      model: 'gpt-image-2',
      prompt: 'cat',
      size: '976x732',
    }).extraParams).toEqual({ aspect_ratio: '4:3' })
  })

  it('adds Gemini compatibility fields without overriding explicit values', () => {
    expect(buildCompatibleImageRequest({
      model: 'gemini-3-pro-image-preview',
      prompt: 'cat',
      size: '1920x1080',
    })).toMatchObject({
      prompt: 'cat --ar 16:9',
      size: '2752x1536',
      extraParams: { aspect_ratio: '16:9', image_size: '2K' },
    })
  })
})
