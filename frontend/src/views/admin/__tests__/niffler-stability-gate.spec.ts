import { describe, expect, it } from 'vitest'

import type { NifflerStabilityObservation } from '@/api/niffler-core'
import {
  STABILITY_REQUIRED_PASS_DAYS,
  STABILITY_WINDOW_MS,
  getStabilityGateState,
} from '../niffler-stability-gate'

const NOW = Date.UTC(2026, 5, 9, 12, 0, 0)
const TODAY_START = Date.UTC(2026, 5, 9, 0, 0, 0)

function observation(
  dayOffset: number,
  status = 'pass',
  blockerCodes: string[] = []
): NifflerStabilityObservation {
  const start = TODAY_START - dayOffset * STABILITY_WINDOW_MS
  return {
    id: `obs-${dayOffset}`,
    window_start_unix_ms: start,
    window_end_unix_ms: start + STABILITY_WINDOW_MS,
    status,
    rollback_drill_status: 'passed',
    consistency_checked_count: 0,
    consistency_issue_count: 0,
    unknown_upstream_count: 0,
    legacy_write_call_count: 0,
    billing_reservation_exception_count: 0,
    referral_exception_count: 0,
    blocker_codes: blockerCodes,
    summary: null,
    created_at_unix_ms: start,
    updated_at_unix_ms: start,
  }
}

function completedPassingDays(count: number): NifflerStabilityObservation[] {
  return Array.from({ length: count }, (_, index) => observation(index + 1))
}

describe('niffler stability gate', () => {
  it('requires 14 completed passing windows before allowing legacy removal', () => {
    const gate = getStabilityGateState(completedPassingDays(13), NOW)

    expect(gate.ready).toBe(false)
    expect(gate.consecutivePassDays).toBe(13)
    expect(gate.description).toBe('还缺 1 个已结束观察窗口')
  })

  it('allows the next slice only after 14 consecutive completed passing windows', () => {
    const gate = getStabilityGateState(completedPassingDays(STABILITY_REQUIRED_PASS_DAYS), NOW)

    expect(gate.ready).toBe(true)
    expect(gate.consecutivePassDays).toBe(14)
    expect(gate.description).toBe('第 5 批第六片可以开始')
  })

  it('blocks when the current unfinished window already has a blocker', () => {
    const gate = getStabilityGateState([
      observation(0, 'pending', ['rollback_drill_not_recorded']),
      ...completedPassingDays(STABILITY_REQUIRED_PASS_DAYS),
    ], NOW)

    expect(gate.ready).toBe(false)
    expect(gate.consecutivePassDays).toBe(14)
    expect(gate.description).toBe('当前观察窗口未通过，不能开始第六片')
  })

  it('blocks when the latest observation is no longer fresh', () => {
    const gate = getStabilityGateState([
      observation(2),
      ...completedPassingDays(12).map((item) => ({
        ...item,
        window_start_unix_ms: item.window_start_unix_ms - STABILITY_WINDOW_MS,
        window_end_unix_ms: item.window_end_unix_ms - STABILITY_WINDOW_MS,
      })),
    ], NOW)

    expect(gate.ready).toBe(false)
    expect(gate.description).toBe('稳定观察任务超过 1 天没有更新，不能开始第六片')
  })

  it('sorts observations before checking the latest window and continuity', () => {
    const observations = [
      observation(3),
      observation(1),
      observation(2),
      ...completedPassingDays(STABILITY_REQUIRED_PASS_DAYS).slice(3),
    ]
    const gate = getStabilityGateState(observations, NOW)

    expect(gate.ready).toBe(true)
    expect(gate.consecutivePassDays).toBe(14)
  })
})
