import { describe, expect, it } from 'vitest'

import { getQuotaDisplayText } from '../providerKeyQuota'

describe('providerKeyQuota', () => {
  it('does not invent a Codex quota when upstream returned no current window', () => {
    expect(getQuotaDisplayText({
      status_snapshot: {
        oauth: {
          code: 'valid',
        },
        account: {
          code: 'ok',
          blocked: false,
        },
        quota: {
          provider_type: 'codex',
          code: 'unknown',
          exhausted: false,
          windows: [],
        },
      },
    }, 'codex')).toBe('暂无上游配额数据')
  })

  it('displays every upstream Codex quota window by its real duration', () => {
    expect(getQuotaDisplayText({
      status_snapshot: {
        oauth: {
          code: 'valid',
        },
        account: {
          code: 'ok',
          blocked: false,
        },
        quota: {
          provider_type: 'codex',
          code: 'ok',
          exhausted: false,
          windows: [
            {
              code: 'weekly',
              window_minutes: 10_080,
              remaining_ratio: 0.9,
            },
            {
              code: '5h',
              window_minutes: 300,
              remaining_ratio: 0.8,
            },
            {
              code: '1m',
              window_minutes: 43_200,
              remaining_ratio: 0.6,
            },
            {
              code: 'window_12345s',
              window_seconds: 12_345,
              remaining_ratio: 0.95,
            },
          ],
        },
      },
    }, 'codex')).toBe('7D剩余 90.0% | 5H剩余 80.0% | 1M剩余 60.0% | 3小时26分钟剩余 95.0%')
  })

  it('formats Grok account quota from structured quota windows', () => {
    expect(getQuotaDisplayText({
      status_snapshot: {
        oauth: {
          code: 'valid',
        },
        account: {
          code: 'ok',
          blocked: false,
        },
        quota: {
          provider_type: 'grok',
          code: 'ok',
          exhausted: false,
          windows: [
            {
              scope: 'account',
              used_value: 2,
              limit_value: 10,
              remaining_ratio: 0.8,
            },
          ],
        },
      },
    }, 'grok')).toBe('剩余 80.0% (8/10)')
  })

  it('formats Grok mode quota from model-scoped windows', () => {
    expect(getQuotaDisplayText({
      status_snapshot: {
        oauth: {
          code: 'valid',
        },
        account: {
          code: 'ok',
          blocked: false,
        },
        quota: {
          provider_type: 'grok',
          code: 'ok',
          exhausted: false,
          plan_type: 'heavy',
          windows: [
            {
              code: 'model:quota_auto',
              label: 'auto',
              scope: 'model',
              remaining_ratio: 0.4,
              used_value: 90,
              limit_value: 150,
            },
            {
              code: 'model:quota_heavy',
              label: 'heavy',
              scope: 'model',
              remaining_ratio: 0,
              used_value: 20,
              limit_value: 20,
            },
          ],
        },
      },
    }, 'grok')).toBe('Auto剩余 40.0% (60/150) | Heavy剩余 0.0% (0/20)')
  })
})
