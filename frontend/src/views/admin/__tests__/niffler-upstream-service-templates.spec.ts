import { describe, expect, it } from 'vitest'

import {
  buildNifflerServiceFormFromTemplate,
  filterCapabilityOptionsForProtocol,
  getDefaultAuthKindForService,
  getServiceKindLabel,
  type NifflerServiceCapabilityOption,
} from '../niffler-upstream-service-templates'

const capabilityOptions: NifflerServiceCapabilityOption[] = [
  { key: 'text', label: '文本对话', description: '' },
  { key: 'streaming', label: '流式响应', description: '' },
  { key: 'images_endpoint', label: '图片接口', description: '' },
  { key: 'openai_responses_image_tool', label: 'Responses 生图工具', description: '' },
  { key: 'model_list', label: '模型列表', description: '' },
  { key: 'model_test', label: '模型测试', description: '' },
]

describe('niffler upstream service templates', () => {
  it('builds OpenAI compatible service defaults without requiring manual protocol fields', () => {
    const form = buildNifflerServiceFormFromTemplate('openai_compatible', {
      display_name: 'cc-max',
      service_kind: 'custom',
      protocol_kind: 'custom',
      default_api_format: 'custom',
      base_url: '',
      cost_multiplier: 0.8,
      is_active: true,
      capabilities: {},
    })

    expect(form).toMatchObject({
      display_name: 'cc-max',
      service_kind: 'openai_compatible',
      protocol_kind: 'openai',
      default_api_format: 'openai',
      cost_multiplier: 0.8,
      is_active: true,
    })
    expect(form.capabilities?.openai_responses_image_tool).toBe(false)
  })

  it('keeps OpenAI image tool hidden for Anthropic and Gemini protocols', () => {
    expect(
      filterCapabilityOptionsForProtocol(capabilityOptions, 'anthropic').map(option => option.key)
    ).not.toContain('openai_responses_image_tool')
    expect(
      filterCapabilityOptionsForProtocol(capabilityOptions, 'gemini').map(option => option.key)
    ).not.toContain('openai_responses_image_tool')
  })

  it('allows OpenAI image capabilities only on OpenAI, Codex, or custom-compatible protocols', () => {
    expect(
      filterCapabilityOptionsForProtocol(capabilityOptions, 'openai').map(option => option.key)
    ).toEqual(capabilityOptions.map(option => option.key))
    expect(
      filterCapabilityOptionsForProtocol(capabilityOptions, 'codex').map(option => option.key)
    ).toEqual(capabilityOptions.map(option => option.key))
  })

  it('uses the service template to choose the default account auth kind', () => {
    expect(getDefaultAuthKindForService({ service_kind: 'openai_compatible' })).toBe('api_key')
    expect(getDefaultAuthKindForService({ service_kind: 'codex' })).toBe('oauth')
    expect(getDefaultAuthKindForService({ service_kind: 'gemini' })).toBe('custom_header')
    expect(getDefaultAuthKindForService({ service_kind: 'claude' })).toBe('oauth')
    expect(getDefaultAuthKindForService({ service_kind: 'claude', base_url: 'https://api.anthropic.com' })).toBe('api_key')
  })

  it('displays a readable label for known service kinds', () => {
    expect(getServiceKindLabel('openai_compatible')).toBe('OpenAI 兼容接口')
    expect(getServiceKindLabel('claude')).toBe('Claude')
    expect(getServiceKindLabel('unknown-provider')).toBe('unknown-provider')
  })
})
