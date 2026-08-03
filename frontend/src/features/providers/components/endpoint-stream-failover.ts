import { normalizeEndpointApiFormat } from './endpoint-default-paths'

export interface EndpointStreamFailoverState {
  enabled: boolean
  maxRetries: string
  maxWaitSeconds: string
  maxBufferKilobytes: string
  cooldownSeconds: string
}

export type EndpointStreamFailoverValidationError =
  | 'maxRetries'
  | 'maxWaitSeconds'
  | 'maxBufferKilobytes'
  | 'cooldownSeconds'

interface EndpointStreamFailoverSource {
  max_retries: number
  config?: Record<string, unknown>
}

const DEFAULT_MAX_WAIT_MS = 5_000
const DEFAULT_MAX_BUFFER_BYTES = 65_536
const DEFAULT_COOLDOWN_SECONDS = 30

export function isOpenAiResponsesStreamFailoverEndpoint(apiFormat: string): boolean {
  return normalizeEndpointApiFormat(apiFormat) === 'openai:responses'
}

function objectValue(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null
}

function finiteNumber(value: unknown, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) ? value : fallback
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, value))
}

function displayNumber(value: number): string {
  return String(Number(value.toFixed(3)))
}

export function initEndpointStreamFailoverState(
  endpoint: EndpointStreamFailoverSource,
): EndpointStreamFailoverState {
  const config = objectValue(endpoint.config?.stream_failover)
  const maxWaitMs = clamp(finiteNumber(config?.max_wait_ms, DEFAULT_MAX_WAIT_MS), 250, 30_000)
  const maxBufferBytes = clamp(
    finiteNumber(config?.max_buffer_bytes, DEFAULT_MAX_BUFFER_BYTES),
    16_384,
    1_048_576,
  )
  const cooldownSeconds = clamp(
    finiteNumber(config?.cooldown_seconds, DEFAULT_COOLDOWN_SECONDS),
    1,
    1_920,
  )

  return {
    enabled: config?.enabled === true && (config.mode === undefined || config.mode === 'before_output'),
    maxRetries: String(clamp(finiteNumber(endpoint.max_retries, 2), 0, 999)),
    maxWaitSeconds: displayNumber(maxWaitMs / 1_000),
    maxBufferKilobytes: displayNumber(maxBufferBytes / 1_024),
    cooldownSeconds: displayNumber(cooldownSeconds),
  }
}

function parseNumber(value: string): number | null {
  if (!value.trim()) return null
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : null
}

export function validateEndpointStreamFailoverState(
  state: EndpointStreamFailoverState,
): EndpointStreamFailoverValidationError | null {
  const maxRetries = parseNumber(state.maxRetries)
  if (maxRetries === null || !Number.isInteger(maxRetries) || maxRetries < 0 || maxRetries > 999) {
    return 'maxRetries'
  }

  const maxWaitSeconds = parseNumber(state.maxWaitSeconds)
  if (
    maxWaitSeconds === null
    || maxWaitSeconds < 0.25
    || maxWaitSeconds > 30
    || !Number.isInteger(maxWaitSeconds * 1_000)
  ) {
    return 'maxWaitSeconds'
  }

  const maxBufferKilobytes = parseNumber(state.maxBufferKilobytes)
  if (
    maxBufferKilobytes === null
    || !Number.isInteger(maxBufferKilobytes)
    || maxBufferKilobytes < 16
    || maxBufferKilobytes > 1_024
  ) {
    return 'maxBufferKilobytes'
  }

  const cooldownSeconds = parseNumber(state.cooldownSeconds)
  if (
    cooldownSeconds === null
    || !Number.isInteger(cooldownSeconds)
    || cooldownSeconds < 1
    || cooldownSeconds > 1_920
  ) {
    return 'cooldownSeconds'
  }
  return null
}

export function endpointStreamFailoverStateChanged(
  endpoint: EndpointStreamFailoverSource,
  state: EndpointStreamFailoverState,
): boolean {
  return JSON.stringify(initEndpointStreamFailoverState(endpoint)) !== JSON.stringify(state)
}

export function buildEndpointStreamFailoverUpdate(
  endpoint: EndpointStreamFailoverSource,
  state: EndpointStreamFailoverState,
): { max_retries: number; config: Record<string, unknown> } {
  const maxRetries = Number(state.maxRetries)
  const maxWaitMs = Math.round(Number(state.maxWaitSeconds) * 1_000)
  const maxBufferBytes = Math.round(Number(state.maxBufferKilobytes) * 1_024)
  const cooldownSeconds = Number(state.cooldownSeconds)
  const config = { ...(endpoint.config ?? {}) }
  config.stream_failover = {
    enabled: state.enabled,
    mode: 'before_output',
    max_wait_ms: maxWaitMs,
    max_buffer_bytes: maxBufferBytes,
    cooldown_seconds: cooldownSeconds,
  }
  return {
    max_retries: maxRetries,
    config,
  }
}
