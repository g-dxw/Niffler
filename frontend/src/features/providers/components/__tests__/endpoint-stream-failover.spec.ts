import { describe, expect, it } from 'vitest'

import {
  buildEndpointStreamFailoverUpdate,
  endpointStreamFailoverStateChanged,
  initEndpointStreamFailoverState,
  isOpenAiResponsesStreamFailoverEndpoint,
  validateEndpointStreamFailoverState,
} from '../endpoint-stream-failover'

describe('endpoint stream failover settings', () => {
  it('is shown only for the OpenAI Responses endpoint', () => {
    expect(isOpenAiResponsesStreamFailoverEndpoint('openai:responses')).toBe(true)
    expect(isOpenAiResponsesStreamFailoverEndpoint('openai:responses:compact')).toBe(false)
    expect(isOpenAiResponsesStreamFailoverEndpoint('openai:chat')).toBe(false)
  })

  it('uses safe disabled defaults when the endpoint has no configuration', () => {
    expect(initEndpointStreamFailoverState({ max_retries: 2 })).toEqual({
      enabled: false,
      maxRetries: '2',
      maxWaitSeconds: '5',
      maxBufferKilobytes: '64',
      cooldownSeconds: '30',
    })
  })

  it('builds backend units and preserves unrelated endpoint configuration', () => {
    const endpoint = {
      max_retries: 2,
      config: { upstream_stream_policy: 'force_stream' },
    }
    const update = buildEndpointStreamFailoverUpdate(endpoint, {
      enabled: true,
      maxRetries: '3',
      maxWaitSeconds: '7.5',
      maxBufferKilobytes: '128',
      cooldownSeconds: '45',
    })

    expect(update).toEqual({
      max_retries: 3,
      config: {
        upstream_stream_policy: 'force_stream',
        stream_failover: {
          enabled: true,
          mode: 'before_output',
          max_wait_ms: 7_500,
          max_buffer_bytes: 131_072,
          cooldown_seconds: 45,
        },
      },
    })
  })

  it('validates every persisted limit before saving', () => {
    const valid = {
      enabled: true,
      maxRetries: '3',
      maxWaitSeconds: '5',
      maxBufferKilobytes: '64',
      cooldownSeconds: '30',
    }
    expect(validateEndpointStreamFailoverState(valid)).toBeNull()
    expect(validateEndpointStreamFailoverState({ ...valid, maxRetries: '-1' })).toBe('maxRetries')
    expect(validateEndpointStreamFailoverState({ ...valid, maxWaitSeconds: '31' })).toBe('maxWaitSeconds')
    expect(validateEndpointStreamFailoverState({ ...valid, maxBufferKilobytes: '15' })).toBe('maxBufferKilobytes')
    expect(validateEndpointStreamFailoverState({ ...valid, cooldownSeconds: '0' })).toBe('cooldownSeconds')
  })

  it('reports changes against the stored endpoint values', () => {
    const endpoint = { max_retries: 2 }
    const state = initEndpointStreamFailoverState(endpoint)
    expect(endpointStreamFailoverStateChanged(endpoint, state)).toBe(false)
    expect(endpointStreamFailoverStateChanged(endpoint, { ...state, enabled: true })).toBe(true)
  })
})
