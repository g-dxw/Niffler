import type { CheckUpdateResponse } from '@/api/admin'

export function describeUpdateStatus(status: CheckUpdateResponse | null): string {
  if (!status) return i18n.global.t('updateUi.checking')
  if (status.has_update) return i18n.global.t('updateUi.available')
  if (status.error) return i18n.global.t('updateUi.failed')
  return i18n.global.t('updateUi.latest')
}

export function buildUpdateErrorStatus(
  previousStatus: CheckUpdateResponse | null,
  error: unknown
): CheckUpdateResponse {
  return {
    current_version: previousStatus?.current_version || '',
    latest_version: null,
    has_update: false,
    release_url: null,
    release_notes: null,
    published_at: null,
    error: error instanceof Error ? error.message : i18n.global.t('updateUi.requestFailed')
  }
}
import { i18n } from '@/i18n'
