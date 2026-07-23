import type {
  NifflerRollbackDrillEvidencePayload,
  NifflerRollbackDrillStatus,
} from '@/api/niffler-core'
import { i18n } from '@/i18n'

export interface RollbackEvidenceRequirement {
  key: 'backup_reference' | 'rollback_image_tag' | 'drill_summary'
  label: string
  missing: boolean
}

export interface RollbackEvidenceHint {
  tone: 'success' | 'warning' | 'danger'
  title: string
  description: string
}

const ROLLBACK_EVIDENCE_FIELDS: Array<{ key: RollbackEvidenceRequirement['key']; label: string }> = [
  { key: 'backup_reference', label: i18n.global.t('rollbackUi.backup') },
  { key: 'rollback_image_tag', label: i18n.global.t('rollbackUi.imageTag') },
  { key: 'drill_summary', label: i18n.global.t('rollbackUi.summary') },
]

function hasText(value?: string | null): boolean {
  return typeof value === 'string' && value.trim().length > 0
}

export function normalizeRollbackEvidenceStatus(payload: NifflerRollbackDrillEvidencePayload | null): NifflerRollbackDrillStatus {
  const status = payload?.evidence?.status || payload?.status
  return status === 'passed' || status === 'failed' || status === 'not_recorded' ? status : 'not_recorded'
}

export function getRollbackEvidenceRequirements(payload: NifflerRollbackDrillEvidencePayload | null): RollbackEvidenceRequirement[] {
  const evidence = payload?.evidence
  const requiresProof = normalizeRollbackEvidenceStatus(payload) === 'passed'
  return ROLLBACK_EVIDENCE_FIELDS.map(field => ({ ...field, missing: requiresProof && !hasText(evidence?.[field.key]) }))
}

export function getRollbackEvidenceMissingLabels(payload: NifflerRollbackDrillEvidencePayload | null): string[] {
  return getRollbackEvidenceRequirements(payload).filter(field => field.missing).map(field => field.label)
}

export function getRollbackEvidenceHint(payload: NifflerRollbackDrillEvidencePayload | null, blockerCodes: string[] = []): RollbackEvidenceHint {
  const status = normalizeRollbackEvidenceStatus(payload)
  const missingLabels = getRollbackEvidenceMissingLabels(payload)
  const missingText = missingLabels.join(i18n.global.locale.value === 'en-US' ? ', ' : '、')

  if (blockerCodes.includes('rollback_drill_failed') || status === 'failed') {
    return { tone: 'danger', title: i18n.global.t('rollbackUi.latestFailed'), description: i18n.global.t('rollbackUi.latestFailedDesc') }
  }
  if (blockerCodes.includes('rollback_drill_evidence_missing')) {
    return {
      tone: 'warning',
      title: i18n.global.t('rollbackUi.incompletePassed'),
      description: missingText ? i18n.global.t('rollbackUi.missing', { value: missingText }) : i18n.global.t('rollbackUi.fillAll'),
    }
  }
  if (blockerCodes.includes('rollback_drill_not_recorded') || status === 'not_recorded') {
    return { tone: 'warning', title: i18n.global.t('rollbackUi.noRecord'), description: i18n.global.t('rollbackUi.noRecordDesc') }
  }
  if (status === 'passed' && missingLabels.length > 0) {
    return { tone: 'warning', title: i18n.global.t('rollbackUi.incomplete'), description: i18n.global.t('rollbackUi.incompleteDesc', { value: missingText }) }
  }
  if (payload?.evidence_complete) {
    return { tone: 'success', title: i18n.global.t('rollbackUi.complete'), description: i18n.global.t('rollbackUi.completeDesc') }
  }
  return { tone: 'warning', title: i18n.global.t('rollbackUi.confirm'), description: i18n.global.t('rollbackUi.confirmDesc') }
}
