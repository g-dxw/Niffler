export type PasswordPolicyLevel = 'weak' | 'medium' | 'strong'
export type PasswordPolicyTranslator = (key: string, params?: Record<string, string | number>) => string
export const PASSWORD_MAX_BYTES = 72

const textEncoder = new TextEncoder()

function getPasswordByteLength(password: string): number {
  return textEncoder.encode(password).length
}

export const PASSWORD_POLICY_OPTION_DEFINITIONS: Array<{
  value: PasswordPolicyLevel
  labelKey: string
  descriptionKey: string
}> = [
  {
    value: 'weak',
    labelKey: 'basicConfig.weak',
    descriptionKey: 'passwordPolicy.hintWeak',
  },
  {
    value: 'medium',
    labelKey: 'basicConfig.medium',
    descriptionKey: 'passwordPolicy.hintMedium',
  },
  {
    value: 'strong',
    labelKey: 'basicConfig.strong',
    descriptionKey: 'passwordPolicy.hintStrong',
  },
]

export function createPasswordPolicyOptions(translate: PasswordPolicyTranslator) {
  return PASSWORD_POLICY_OPTION_DEFINITIONS.map(option => ({
    value: option.value,
    label: translate(option.labelKey).split(' - ')[0],
    description: translate(option.descriptionKey),
  }))
}

export function normalizePasswordPolicyLevel(value: unknown): PasswordPolicyLevel {
  if (value === 'medium' || value === 'strong') {
    return value
  }
  return 'weak'
}

export function getPasswordPolicyHint(level: unknown, translate?: PasswordPolicyTranslator): string {
  switch (normalizePasswordPolicyLevel(level)) {
    case 'medium':
      return translate?.('passwordPolicy.hintMedium') ?? '至少 8 个字符，且需包含字母和数字'
    case 'strong':
      return translate?.('passwordPolicy.hintStrong') ?? '至少 8 个字符，且需包含大写字母、小写字母、数字和特殊字符'
    case 'weak':
    default:
      return translate?.('passwordPolicy.hintWeak') ?? '至少 6 个字符'
  }
}

export function getPasswordPolicyPlaceholder(level: unknown, translate?: PasswordPolicyTranslator): string {
  switch (normalizePasswordPolicyLevel(level)) {
    case 'medium':
      return translate?.('passwordPolicy.placeholderMedium') ?? '至少 8 位，含字母和数字'
    case 'strong':
      return translate?.('passwordPolicy.placeholderStrong') ?? '至少 8 位，含大小写字母、数字和特殊字符'
    case 'weak':
    default:
      return translate?.('passwordPolicy.placeholderWeak') ?? '至少 6 个字符'
  }
}

/**
 * 返回所有未满足的密码策略条件。
 * 空数组 = 密码合规。
 */
export function getPasswordPolicyErrors(password: string, level: unknown, translate?: PasswordPolicyTranslator): string[] {
  if (!password) return []

  const normalized = normalizePasswordPolicyLevel(level)
  const errors: string[] = []

  const byteLength = getPasswordByteLength(password)
  if (byteLength > PASSWORD_MAX_BYTES) {
    errors.push(translate?.('passwordPolicy.maxBytes', { count: PASSWORD_MAX_BYTES }) ?? `长度不能超过${PASSWORD_MAX_BYTES}字节`)
  }

  // 根据策略确定最小长度，不做两段式报错
  const minLen = normalized === 'weak' ? 6 : 8
  if (password.length < minLen) {
    errors.push(translate?.('passwordPolicy.minCharacters', { count: minLen }) ?? `至少 ${minLen} 个字符`)
  }

  if (normalized === 'medium') {
    if (!/[A-Za-z]/.test(password)) errors.push(translate?.('passwordPolicy.includeLetter') ?? '包含字母')
    if (!/[0-9]/.test(password)) errors.push(translate?.('passwordPolicy.includeNumber') ?? '包含数字')
  }

  if (normalized === 'strong') {
    if (!/[A-Z]/.test(password)) errors.push(translate?.('passwordPolicy.includeUppercase') ?? '包含大写字母')
    if (!/[a-z]/.test(password)) errors.push(translate?.('passwordPolicy.includeLowercase') ?? '包含小写字母')
    if (!/[0-9]/.test(password)) errors.push(translate?.('passwordPolicy.includeNumber') ?? '包含数字')
    if (!/[!@#$%^&*()_+\-=[\]{};:'",.<>?/\\|`~]/.test(password)) errors.push(translate?.('passwordPolicy.includeSpecial') ?? '包含特殊字符')
  }

  return errors
}

/**
 * 兼容旧接口：返回单条错误字符串，空字符串表示通过。
 * 多条未满足条件时用顿号连接。
 */
export function validatePasswordByPolicy(password: string, level: unknown, translate?: PasswordPolicyTranslator): string {
  const errors = getPasswordPolicyErrors(password, level, translate)
  if (errors.length === 0) return ''
  if (errors.length === 1 && getPasswordByteLength(password) > PASSWORD_MAX_BYTES) {
    return translate?.('passwordPolicy.passwordMaxBytes', { count: PASSWORD_MAX_BYTES }) ?? `密码${errors[0]}`
  }
  return translate?.('passwordPolicy.requirements', { requirements: errors.join(translate?.('passwordPolicy.separator') ?? '、') }) ?? `密码需要：${errors.join('、')}`
}
import { i18n } from '@/i18n'
