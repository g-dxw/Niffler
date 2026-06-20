import type {
  CreateNifflerUpstreamAccountPayload,
  CreateNifflerUpstreamServicePayload,
  NifflerProtocolKind,
  NifflerUpstreamService,
  NifflerUpstreamServiceCapability,
} from '@/api/niffler-core'

export type NifflerServiceTemplateKey =
  | 'codex_oauth'
  | 'claude_oauth'
  | 'openai_api_key'
  | 'claude_api_key'
  | 'gemini_service_account'
  | 'openai_compatible'
  | 'anthropic_compatible'
  | 'custom'

export type NifflerServiceCapabilityKey = keyof NonNullable<
  CreateNifflerUpstreamServicePayload['capabilities']
>

export type NifflerServiceAuthKind = CreateNifflerUpstreamAccountPayload['auth_kind']

type CapabilityDefaults = Required<
  NonNullable<CreateNifflerUpstreamServicePayload['capabilities']>
>

export interface NifflerServiceTemplate {
  key: NifflerServiceTemplateKey
  label: string
  description: string
  serviceKind: string
  protocolKind: NifflerProtocolKind
  defaultApiFormat: string
  defaultBaseUrl: string
  baseUrlPlaceholder: string
  baseUrlRequired: boolean
  defaultAuthKind: NifflerServiceAuthKind
  capabilities: CapabilityDefaults
}

export interface NifflerServiceCapabilityOption {
  key: NifflerServiceCapabilityKey
  label: string
  description: string
}

export interface NifflerServiceAuthSource {
  service_kind?: string | null
  base_url?: string | null
}

export interface NifflerServiceCapabilityForm {
  protocol_kind: NifflerProtocolKind
  capabilities: CapabilityDefaults
}

export const DEFAULT_NIFFLER_SERVICE_TEMPLATE_KEY: NifflerServiceTemplateKey = 'openai_compatible'

const textModelCapabilities = {
  text: true,
  streaming: true,
  images_endpoint: false,
  openai_responses_image_tool: false,
  model_list: true,
  model_test: true,
} satisfies CapabilityDefaults

