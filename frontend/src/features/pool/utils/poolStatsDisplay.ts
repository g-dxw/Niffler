import type { QuotaWindowUsageSnapshot } from '@/api/endpoints/types/statusSnapshot'
import type { PoolManagementStatsMode } from '@/features/pool/utils/poolManagementState'

export type PoolStatsMetricKey = 'request_count' | 'total_tokens' | 'total_cost_usd'
export type PoolStatsDisplayKind = 'account_total' | 'codex_cycle'
export type PoolCodexCycleWindowCode = string

export interface PoolStatsKeyInput {
  request_count?: number | null
  total_tokens?: number | null
  total_cost_usd?: number | string | null
  status_snapshot?: {
    quota?: {
      windows?: Array<{
        code?: string | null
        label?: string | null
        scope?: string | null
        window_seconds?: number | null
        window_minutes?: number | null
        usage?: QuotaWindowUsageSnapshot | null
      } | null> | null
    } | null
  } | null
}

type PoolStatsQuotaWindow = NonNullable<
  NonNullable<
    NonNullable<PoolStatsKeyInput['status_snapshot']>['quota']
  >['windows']
>[number]

export interface PoolStatsMetric {
  key: PoolStatsMetricKey
  label: string
  value: string
  missing: boolean
}

export interface PoolAccountTotalStatsDisplay {
  kind: 'account_total'
  metrics: PoolStatsMetric[]
}

export interface PoolCodexCycleStatsGroup {
  code: PoolCodexCycleWindowCode
  label: string
  metrics: PoolStatsMetric[]
}

export interface PoolCodexCycleStatsDisplay {
  kind: 'codex_cycle'
  groups: PoolCodexCycleStatsGroup[]
}

export type PoolStatsDisplay = PoolAccountTotalStatsDisplay | PoolCodexCycleStatsDisplay

const MISSING_STAT_VALUE = '统计中'

export function isCodexProviderType(providerType: string | null | undefined): boolean {
  return String(providerType || '').trim().toLowerCase() === 'codex'
}

export function hasPendingCodexCycleStats(
  key: PoolStatsKeyInput,
  providerType: string | null | undefined,
): boolean {
  if (!isCodexProviderType(providerType)) return false
  const windows = key.status_snapshot?.quota?.windows
  if (!Array.isArray(windows)) return false

  return windows
    .filter(window => window != null)
    .filter((window) => {
      const scope = String(window?.scope || 'account').trim().toLowerCase()
      return !['feature', 'model', 'workspace'].includes(scope)
    })
    .some(window => normalizeWindowCode(window?.code) !== '' && window?.usage == null)
}

export function formatPoolStatInteger(value: number | null | undefined): string {
  const n = Number(value ?? 0)
  if (!Number.isFinite(n) || n <= 0) return '0'
  return Math.round(n).toLocaleString('en-US')
}

export function formatPoolTokenCount(value: number | null | undefined): string {
  const n = Number(value ?? 0)
  if (!Number.isFinite(n) || n <= 0) return '0'
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return String(Math.round(n))
}

export function formatPoolStatUsd(value: number | string | null | undefined): string {
  const n = Number(value ?? 0)
  if (!Number.isFinite(n) || n <= 0) return '$0.00'
  if (n < 0.01) return `$${n.toFixed(4)}`
  if (n < 1) return `$${n.toFixed(3)}`
  if (n < 1000) return `$${n.toFixed(2)}`
  return `$${n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`
}

function formatCycleInteger(value: number | null | undefined): string | null {
  if (value == null) return null
  const n = Number(value)
  if (!Number.isFinite(n)) return null
  if (n <= 0) return '0'
  return Math.round(n).toLocaleString('en-US')
}

function formatCycleTokenCount(value: number | null | undefined): string | null {
  if (value == null) return null
  const n = Number(value)
  if (!Number.isFinite(n)) return null
  return formatPoolTokenCount(n)
}

function formatCycleUsd(value: number | string | null | undefined): string | null {
  if (value == null) return null
  const n = Number(value)
  if (!Number.isFinite(n)) return null
  if (n <= 0) return '0'
  return formatPoolStatUsd(value)
}

