import { describe, expect, it } from 'vitest'

import { modelManufacturerId, modelManufacturerLabel } from '../model-manufacturers'

describe('public model manufacturers', () => {
  it.each([
    ['gpt-5.4', 'openai'],
    ['claude-sonnet-4-6', 'anthropic'],
    ['gemini-2.5-pro', 'google'],
    ['deepseek-v3', 'deepseek'],
    ['qwen3-max', 'alibaba'],
    ['glm-4.5', 'zhipu'],
    ['grok-4', 'xai'],
    ['kimi-k2', 'moonshot'],
    ['doubao-seed-1.6', 'bytedance'],
    ['llama-4-maverick', 'meta'],
    ['mistral-large', 'mistral'],
  ])('classifies %s as %s', (name, manufacturer) => {
    expect(modelManufacturerId({ name, display_name: null })).toBe(manufacturer)
  })

  it('uses the localized fallback for unknown model names', () => {
    const model = { name: 'private-model-v1', display_name: 'Private Model' }
    expect(modelManufacturerId(model)).toBe('other')
    expect(modelManufacturerLabel(model, '其他厂商')).toBe('其他厂商')
  })
})