export const nifflerServiceTemplates: NifflerServiceTemplate[] = [
  {
    key: 'codex_oauth',
    label: 'Codex / ChatGPT OAuth',
    description: '用于接入 Codex 或 ChatGPT OAuth 账号池，默认开启对话和 Responses 生图工具能力。',
    serviceKind: 'codex',
    protocolKind: 'codex',
    defaultApiFormat: 'codex',
    defaultBaseUrl: '',
    baseUrlPlaceholder: 'OAuth 账号通常不需要填写',
    baseUrlRequired: false,
    defaultAuthKind: 'oauth',
    capabilities: {
      ...textModelCapabilities,
      openai_responses_image_tool: true,
    },
  },
  {
    key: 'claude_oauth',
    label: 'Claude OAuth',
    description: '用于接入 Claude OAuth 账号池，默认按 Anthropic 协议登记文本和流式能力。',
    serviceKind: 'claude',
    protocolKind: 'anthropic',
    defaultApiFormat: 'anthropic',
    defaultBaseUrl: '',
    baseUrlPlaceholder: 'OAuth 账号通常不需要填写',
    baseUrlRequired: false,
    defaultAuthKind: 'oauth',
    capabilities: { ...textModelCapabilities },
  },
  {
    key: 'openai_api_key',
    label: 'OpenAI API Key',
    description: '用于接入官方 OpenAI API Key，默认填入官方地址和 OpenAI 协议能力。',
    serviceKind: 'openai',
    protocolKind: 'openai',
    defaultApiFormat: 'openai',
    defaultBaseUrl: 'https://api.openai.com',
    baseUrlPlaceholder: 'https://api.openai.com',
    baseUrlRequired: true,
    defaultAuthKind: 'api_key',
    capabilities: {
      ...textModelCapabilities,
      images_endpoint: true,
      openai_responses_image_tool: true,
    },
  },
  {
    key: 'claude_api_key',
    label: 'Claude API Key',
    description: '用于接入官方 Anthropic Claude API Key，默认按 Anthropic 协议登记。',
    serviceKind: 'claude',
    protocolKind: 'anthropic',
    defaultApiFormat: 'anthropic',
    defaultBaseUrl: 'https://api.anthropic.com',
    baseUrlPlaceholder: 'https://api.anthropic.com',
    baseUrlRequired: true,
    defaultAuthKind: 'api_key',
    capabilities: { ...textModelCapabilities },
  },
  {
    key: 'gemini_service_account',
    label: 'Gemini Service Account',
    description: '用于登记 Gemini 服务账号类接入。当前只记录服务形态，不保存真实凭证。',
    serviceKind: 'gemini',
    protocolKind: 'gemini',
    defaultApiFormat: 'gemini',
    defaultBaseUrl: 'https://generativelanguage.googleapis.com',
    baseUrlPlaceholder: 'https://generativelanguage.googleapis.com',
    baseUrlRequired: true,
    defaultAuthKind: 'custom_header',
    capabilities: { ...textModelCapabilities },
  },
  {
    key: 'openai_compatible',
    label: 'OpenAI 兼容接口',
    description: '用于接入第三方 OpenAI 兼容服务，可手动声明图片接口和 Responses 生图工具能力。',
    serviceKind: 'openai_compatible',
    protocolKind: 'openai',
    defaultApiFormat: 'openai',
    defaultBaseUrl: '',
    baseUrlPlaceholder: 'https://api.example.com',
    baseUrlRequired: true,
    defaultAuthKind: 'api_key',
    capabilities: { ...textModelCapabilities },
  },
  {
    key: 'anthropic_compatible',
    label: 'Anthropic 兼容接口',
    description: '用于接入第三方 Anthropic 兼容服务，不显示 OpenAI Responses 生图工具能力。',
    serviceKind: 'anthropic_compatible',
    protocolKind: 'anthropic',
    defaultApiFormat: 'anthropic',
    defaultBaseUrl: '',
    baseUrlPlaceholder: 'https://api.example.com',
    baseUrlRequired: true,
    defaultAuthKind: 'api_key',
    capabilities: { ...textModelCapabilities },
  },
  {
    key: 'custom',
    label: '自定义接口',
    description: '用于暂时无法归类的上游服务，需要管理员自行确认协议和能力。',
    serviceKind: 'custom',
    protocolKind: 'custom',
    defaultApiFormat: 'custom',
    defaultBaseUrl: '',
    baseUrlPlaceholder: 'https://api.example.com',
    baseUrlRequired: false,
    defaultAuthKind: 'custom_header',
    capabilities: { ...textModelCapabilities },
  },
]

const templatesByKey = new Map(nifflerServiceTemplates.map(template => [template.key, template]))

export function getNifflerServiceTemplate(
  key: NifflerServiceTemplateKey
): NifflerServiceTemplate {
  return templatesByKey.get(key) ?? templatesByKey.get(DEFAULT_NIFFLER_SERVICE_TEMPLATE_KEY)!
}

export function buildNifflerServiceFormFromTemplate(
  key: NifflerServiceTemplateKey,
  current?: CreateNifflerUpstreamServicePayload
): CreateNifflerUpstreamServicePayload {
  const template = getNifflerServiceTemplate(key)
  return {
    display_name: current?.display_name ?? '',
    service_kind: template.serviceKind,
    protocol_kind: template.protocolKind,
    default_api_format: template.defaultApiFormat,
    base_url: current?.base_url?.trim() ? current.base_url : template.defaultBaseUrl,
    cost_multiplier: current?.cost_multiplier ?? 1,
    is_active: current?.is_active ?? true,
    capabilities: { ...template.capabilities },
  }
}

export function getDefaultAuthKindForService(service?: NifflerServiceAuthSource | null): NifflerServiceAuthKind {
  const serviceKind = service?.service_kind?.trim().toLowerCase()
  if (serviceKind === 'codex') return 'oauth'
  if (serviceKind === 'claude') return service?.base_url?.trim() ? 'api_key' : 'oauth'
  if (serviceKind === 'openai' || serviceKind === 'openai_compatible' || serviceKind === 'anthropic_compatible') {
    return 'api_key'
  }
  if (serviceKind === 'gemini' || serviceKind === 'custom') return 'custom_header'
  return 'oauth'
}

export function getServiceKindLabel(serviceKind: string): string {
  const normalized = serviceKind.trim().toLowerCase()
  const labels: Record<string, string> = {
    codex: 'Codex / ChatGPT OAuth',
    claude: 'Claude',
    openai: 'OpenAI',
    gemini: 'Gemini',
    openai_compatible: 'OpenAI 兼容接口',
    anthropic_compatible: 'Anthropic 兼容接口',
    custom: '自定义接口',
  }
  return labels[normalized] ?? serviceKind
}