function createMetric(
  key: PoolStatsMetricKey,
  label: string,
  value: string | null,
): PoolStatsMetric {
  return {
    key,
    label,
    value: value ?? MISSING_STAT_VALUE,
    missing: value == null,
  }
}

function normalizeWindowCode(value: unknown): string {
  return String(value || '').trim().toLowerCase()
}

function formatWindowDuration(totalMinutes: number): string {
  if (!Number.isFinite(totalMinutes) || totalMinutes <= 0) return ''
  if (totalMinutes % (30 * 24 * 60) === 0) return `${totalMinutes / (30 * 24 * 60)}M`
  if (totalMinutes % (7 * 24 * 60) === 0) return `${totalMinutes / (7 * 24 * 60)}周`
  if (totalMinutes % (24 * 60) === 0) return `${totalMinutes / (24 * 60)}D`
  if (totalMinutes % 60 === 0) return `${totalMinutes / 60}H`
  return `${totalMinutes}分钟`
}

function quotaWindowLabel(window: NonNullable<PoolStatsQuotaWindow>): string {
  const explicitLabel = String(window?.label || '').trim()
  if (explicitLabel) return explicitLabel
  const seconds = Number(window?.window_seconds ?? 0)
  if (Number.isFinite(seconds) && seconds > 0) {
    return formatWindowDuration(Math.ceil(seconds / 60))
  }
  const minutes = Number(window?.window_minutes ?? 0)
  if (Number.isFinite(minutes) && minutes > 0) {
    return formatWindowDuration(minutes)
  }
  const code = normalizeWindowCode(window?.code)
  if (code === '5h') return '5H'
  if (code === 'weekly') return '周'
  if (code === '7d') return '7D'
  if (code === '1m' || code === 'monthly') return '月'
  return String(window?.code || '窗口').trim()
}

function buildAccountTotalMetrics(key: PoolStatsKeyInput): PoolStatsMetric[] {
  return [
    createMetric('request_count', '请求', formatPoolStatInteger(key.request_count)),
    createMetric('total_tokens', 'Token', formatPoolTokenCount(key.total_tokens)),
    createMetric('total_cost_usd', '基础费用', formatPoolStatUsd(key.total_cost_usd)),
  ]
}

function buildCycleMetrics(usage: QuotaWindowUsageSnapshot | null): PoolStatsMetric[] {
  return [
    createMetric('request_count', '请求', formatCycleInteger(usage?.request_count)),
    createMetric('total_tokens', 'Token', formatCycleTokenCount(usage?.total_tokens)),
    createMetric('total_cost_usd', '基础费用', formatCycleUsd(usage?.total_cost_usd)),
  ]
}

export function buildAccountTotalStatsDisplay(
  key: PoolStatsKeyInput,
): PoolAccountTotalStatsDisplay {
  return {
    kind: 'account_total',
    metrics: buildAccountTotalMetrics(key),
  }
}

export function buildCodexCycleStatsDisplay(
  key: PoolStatsKeyInput,
): PoolCodexCycleStatsDisplay {
  const windows = key.status_snapshot?.quota?.windows
  const groups = Array.isArray(windows)
    ? windows
        .filter(window => window != null)
        .filter((window) => {
          const scope = String(window?.scope || 'account').trim().toLowerCase()
          return !['feature', 'model', 'workspace'].includes(scope)
        })
        .filter(window => normalizeWindowCode(window?.code) !== '')
        .map(window => ({
          code: normalizeWindowCode(window?.code),
          label: quotaWindowLabel(window),
          metrics: buildCycleMetrics(window?.usage ?? null),
        }))
    : []

  return {
    kind: 'codex_cycle',
    groups,
  }
}

export function buildPoolStatsDisplay(
  key: PoolStatsKeyInput,
  providerType: string | null | undefined,
  mode: PoolManagementStatsMode,
): PoolStatsDisplay {
  if (isCodexProviderType(providerType) && mode === 'current_cycle') {
    return buildCodexCycleStatsDisplay(key)
  }

  return buildAccountTotalStatsDisplay(key)
}
