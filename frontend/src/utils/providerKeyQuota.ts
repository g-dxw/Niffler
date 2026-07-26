import type {
  ProviderKeyStatusSnapshot,
  QuotaStatusSnapshot,
  QuotaWindowSnapshot,
} from '@/api/endpoints/types/statusSnapshot'
import { i18n } from '@/i18n'

export interface ProviderKeyQuotaCarrier {
  account_quota?: string | null
  status_snapshot?: ProviderKeyStatusSnapshot | null
}

function normalizeText(value: unknown): string | null {
  if (typeof value !== 'string') return null
  const text = value.trim()
  return text || null
}

function t(key: string, params?: Record<string, unknown>): string {
  return i18n.global.t(`quotaUi.${key}`, params)
}

function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 0
  if (value < 0) return 0
  if (value > 100) return 100
  return value
}

function formatPercent(value: number): string {
  return `${clampPercent(value).toFixed(1)}%`
}

function getQuotaSnapshot(
  input: ProviderKeyQuotaCarrier,
): QuotaStatusSnapshot | null {
  return input.status_snapshot?.quota ?? null
}

function getQuotaProviderType(
  quota: QuotaStatusSnapshot | null | undefined,
  fallbackProviderType?: string | null,
): string {
  const snapshotProviderType = normalizeText(quota?.provider_type)?.toLowerCase()
  if (snapshotProviderType) return snapshotProviderType
  return normalizeText(fallbackProviderType)?.toLowerCase() || ''
}

function getQuotaWindows(
  quota: QuotaStatusSnapshot | null | undefined,
): QuotaWindowSnapshot[] {
  return Array.isArray(quota?.windows) ? quota.windows : []
}

function getQuotaWindowRemainingPercent(
  window: QuotaWindowSnapshot | null | undefined,
): number | null {
  if (!window) return null
  if (typeof window.remaining_ratio === 'number') {
    return clampPercent(window.remaining_ratio * 100)
  }
  if (typeof window.used_ratio === 'number') {
    return clampPercent((1 - window.used_ratio) * 100)
  }
  if (typeof window.limit_value === 'number' && window.limit_value > 0) {
    if (typeof window.remaining_value === 'number') {
      return clampPercent((window.remaining_value / window.limit_value) * 100)
    }
    if (typeof window.used_value === 'number') {
      return clampPercent((1 - (window.used_value / window.limit_value)) * 100)
    }
  }
  return null
}

function getQuotaWindow(
  quota: QuotaStatusSnapshot | null | undefined,
  code: string,
): QuotaWindowSnapshot | null {
  const normalizedCode = code.trim().toLowerCase()
  return getQuotaWindows(quota).find(window => normalizeText(window.code)?.toLowerCase() === normalizedCode) ?? null
}

function getQuotaWindowsByScope(
  quota: QuotaStatusSnapshot | null | undefined,
  scope: string,
): QuotaWindowSnapshot[] {
  const normalizedScope = scope.trim().toLowerCase()
  return getQuotaWindows(quota).filter(window => normalizeText(window.scope)?.toLowerCase() === normalizedScope)
}

function formatQuotaValue(value: number | null | undefined): string {
  const normalized = Number(value)
  if (!Number.isFinite(normalized)) return '0'
  const rounded = Math.round(normalized)
  if (Math.abs(normalized - rounded) < 1e-6) {
    return String(rounded)
  }
  return normalized.toFixed(1)
}

function getQuotaWindowValueText(window: QuotaWindowSnapshot | null | undefined): string | null {
  if (!window || typeof window.limit_value !== 'number' || window.limit_value <= 0) return null
  if (typeof window.remaining_value === 'number') {
    return `${formatQuotaValue(window.remaining_value)}/${formatQuotaValue(window.limit_value)}`
  }
  if (typeof window.used_value === 'number') {
    return `${formatQuotaValue(Math.max(window.limit_value - window.used_value, 0))}/${formatQuotaValue(window.limit_value)}`
  }
  return null
}

const GROK_QUOTA_MODE_LABELS: Record<string, string> = {
  quota_auto: 'Auto',
  auto: 'Auto',
  quota_fast: 'Fast',
  fast: 'Fast',
  quota_expert: 'Expert',
  expert: 'Expert',
  quota_heavy: 'Heavy',
  heavy: 'Heavy',
  quota_grok_4_3: 'Grok 4.3',
  'grok-420-computer-use-sa': 'Grok 4.3',
}

function getGrokQuotaWindowLabel(window: QuotaWindowSnapshot): string {
  const rawCode = normalizeText(window.code)?.replace(/^model:/i, '') || ''
  const rawLabel = normalizeText(window.label) || normalizeText(window.model) || rawCode
  const normalized = (rawLabel || rawCode).trim().toLowerCase()
  return GROK_QUOTA_MODE_LABELS[normalized] || GROK_QUOTA_MODE_LABELS[rawCode.toLowerCase()] || rawLabel || rawCode || t('mode')
}

