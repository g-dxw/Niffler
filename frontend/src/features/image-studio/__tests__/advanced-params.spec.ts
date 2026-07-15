import { describe, expect, it } from 'vitest'
import { parseAdvancedParams } from '../utils/advanced-params'

describe('advanced image parameters', () => {
  it('accepts compatible parameters and strips internal metadata', () => {
    expect(parseAdvancedParams('{"quality":"high","input_fidelity":"high","_batchId":"1"}')).toEqual({
      quality: 'high',
      input_fidelity: 'high',
    })
  })

  it.each(['model', 'prompt', 'n', 'image', 'mask', 'response_format', 'stream', 'partial_images'])('rejects reserved field %s', field => {
    expect(() => parseAdvancedParams(JSON.stringify({ [field]: 'override' }))).toThrow(`高级参数不能覆盖 ${field}`)
  })

  it('rejects invalid JSON and arrays', () => {
    expect(() => parseAdvancedParams('{')).toThrow('高级参数不是有效的 JSON')
    expect(() => parseAdvancedParams('[]')).toThrow('高级参数必须是一个 JSON 对象')
  })
})
