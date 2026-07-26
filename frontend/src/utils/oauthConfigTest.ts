import type { OAuthProviderTestResponse } from '@/api/oauth'

export type OAuthConfigTestSeverity = 'success' | 'warning' | 'error'

export interface OAuthConfigTestSummary {
  severity: OAuthConfigTestSeverity
  message: string
  failures: string[]
  warnings: string[]
}

function describeSecretStatus(status: string | undefined): string | null {
  const normalized = (status || '').trim().toLowerCase()
  if (!normalized || normalized === 'likely_valid' || normalized === 'configured') return null
  if (normalized === 'invalid') return i18n.global.t('oauthTestUi.secretInvalid')
  if (normalized === 'unsupported') return i18n.global.t('oauthTestUi.secretUnsupported')
  if (normalized === 'not_provided') return i18n.global.t('oauthTestUi.secretMissing')
  if (normalized === 'unknown') return i18n.global.t('oauthTestUi.secretUnknown')
  return `Secret: ${status}`
}

export function summarizeOAuthConfigTest(result: OAuthProviderTestResponse): OAuthConfigTestSummary {
  const failures: string[] = []
  const warnings: string[] = []

  if (!result.authorization_url_reachable) {
    failures.push(i18n.global.t('oauthTestUi.authorizationUnreachable'))
  }
  if (!result.token_url_reachable) {
    failures.push(i18n.global.t('oauthTestUi.tokenUnreachable'))
  }

  const secretStatus = (result.secret_status || '').trim().toLowerCase()
  const secretMessage = describeSecretStatus(result.secret_status)
  if (secretMessage && (secretStatus === 'invalid' || secretStatus === 'unsupported')) {
    failures.push(secretMessage)
  } else if (secretMessage) {
    warnings.push(secretMessage)
  }

  if (failures.length > 0) {
    return {
      severity: 'error',
      message: i18n.global.t('oauthTestUi.failed', { items: failures.join(i18n.global.locale.value === 'en-US' ? ', ' : '，') }),
      failures,
      warnings,
    }
  }

  if (warnings.length > 0) {
    return {
      severity: 'warning',
      message: i18n.global.t('oauthTestUi.completedWarnings', { items: warnings.join(i18n.global.locale.value === 'en-US' ? ', ' : '，') }),
      failures,
      warnings,
    }
  }

  return {
    severity: 'success',
    message: i18n.global.t('oauthTestUi.passed'),
    failures,
    warnings,
  }
}
import { i18n } from '@/i18n'
