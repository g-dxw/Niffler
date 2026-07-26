import type { NifflerStabilityObservation } from '@/api/niffler-core'
import { i18n } from '@/i18n'

export const STABILITY_REQUIRED_PASS_DAYS = 14
export const STABILITY_OBSERVATION_FETCH_LIMIT = STABILITY_REQUIRED_PASS_DAYS + 1
export const STABILITY_WINDOW_MS = 24 * 60 * 60 * 1000

export interface NifflerStabilityGateState {
  ready: boolean
  consecutivePassDays: number
  description: string
  blockReason: string
}

export function sortStabilityObservations(
  observations: NifflerStabilityObservation[]
): NifflerStabilityObservation[] {
  return [...observations].sort(
    (left, right) => right.window_start_unix_ms - left.window_start_unix_ms
  )
}

export function getCompletedStabilityObservations(
  observations: NifflerStabilityObservation[],
  nowUnixMs = Date.now()
): NifflerStabilityObservation[] {
  return sortStabilityObservations(observations).filter((item) =>
    isCompletedStabilityObservation(item, nowUnixMs)
  )
}

export function getStabilityGateState(
  observations: NifflerStabilityObservation[],
  nowUnixMs = Date.now()
): NifflerStabilityGateState {
  const sortedObservations = sortStabilityObservations(observations)
  const latestObservation = sortedObservations[0] ?? null
  const completedObservations = getCompletedStabilityObservations(observations, nowUnixMs)
  const consecutivePassDays = getStabilityConsecutivePassDays(completedObservations, nowUnixMs)
  const blockReason = getStabilityGateBlockReason(
    latestObservation,
    completedObservations,
    consecutivePassDays,
    nowUnixMs
  )
  const ready = Boolean(latestObservation)
    && isFreshStabilityObservation(latestObservation, nowUnixMs)
    && latestStabilityObservationAllowsGate(latestObservation, nowUnixMs)
    && consecutivePassDays >= STABILITY_REQUIRED_PASS_DAYS

  return {
    ready,
    consecutivePassDays,
    blockReason,
    description: ready ? i18n.global.t('stabilityUi.ready') : blockReason
  }
}

export function getStabilityConsecutivePassDays(
  completedObservations: NifflerStabilityObservation[],
  nowUnixMs = Date.now()
): number {
  let count = 0
  let expectedWindowStart = currentUtcDayStartUnixMs(nowUnixMs) - STABILITY_WINDOW_MS
  for (const item of sortStabilityObservations(completedObservations)) {
    if (item.window_start_unix_ms !== expectedWindowStart) {
      break
    }
    if (!isPassingStabilityObservation(item)) {
      break
    }
    count += 1
    expectedWindowStart = item.window_start_unix_ms - STABILITY_WINDOW_MS
  }
  return count
}

export function getStabilityGateBlockReason(
  latestObservation: NifflerStabilityObservation | null,
  completedObservations: NifflerStabilityObservation[],
  consecutivePassDays: number,
  nowUnixMs = Date.now()
): string {
  if (!latestObservation) {
    return i18n.global.t('stabilityUi.noRecords')
  }
  if (latestObservation.window_start_unix_ms > currentUtcDayStartUnixMs(nowUnixMs)) {
    return i18n.global.t('stabilityUi.timeInvalid')
  }
  if (!isFreshStabilityObservation(latestObservation, nowUnixMs)) {
    return i18n.global.t('stabilityUi.stale')
  }
  if (!isCompletedStabilityObservation(latestObservation, nowUnixMs)
    && !isPassingStabilityObservation(latestObservation)) {
    return i18n.global.t('stabilityUi.notPassed')
  }

  let expectedWindowStart = currentUtcDayStartUnixMs(nowUnixMs) - STABILITY_WINDOW_MS
  for (let index = 0; index < completedObservations.length; index += 1) {
    const item = completedObservations[index]
    if (item.window_start_unix_ms !== expectedWindowStart) {
      return i18n.global.t('stabilityUi.notContinuous')
    }
    if (!isPassingStabilityObservation(item)) {
      return index === 0
        ? i18n.global.t('stabilityUi.latestFailed')
        : i18n.global.t('stabilityUi.someFailed')
    }
    expectedWindowStart = item.window_start_unix_ms - STABILITY_WINDOW_MS
  }

  if (consecutivePassDays < STABILITY_REQUIRED_PASS_DAYS) {
    return i18n.global.t('stabilityUi.missing', { count: STABILITY_REQUIRED_PASS_DAYS - consecutivePassDays })
  }
  return i18n.global.t('stabilityUi.needDays')
}

export function isPassingStabilityObservation(item: NifflerStabilityObservation): boolean {
  return item.status === 'pass' && item.blocker_codes.length === 0
}

export function isCompletedStabilityObservation(
  item: NifflerStabilityObservation,
  nowUnixMs = Date.now()
): boolean {
  return item.window_end_unix_ms <= nowUnixMs
}

export function currentUtcDayStartUnixMs(nowUnixMs = Date.now()): number {
  return Math.floor(nowUnixMs / STABILITY_WINDOW_MS) * STABILITY_WINDOW_MS
}

export function isFreshStabilityObservation(
  item: NifflerStabilityObservation,
  nowUnixMs = Date.now()
): boolean {
  const currentDayStart = currentUtcDayStartUnixMs(nowUnixMs)
  return item.window_start_unix_ms <= currentDayStart
    && item.window_end_unix_ms >= currentDayStart
}

export function latestStabilityObservationAllowsGate(
  item: NifflerStabilityObservation,
  nowUnixMs = Date.now()
): boolean {
  return isCompletedStabilityObservation(item, nowUnixMs) || isPassingStabilityObservation(item)
}
