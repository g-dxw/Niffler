import {
  getAccountStatusDisplay,
  type ProviderKeyStatusCarrier,
} from './providerKeyStatus'
import { i18n } from '@/i18n'

export interface OAuthAccountBlockDisplay {
  label: string | null
  reason: string | null
}

export interface OAuthRefreshFeedbackInput {
  accountStateRecheckAttempted?: boolean | null
  accountStateRecheckError?: string | null
  snapshot?: ProviderKeyStatusCarrier | null
}

function normalizeText(value: unknown): string | null {
  if (typeof value !== 'string') return null
  const text = value.trim()
  return text || null
}

function formatRecheckError(value: string): string {
  const collapsed = value.replace(/\s+/g, ' ').trim()
  const maxLength = 180
  if (collapsed.length <= maxLength) return collapsed
  return `${collapsed.slice(0, maxLength - 3).trimEnd()}...`
}

export function resolveOAuthAccountBlockDisplay(
  snapshot: ProviderKeyStatusCarrier,
): OAuthAccountBlockDisplay {
  const account = getAccountStatusDisplay(snapshot)
  if (!account.blocked || !account.label) {
    return { label: null, reason: null }
  }
  return {
    label: account.label,
    reason: account.reason,
  }
}

export function getOAuthRefreshFeedback(
  input: OAuthRefreshFeedbackInput,
): { tone: 'success' | 'warning'; message: string } {
  const blockedLabel = normalizeText(
    input.snapshot ? getAccountStatusDisplay(input.snapshot).label : null,
  )
  const recheckError = normalizeText(input.accountStateRecheckError)

  if (input.accountStateRecheckAttempted) {
    if (recheckError) {
      return {
        tone: 'warning',
        message: i18n.global.t('oauthFeedbackUi.successRecheckFailed', { error: formatRecheckError(recheckError) }),
      }
    }
    if (blockedLabel) {
      return {
        tone: 'warning',
        message: i18n.global.t('oauthFeedbackUi.successStillBlocked', { label: blockedLabel }),
      }
    }
    return {
      tone: 'success',
      message: i18n.global.t('oauthFeedbackUi.successRechecked'),
    }
  }

  if (blockedLabel) {
    return {
      tone: 'warning',
      message: i18n.global.t('oauthFeedbackUi.successBlocked', { label: blockedLabel }),
    }
  }

  return {
    tone: 'success',
    message: i18n.global.t('oauthFeedbackUi.success'),
  }
}
