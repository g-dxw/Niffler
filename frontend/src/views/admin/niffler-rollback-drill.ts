import type {
  NifflerRollbackDrillEvidencePayload,
  NifflerRollbackDrillStatus,
} from '@/api/niffler-core'

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

const ROLLBACK_EVIDENCE_FIELDS: Array<{
  key: RollbackEvidenceRequirement['key']
  label: RollbackEvidenceRequirement['label']
}> = [
  { key: 'backup_reference', label: '备份引用' },
  { key: 'rollback_image_tag', label: '可回滚镜像标签' },
  { key: 'drill_summary', label: '演练说明' }
]

function hasText(value?: string | null): boolean {
  return typeof value === 'string' && value.trim().length > 0
}

export function normalizeRollbackEvidenceStatus(
  payload: NifflerRollbackDrillEvidencePayload | null
): NifflerRollbackDrillStatus {
  const status = payload?.evidence?.status || payload?.status
  return status === 'passed' || status === 'failed' || status === 'not_recorded'
    ? status
    : 'not_recorded'
}

export function getRollbackEvidenceRequirements(
  payload: NifflerRollbackDrillEvidencePayload | null
): RollbackEvidenceRequirement[] {
  const evidence = payload?.evidence
  const requiresProof = normalizeRollbackEvidenceStatus(payload) === 'passed'

  return ROLLBACK_EVIDENCE_FIELDS.map((field) => ({
    ...field,
    missing: requiresProof && !hasText(evidence?.[field.key])
  }))
}

export function getRollbackEvidenceMissingLabels(
  payload: NifflerRollbackDrillEvidencePayload | null
): string[] {
  return getRollbackEvidenceRequirements(payload)
    .filter((field) => field.missing)
    .map((field) => field.label)
}

export function getRollbackEvidenceHint(
  payload: NifflerRollbackDrillEvidencePayload | null,
  blockerCodes: string[] = []
): RollbackEvidenceHint {
  const status = normalizeRollbackEvidenceStatus(payload)
  const missingLabels = getRollbackEvidenceMissingLabels(payload)
  const missingText = missingLabels.join('、')

  if (blockerCodes.includes('rollback_drill_failed') || status === 'failed') {
    return {
      tone: 'danger',
      title: '最近一次回滚演练失败',
      description: '先修复演练问题，再重新完成一次演练并记录新的证据。'
    }
  }

  if (blockerCodes.includes('rollback_drill_evidence_missing')) {
    return {
      tone: 'warning',
      title: '回滚演练已标记为通过，但证据不完整',
      description: missingText
        ? `还缺：${missingText}。补齐后重新保存，稳定观察才会计入通过。`
        : '请补齐备份引用、可回滚镜像标签和演练说明，再重新保存。'
    }
  }

  if (blockerCodes.includes('rollback_drill_not_recorded') || status === 'not_recorded') {
    return {
      tone: 'warning',
      title: '当前还没有回滚演练记录',
      description: '完成一次可验证的回滚演练后，把备份引用、可回滚镜像标签和演练说明填在下面，再保存为已通过。'
    }
  }

  if (status === 'passed' && missingLabels.length > 0) {
    return {
      tone: 'warning',
      title: '回滚演练证据还不完整',
      description: `还缺：${missingText}。只有三项都存在，稳定观察才会计入通过。`
    }
  }

  if (payload?.evidence_complete) {
    return {
      tone: 'success',
      title: '当前回滚演练证据完整',
      description: '如果其他稳定观察指标也通过，这一项不会再阻塞 14 天稳定期。'
    }
  }

  return {
    tone: 'warning',
    title: '请确认回滚演练证据',
    description: '如果本次演练已经完成，请补齐备份引用、可回滚镜像标签和演练说明。'
  }
}
