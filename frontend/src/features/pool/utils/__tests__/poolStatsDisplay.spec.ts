import { describe, expect, it } from 'vitest'

import {
  buildPoolStatsDisplay,
  hasPendingCodexCycleStats,
  type PoolStatsKeyInput,
} from '@/features/pool/utils/poolStatsDisplay'

function metricValues(metrics: Array<{ key: string, value: string }>) {
  return Object.fromEntries(metrics.map(metric => [metric.key, metric.value]))
}

function createCodexKey(overrides: Partial<PoolStatsKeyInput> = {}): PoolStatsKeyInput {
  return {
    request_count: 1234,
    total_tokens: 5678000,
    total_cost_usd: '12.3456',
    status_snapshot: {
      quota: {
        windows: [
          {
            code: '5h',
            usage: {
              request_count: 5,
              total_tokens: 2500,
              total_cost_usd: '0.0045',
            },
          },
          {
            code: 'weekly',
            usage: {
              request_count: 0,
              total_tokens: 0,
              total_cost_usd: '0.00000000',
            },
          },
        ],
      },
    },
    ...overrides,
  }
}

describe('poolStatsDisplay', () => {
  it('builds Codex current-cycle groups in 5H and weekly order', () => {
    const display = buildPoolStatsDisplay(createCodexKey(), 'codex', 'current_cycle')

    expect(display.kind).toBe('codex_cycle')
    if (display.kind !== 'codex_cycle') throw new Error('expected codex cycle display')

    expect(display.groups.map(group => group.label)).toEqual(['5H', '周'])
    expect(metricValues(display.groups[0].metrics)).toEqual({
      request_count: '5',
      total_tokens: '2.5K',
      total_cost_usd: '$0.0045',
    })
    expect(metricValues(display.groups[1].metrics)).toEqual({
      request_count: '0',
      total_tokens: '0',
      total_cost_usd: '0',
    })
  })

  it('renders missing cycle usage as pending without inventing absent windows', () => {
    const display = buildPoolStatsDisplay(
      createCodexKey({
        status_snapshot: {
          quota: {
            windows: [{ code: '5h', usage: null }],
          },
        },
      }),
      'codex',
      'current_cycle',
    )

    expect(display.kind).toBe('codex_cycle')
    if (display.kind !== 'codex_cycle') throw new Error('expected codex cycle display')

    expect(metricValues(display.groups[0].metrics)).toEqual({
      request_count: '统计中',
      total_tokens: '统计中',
      total_cost_usd: '统计中',
    })
    expect(display.groups.map(group => group.label)).toEqual(['5H'])
  })

  it('detects only missing account-window stats as pending', () => {
    expect(hasPendingCodexCycleStats(
      createCodexKey({
        status_snapshot: {
          quota: {
            windows: [
              { code: 'weekly', scope: 'account', usage: null },
              { code: 'weekly', scope: 'feature', usage: null },
            ],
          },
        },
      }),
      'codex',
    )).toBe(true)

    expect(hasPendingCodexCycleStats(
      createCodexKey({
        status_snapshot: {
          quota: {
            windows: [
              { code: 'weekly', scope: 'feature', usage: null },
              {
                code: '5h',
                scope: 'account',
                usage: { request_count: 0, total_tokens: 0, total_cost_usd: 0 },
              },
            ],
          },
        },
      }),
      'codex',
    )).toBe(false)
  })

  it('renders only the weekly window when upstream exposes only weekly quota', () => {
    const display = buildPoolStatsDisplay(
      createCodexKey({
        status_snapshot: {
          quota: {
            windows: [{
              code: 'weekly',
              window_minutes: 10_080,
              usage: {
                request_count: 4,
                total_tokens: 800,
                total_cost_usd: '0.25',
              },
            }],
          },
        },
      }),
      'codex',
      'current_cycle',
    )

    expect(display.kind).toBe('codex_cycle')
    if (display.kind !== 'codex_cycle') throw new Error('expected codex cycle display')

    expect(display.groups.map(group => group.code)).toEqual(['weekly'])
    expect(display.groups.map(group => group.label)).toEqual(['1周'])
    expect(metricValues(display.groups[0].metrics)).toEqual({
      request_count: '4',
      total_tokens: '800',
      total_cost_usd: '$0.250',
    })
  })

  it('preserves account-total formatting when toggled away from current cycle', () => {
    const display = buildPoolStatsDisplay(createCodexKey(), 'codex', 'account_total')

    expect(display.kind).toBe('account_total')
    if (display.kind !== 'account_total') throw new Error('expected account total display')

    expect(metricValues(display.metrics)).toEqual({
      request_count: '1,234',
      total_tokens: '5.7M',
      total_cost_usd: '$12.35',
    })
  })

  it('keeps non-Codex providers on account totals even in current-cycle mode', () => {
    const display = buildPoolStatsDisplay(createCodexKey(), 'openai', 'current_cycle')

    expect(display.kind).toBe('account_total')
    if (display.kind !== 'account_total') throw new Error('expected account total display')
    expect(metricValues(display.metrics)).toMatchObject({
      request_count: '1,234',
      total_tokens: '5.7M',
      total_cost_usd: '$12.35',
    })
  })
})
