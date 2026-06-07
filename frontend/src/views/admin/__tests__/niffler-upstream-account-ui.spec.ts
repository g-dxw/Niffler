import { describe, expect, it } from 'vitest'

import {
  formatNifflerAccountTestStatus,
  formatNifflerUnixMs,
  getNifflerAccountAuthGuide,
} from '../niffler-upstream-account-ui'

describe('niffler upstream account ui helpers', () => {
  it('describes account auth kinds without asking for credentials in this slice', () => {
    expect(getNifflerAccountAuthGuide('oauth').description).toContain('不保存 Refresh Token')
    expect(getNifflerAccountAuthGuide('api_key').description).toContain('不保存真实 API Key')
    expect(getNifflerAccountAuthGuide('custom_header').description).toContain('不保存 Header')
  })

  it('formats account test status from stored test fields', () => {
    expect(formatNifflerAccountTestStatus({ last_test_error: '401', last_tested_at_unix_ms: 1000 }))
      .toBe('测试失败')
    expect(formatNifflerAccountTestStatus({ last_test_error: '', last_tested_at_unix_ms: 1000 }))
      .toBe('测试通过')
    expect(formatNifflerAccountTestStatus({ last_test_error: null, last_tested_at_unix_ms: null }))
      .toBe('未测试')
  })

  it('formats unix milliseconds and hides invalid values', () => {
    expect(formatNifflerUnixMs(null)).toBe('-')
    expect(formatNifflerUnixMs(0)).toBe('-')
    expect(formatNifflerUnixMs(1_700_000_000_000)).toContain('2023')
  })
})
