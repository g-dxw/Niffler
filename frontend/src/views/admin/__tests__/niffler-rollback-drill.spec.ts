import { describe, expect, it } from 'vitest'

import type { NifflerRollbackDrillEvidencePayload } from '@/api/niffler-core'
import {
  getRollbackEvidenceHint,
  getRollbackEvidenceMissingLabels,
  normalizeRollbackEvidenceStatus,
} from '../niffler-rollback-drill'

function payload(
  overrides: Partial<NifflerRollbackDrillEvidencePayload> = {}
): NifflerRollbackDrillEvidencePayload {
  return {
    status: 'not_recorded',
    evidence_complete: false,
    status_config_key: 'niffler_stability_rollback_drill_status',
    evidence_config_key: 'niffler_stability_rollback_drill_evidence',
    evidence: {
      status: 'not_recorded',
      backup_reference: null,
      rollback_image_tag: null,
      drill_summary: null,
      recorded_at_unix_ms: null,
      recorded_by: null
    },
    ...overrides
  }
}

describe('niffler rollback drill helpers', () => {
  it('treats unknown status as not_recorded', () => {
    expect(
      normalizeRollbackEvidenceStatus(
        payload({
          status: 'weird',
          evidence: { status: 'oops' }
        })
      )
    ).toBe('not_recorded')
  })

  it('lists all missing fields when status is passed but evidence is incomplete', () => {
    expect(
      getRollbackEvidenceMissingLabels(
        payload({
          status: 'passed',
          evidence: {
            status: 'passed',
            backup_reference: '',
            rollback_image_tag: null,
            drill_summary: '  '
          }
        })
      )
    ).toEqual(['备份引用', '可回滚镜像标签', '演练说明'])
  })

  it('explains rollback_drill_evidence_missing with concrete missing fields', () => {
    const hint = getRollbackEvidenceHint(
      payload({
        status: 'passed',
        evidence: {
          status: 'passed',
          backup_reference: 'backup-2026-06-09',
          rollback_image_tag: '',
          drill_summary: null
        }
      }),
      ['rollback_drill_evidence_missing']
    )

    expect(hint.tone).toBe('warning')
    expect(hint.title).toBe('回滚演练已标记为通过，但证据不完整')
    expect(hint.description).toContain('可回滚镜像标签')
    expect(hint.description).toContain('演练说明')
  })

  it('shows success hint when evidence is complete', () => {
    const hint = getRollbackEvidenceHint(
      payload({
        status: 'passed',
        evidence_complete: true,
        evidence: {
          status: 'passed',
          backup_reference: 'backup-2026-06-09',
          rollback_image_tag: 'niffler-app:31836c54',
          drill_summary: 'rollback ok'
        }
      })
    )

    expect(hint.tone).toBe('success')
    expect(hint.title).toBe('当前回滚演练证据完整')
  })

  it('shows failed hint when rollback drill failed', () => {
    const hint = getRollbackEvidenceHint(
      payload({
        status: 'failed',
        evidence: {
          status: 'failed',
          drill_summary: 'network timeout'
        }
      })
    )

    expect(hint.tone).toBe('danger')
    expect(hint.title).toBe('最近一次回滚演练失败')
  })
})