function getCodexQuotaWindowLabel(window: QuotaWindowSnapshot): string {
  const label = normalizeText(window.label)
  if (label) {
    return label
  }
  const seconds = typeof window.window_seconds === 'number' ? window.window_seconds : null
  if (seconds && seconds > 0) return formatCodexWindowSeconds(seconds)
  const minutes = typeof window.window_minutes === 'number' ? window.window_minutes : null
  if (minutes === 300) return '5H'
  if (minutes === 10_080) return '7D'
  if (minutes === 43_200) return '1M'
  if (minutes) return formatCodexWindowMinutes(minutes)
  return normalizeText(window.code) || t('window')
}

function formatCodexWindowSeconds(totalSeconds: number): string {
  return formatCodexWindowMinutes(Math.max(Math.ceil(totalSeconds / 60), 1))
}

function formatCodexWindowMinutes(totalMinutes: number): string {
  const normalizedMinutes = Math.max(Math.ceil(totalMinutes), 1)
  const days = Math.floor(normalizedMinutes / (24 * 60))
  const hours = Math.floor((normalizedMinutes % (24 * 60)) / 60)
  const minutes = normalizedMinutes % 60
  const parts: string[] = []
  if (days > 0) parts.push(t('durationDays', { value: days }))
  if (hours > 0) parts.push(t('durationHours', { value: hours }))
  if (minutes > 0 || parts.length === 0) {
    parts.push(t('durationMinutes', { value: minutes }))
  }
  return parts.join('')
}

function getCodexQuotaText(quota: QuotaStatusSnapshot): string | null {
  const parts = getQuotaWindows(quota)
    .map((window) => {
      const remainingPercent = getQuotaWindowRemainingPercent(window)
      if (remainingPercent == null) return null
      return `${getCodexQuotaWindowLabel(window)}${t('remaining', { value: formatPercent(remainingPercent) })}`
    })
    .filter((value): value is string => value != null)
  if (parts.length > 0) return parts.join(' | ')

  if (quota.credits?.has_credits === true && typeof quota.credits.balance === 'number') {
    return t('credits', { value: quota.credits.balance.toFixed(2) })
  }
  if (quota.credits?.has_credits === true) return t('hasCredits')
  if (quota.credits?.has_credits === false) return t('noCredits')

  return normalizeText(quota.label) || t('noData')
}

function getKiroQuotaText(quota: QuotaStatusSnapshot): string | null {
  const code = normalizeText(quota.code)?.toLowerCase()
  if (code === 'banned') {
    return normalizeText(quota.label) || t('accountBanned')
  }

  const window = getQuotaWindow(quota, 'usage') ?? getQuotaWindowsByScope(quota, 'account')[0] ?? null
  const remainingPercent = getQuotaWindowRemainingPercent(window)
  if (typeof window?.remaining_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0 && window.remaining_value <= 0) {
    return t('remaining', { value: `${formatQuotaValue(window.remaining_value)}/${formatQuotaValue(window.limit_value)}` })
  }
  if (remainingPercent != null) {
    if (typeof window?.used_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0) {
      return t('remaining', { value: `${formatPercent(remainingPercent)} (${formatQuotaValue(window.used_value)}/${formatQuotaValue(window.limit_value)})` })
    }
    return t('remaining', { value: formatPercent(remainingPercent) })
  }

  if (typeof window?.remaining_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0) {
    return t('remaining', { value: `${formatQuotaValue(window.remaining_value)}/${formatQuotaValue(window.limit_value)}` })
  }

  return normalizeText(quota.label)
}

function getGrokQuotaText(quota: QuotaStatusSnapshot): string | null {
  const code = normalizeText(quota.code)?.toLowerCase()
  if (code === 'banned') {
    return normalizeText(quota.label) || t('accountBanned')
  }
  if (code === 'forbidden') {
    return normalizeText(quota.label) || t('accessRestricted')
  }

  const modelWindows = getQuotaWindowsByScope(quota, 'model')
  const modelParts = modelWindows
    .map((window) => {
      const remainingPercent = getQuotaWindowRemainingPercent(window)
      if (remainingPercent == null) return null
      const valueText = getQuotaWindowValueText(window)
      return `${getGrokQuotaWindowLabel(window)}${t('remaining', { value: `${formatPercent(remainingPercent)}${valueText ? ` (${valueText})` : ''}` })}`
    })
    .filter((value): value is string => value != null)

  if (modelParts.length > 0) return modelParts.join(' | ')

  const window = getQuotaWindow(quota, 'usage') ?? getQuotaWindowsByScope(quota, 'account')[0] ?? null
  const remainingPercent = getQuotaWindowRemainingPercent(window)
  if (typeof window?.remaining_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0 && window.remaining_value <= 0) {
    return t('remaining', { value: `${formatQuotaValue(window.remaining_value)}/${formatQuotaValue(window.limit_value)}` })
  }
  if (remainingPercent != null) {
    const valueText = getQuotaWindowValueText(window)
    if (valueText) {
      return t('remaining', { value: `${formatPercent(remainingPercent)} (${valueText})` })
    }
    return t('remaining', { value: formatPercent(remainingPercent) })
  }

  if (typeof window?.remaining_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0) {
    return t('remaining', { value: `${formatQuotaValue(window.remaining_value)}/${formatQuotaValue(window.limit_value)}` })
  }

  return normalizeText(quota.label)
}

