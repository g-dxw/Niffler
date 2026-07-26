import { describe, expect, it } from 'vitest'

import { isKeyManagedProviderType, isOAuthAccountProviderType } from '../providerTypeUtils'

describe('providerTypeUtils', () => {
  it('treats ChatGPT-Web as an OAuth account provider', () => {
    expect(isOAuthAccountProviderType('chatgpt_web')).toBe(true)
    expect(isOAuthAccountProviderType('ChatGPT_Web')).toBe(true)
    expect(isKeyManagedProviderType('chatgpt_web')).toBe(false)
  })

  it('treats Grok as an OAuth account provider', () => {
    expect(isOAuthAccountProviderType('grok')).toBe(true)
    expect(isOAuthAccountProviderType('GROK')).toBe(true)
    expect(isKeyManagedProviderType('grok')).toBe(false)
  })

  it('routes Grok OAuth through account management', () => {
    expect(isOAuthAccountProviderType('grok_oauth')).toBe(true)
    expect(isOAuthAccountProviderType('GROK_OAUTH')).toBe(true)
    expect(isKeyManagedProviderType('grok_oauth')).toBe(false)
  })
})