export function filterCapabilityOptionsForProtocol(
  options: NifflerServiceCapabilityOption[],
  protocolKind: NifflerProtocolKind
): NifflerServiceCapabilityOption[] {
  return options.filter((option) => {
    if (option.key === 'openai_responses_image_tool') {
      return protocolKind === 'openai' || protocolKind === 'codex'
    }
    if (option.key === 'images_endpoint') {
      return protocolKind === 'openai' || protocolKind === 'codex' || protocolKind === 'custom'
    }
    return true
  })
}

export function inferNifflerServiceProtocolKind(
  service?: Pick<NifflerUpstreamService, 'service_kind' | 'default_api_format' | 'config'> | null,
  capabilities: NifflerUpstreamServiceCapability[] = []
): NifflerProtocolKind {
  const storedProtocol = capabilities[0]?.protocol_kind
  if (storedProtocol) return storedProtocol

  const configProtocol = readProtocolKindFromConfig(service?.config)
  if (configProtocol) return configProtocol

  const format = service?.default_api_format?.trim().toLowerCase()
  if (format === 'codex') return 'codex'
  if (format === 'anthropic') return 'anthropic'
  if (format === 'gemini') return 'gemini'
  if (format === 'openai' || format === 'openai:image') return 'openai'

  const serviceKind = service?.service_kind?.trim().toLowerCase()
  if (serviceKind === 'codex') return 'codex'
  if (serviceKind === 'claude' || serviceKind === 'anthropic_compatible') return 'anthropic'
  if (serviceKind === 'gemini') return 'gemini'
  if (serviceKind === 'openai' || serviceKind === 'openai_compatible') return 'openai'
  return 'custom'
}

export function buildNifflerServiceCapabilityForm(
  service?: Pick<NifflerUpstreamService, 'service_kind' | 'default_api_format' | 'config'> | null,
  capabilities: NifflerUpstreamServiceCapability[] = []
): NifflerServiceCapabilityForm {
  const protocolKind = inferNifflerServiceProtocolKind(service, capabilities)
  const form: CapabilityDefaults = { ...textModelCapabilities }
  for (const capability of capabilities) {
    form[capability.capability_kind] = capability.is_enabled
  }
  clearUnsupportedNifflerCapabilities(form, protocolKind)
  return {
    protocol_kind: protocolKind,
    capabilities: form,
  }
}

export function validateNifflerServiceCapabilities(
  form: NifflerServiceCapabilityForm
): string[] {
  const issues: string[] = []
  if (
    form.capabilities.openai_responses_image_tool
    && form.protocol_kind !== 'openai'
    && form.protocol_kind !== 'codex'
  ) {
    issues.push('OpenAI Responses 生图工具只能用于 OpenAI 或 Codex 协议。')
  }
  if (
    form.capabilities.images_endpoint
    && form.protocol_kind !== 'openai'
    && form.protocol_kind !== 'codex'
    && form.protocol_kind !== 'custom'
  ) {
    issues.push('图片接口只适用于 OpenAI、Codex 或自定义协议。')
  }
  return issues
}

export function enabledCapabilityLabels(
  options: NifflerServiceCapabilityOption[],
  capabilities: Partial<Record<NifflerServiceCapabilityKey, boolean>>
): string[] {
  return options
    .filter(option => capabilities[option.key])
    .map(option => option.label)
}

function clearUnsupportedNifflerCapabilities(
  capabilities: CapabilityDefaults,
  protocolKind: NifflerProtocolKind
) {
  if (protocolKind !== 'openai' && protocolKind !== 'codex') {
    capabilities.openai_responses_image_tool = false
  }
  if (protocolKind !== 'openai' && protocolKind !== 'codex' && protocolKind !== 'custom') {
    capabilities.images_endpoint = false
  }
}

function readProtocolKindFromConfig(config?: Record<string, unknown> | null): NifflerProtocolKind | null {
  const value = typeof config?.protocol_kind === 'string' ? config.protocol_kind : ''
  if (value === 'openai' || value === 'anthropic' || value === 'gemini' || value === 'codex' || value === 'custom') {
    return value
  }
  return null
}
