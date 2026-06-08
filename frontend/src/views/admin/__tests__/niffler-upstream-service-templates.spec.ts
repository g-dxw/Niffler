import { describe, expect, it } from 'vitest'

import {
  buildNifflerServiceCapabilityForm,
  buildNifflerServiceFormFromTemplate,
  enabledCapabilityLabels,
  filterCapabilityOptionsForProtocol,
  getDefaultAuthKindForService,
  getServiceKindLabel,
  inferNifflerServiceProtocolKind,
  validateNifflerServiceCapabilities,
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

  it('builds capability form from stored service capabilities', () => {
    const form = buildNifflerServiceCapabilityForm(
      { service_kind: 'openai_compatible', default_api_format: 'openai', config: null },
      [
        {
          id: 'cap-1',
          upstream_service_id: 'svc-1',
          protocol_kind: 'openai',
          capability_kind: 'images_endpoint',
          is_enabled: true,
          config: null,
          created_at_unix_ms: 1,
          updated_at_unix_ms: 1,
        },
        {
          id: 'cap-2',
          upstream_service_id: 'svc-1',
          protocol_kind: 'openai',
          capability_kind: 'openai_responses_image_tool',
          is_enabled: true,
          config: null,
          created_at_unix_ms: 1,
          updated_at_unix_ms: 1,
        },
      ]
    )

    expect(form.protocol_kind).toBe('openai')
    expect(form.capabilities.images_endpoint).toBe(true)
    expect(form.capabilities.openai_responses_image_tool).toBe(true)
  })

  it('clears OpenAI image tool when stored capabilities are used with unsupported protocol', () => {
    const form = buildNifflerServiceCapabilityForm(
      { service_kind: 'anthropic_compatible', default_api_format: 'anthropic', config: null },
      [
        {
          id: 'cap-1',
          upstream_service_id: 'svc-1',
          protocol_kind: 'anthropic',
          capability_kind: 'openai_responses_image_tool',
          is_enabled: true,
          config: null,
          created_at_unix_ms: 1,
          updated_at_unix_ms: 1,
        },
      ]
    )

    expect(form.protocol_kind).toBe('anthropic')
    expect(form.capabilities.openai_responses_image_tool).toBe(false)
  })

  it('infers protocol from service config before falling back to api format', () => {
    expect(inferNifflerServiceProtocolKind({
      service_kind: 'custom',
      default_api_format: 'custom',
      config: { protocol_kind: 'codex' },
    })).toBe('codex')
    expect(inferNifflerServiceProtocolKind({
      service_kind: 'custom',
      default_api_format: 'gemini',
      config: null,
    })).toBe('gemini')
  })

  it('validates image capability and protocol combinations', () => {
    expect(validateNifflerServiceCapabilities({
      protocol_kind: 'anthropic',
      capabilities: {
        text: true,
        streaming: true,
        images_endpoint: false,
        openai_responses_image_tool: true,
        model_list: true,
        model_test: true,
      },
    })).toContain('OpenAI Responses 生图工具只能用于 OpenAI 或 Codex 协议。')
  })

  it('formats enabled capability labels for selected service summaries', () => {
    expect(enabledCapabilityLabels(capabilityOptions, {
      text: true,
      images_endpoint: true,
      openai_responses_image_tool: false,
    })).toEqual(['文本对话', '图片接口'])
  })
})
