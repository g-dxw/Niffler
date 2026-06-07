import type {
  CreateNifflerUpstreamAccountPayload,
  NifflerUpstreamAccount,
} from '@/api/niffler-core'

export type NifflerAccountAuthKind = CreateNifflerUpstreamAccountPayload['auth_kind']

export interface NifflerAccountAuthGuide {
  title: string
  description: string
  namePlaceholder: string
  contactHint: string
}

export function getNifflerAccountAuthGuide(authKind: NifflerAccountAuthKind): NifflerAccountAuthGuide {
  if (authKind === 'api_key') {
    return {
      title: 'API Key 账号',
      description: '当前只登记账号身份和调度信息，不保存真实 API Key。凭证保存会在后续凭证片落地。',
      namePlaceholder: '例如 OpenAI 主 Key、cc-max 备用 Key',
      contactHint: '邮箱和手机号只是管理员识别账号用，可不填。',
    }
  }

  if (authKind === 'custom_header') {
    return {
      title: '自定义 Header / Service Account',
      description: '当前只登记账号身份，不保存 Header、JSON 或私钥内容。',
      namePlaceholder: '例如 Gemini Service Account、内网 Header 账号',
      contactHint: '建议填写能识别账号归属的邮箱、手机号或备注。',
    }
  }

  return {
    title: 'OAuth 账号',
    description: '当前只登记 OAuth 账号身份，不发起授权，也不保存 Refresh Token。',
    namePlaceholder: '例如 codex-plus 主账号、Claude Team 账号',
    contactHint: 'OAuth 账号建议至少填写邮箱或手机号，方便后续调度和问题排查。',
  }
}

export function formatNifflerAccountTestStatus(account: Pick<
  NifflerUpstreamAccount,
  'last_test_error' | 'last_tested_at_unix_ms'
>): string {
  if (account.last_test_error?.trim()) {
    return '测试失败'
  }
  if (account.last_tested_at_unix_ms) {
    return '测试通过'
  }
  return '未测试'
}

export function formatNifflerUnixMs(value?: number | null): string {
  if (!value || !Number.isFinite(value) || value <= 0) {
    return '-'
  }
  return new Date(value).toLocaleString('zh-CN')
}
