import type { PublicGlobalModel } from '@/api/public-models'

export const MODEL_MANUFACTURERS = [
  { id: 'openai', label: 'OpenAI' },
  { id: 'anthropic', label: 'Anthropic' },
  { id: 'google', label: 'Google' },
  { id: 'deepseek', label: 'DeepSeek' },
  { id: 'alibaba', label: 'Alibaba / Qwen' },
  { id: 'zhipu', label: 'Zhipu AI' },
  { id: 'xai', label: 'xAI' },
  { id: 'moonshot', label: 'Moonshot AI' },
  { id: 'minimax', label: 'MiniMax' },
  { id: 'bytedance', label: 'ByteDance' },
  { id: 'xiaomi', label: 'Xiaomi' },
  { id: 'baidu', label: 'Baidu' },
  { id: 'meta', label: 'Meta' },
  { id: 'mistral', label: 'Mistral AI' },
  { id: 'cohere', label: 'Cohere' },
] as const

export type ModelManufacturerId = typeof MODEL_MANUFACTURERS[number]['id'] | 'other'

const manufacturerLabels = new Map<string, string>(
  MODEL_MANUFACTURERS.map(manufacturer => [manufacturer.id, manufacturer.label]),
)

export function modelManufacturerId(
  model: Pick<PublicGlobalModel, 'name' | 'display_name'>,
): ModelManufacturerId {
  const name = `${model.name} ${model.display_name || ''}`.trim().toLowerCase()

  if (/^(gpt|o[134](?:\b|-)|codex|text-embedding|dall-e|whisper|tts)/.test(name)) return 'openai'
  if (/^(claude|anthropic)/.test(name)) return 'anthropic'
  if (/^(gemini|gemma|imagen|veo)/.test(name)) return 'google'
  if (/^deepseek/.test(name)) return 'deepseek'
  if (/^(qwen|tongyi)/.test(name)) return 'alibaba'
  if (/^(glm|chatglm|zhipu)/.test(name)) return 'zhipu'
  if (/^grok/.test(name)) return 'xai'
  if (/^(kimi|moonshot)/.test(name)) return 'moonshot'
  if (/^minimax/.test(name)) return 'minimax'
  if (/^(doubao|seed)/.test(name)) return 'bytedance'
  if (/^(mimo|xiaomi)/.test(name)) return 'xiaomi'
  if (/^(ernie|wenxin|baidu)/.test(name)) return 'baidu'
  if (/^(llama|meta)/.test(name)) return 'meta'
  if (/^(mistral|mixtral|codestral|pixtral)/.test(name)) return 'mistral'
  if (/^(command-r|command-a|cohere)/.test(name)) return 'cohere'
  return 'other'
}

export function modelManufacturerLabel(
  model: Pick<PublicGlobalModel, 'name' | 'display_name'>,
  otherLabel: string,
): string {
  return manufacturerLabels.get(modelManufacturerId(model)) || otherLabel
}