function getAntigravityQuotaText(quota: QuotaStatusSnapshot): string | null {
  const code = normalizeText(quota.code)?.toLowerCase()
  if (code === 'forbidden') {
    return normalizeText(quota.label) || t('accessRestricted')
  }

  const remainingList = getQuotaWindowsByScope(quota, 'model')
    .map(getQuotaWindowRemainingPercent)
    .filter((value): value is number => value != null)

  if (remainingList.length === 0) return normalizeText(quota.label)

  const minimumRemaining = Math.min(...remainingList)
  if (remainingList.length === 1) {
    return t('remaining', { value: formatPercent(minimumRemaining) })
  }
  return t('minimumRemaining', { value: formatPercent(minimumRemaining), count: remainingList.length })
}

function getGeminiCliQuotaText(quota: QuotaStatusSnapshot): string | null {
  const modelWindows = getQuotaWindowsByScope(quota, 'model')
  const activeCoolingModels = modelWindows
    .filter((window) => {
      if (window.is_exhausted === true) return true
      if (typeof window.used_ratio === 'number') return window.used_ratio >= 1.0 - 1e-6
      return false
    })
    .filter((window) => {
      if (typeof window.reset_at !== 'number') return true
      return window.reset_at > Math.floor(Date.now() / 1000)
    })
    .map((window) => normalizeText(window.label) || normalizeText(window.model) || t('model'))

  if (activeCoolingModels.length === 1) {
    return t('cooling', { value: activeCoolingModels[0] })
  }
  if (activeCoolingModels.length > 1) {
    return t('coolingCount', { count: activeCoolingModels.length })
  }

  const remainingList = modelWindows
    .map(getQuotaWindowRemainingPercent)
    .filter((value): value is number => value != null)
  if (remainingList.length === 0) return normalizeText(quota.label)

  const minimumRemaining = Math.min(...remainingList)
  if (remainingList.length === 1) {
    return t('remaining', { value: formatPercent(minimumRemaining) })
  }
  return t('minimumRemaining', { value: formatPercent(minimumRemaining), count: remainingList.length })
}

function getChatGPTWebQuotaText(quota: QuotaStatusSnapshot): string | null {
  const window = getQuotaWindow(quota, 'image_gen') ?? getQuotaWindowsByScope(quota, 'account')[0] ?? null
  if (!window) return normalizeText(quota.label)

  const remainingPercent = getQuotaWindowRemainingPercent(window)
  if (typeof window.remaining_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0 && window.remaining_value <= 0) {
    return t('imageRemaining', { value: `${formatQuotaValue(window.remaining_value)}/${formatQuotaValue(window.limit_value)}` })
  }
  if (remainingPercent != null) {
    if (typeof window.used_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0) {
      return t('imageRemaining', { value: `${formatPercent(remainingPercent)} (${formatQuotaValue(window.used_value)}/${formatQuotaValue(window.limit_value)})` })
    }
    return t('imageRemaining', { value: formatPercent(remainingPercent) })
  }

  if (typeof window.remaining_value === 'number' && typeof window.limit_value === 'number' && window.limit_value > 0) {
    return t('imageRemaining', { value: `${formatQuotaValue(window.remaining_value)}/${formatQuotaValue(window.limit_value)}` })
  }
  if (typeof window.remaining_value === 'number') {
    return t('imageRemaining', { value: formatQuotaValue(window.remaining_value) })
  }

  return normalizeText(quota.label)
}

export function getLegacyAccountQuotaText(
  input: ProviderKeyQuotaCarrier,
): string | null {
  return normalizeText(input.account_quota)
}

export function getQuotaSnapshotFallbackText(
  input: ProviderKeyQuotaCarrier,
  fallbackProviderType?: string | null,
): string | null {
  const quota = getQuotaSnapshot(input)
  if (!quota) return null

  const providerType = getQuotaProviderType(quota, fallbackProviderType)
  switch (providerType) {
    case 'codex':
      return getCodexQuotaText(quota)
    case 'kiro':
      return getKiroQuotaText(quota)
    case 'grok':
      return getGrokQuotaText(quota)
    case 'antigravity':
      return getAntigravityQuotaText(quota)
    case 'gemini_cli':
      return getGeminiCliQuotaText(quota)
    case 'chatgpt_web':
      return getChatGPTWebQuotaText(quota)
    default:
      return normalizeText(quota.label)
  }
}

export function getQuotaDisplayText(
  input: ProviderKeyQuotaCarrier,
  fallbackProviderType?: string | null,
): string | null {
  return getQuotaSnapshotFallbackText(input, fallbackProviderType) || getLegacyAccountQuotaText(input)
}
