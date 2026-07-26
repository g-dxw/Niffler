export interface RoutingCandidateRankVector {
  provider_priority_before: number
  provider_priority_after: number
  key_priority_before: number
  key_priority_after: number
}

export interface RoutingCandidateTrace {
  candidate_kind: 'provider' | 'pool_group'
  provider_id: string
  endpoint_id: string
  model_id: string
  key_id?: string | null
  ranking_vector: RoutingCandidateRankVector
  skip_reason?: string | null
  selected_order?: number | null
}

export interface RoutingDecisionTrace {
  group_id?: string | null
  group_version?: number | null
  selection_source: string
  selected_rules: string[]
  original_model: string
  resolved_model: string
  client_api_format: string
  global_candidates: RoutingCandidateTrace[]
  pool_expansion: unknown[]
  runtime_facts: Record<string, unknown>
}

export function candidateTraceLabel(candidate: RoutingCandidateTrace): string {
  const kind = candidate.candidate_kind === 'pool_group' ? i18n.global.t('routingTraceUi.pool') : 'Provider'
  const key = candidate.key_id ? ` / ${candidate.key_id}` : ''
  return `${kind} ${candidate.provider_id}${key}`
}

export function summarizeRoutingTrace(trace: RoutingDecisionTrace): string[] {
  const lines = [
    `${i18n.global.t('routingTraceUi.group')}: ${trace.group_id ?? 'legacy'}`,
    `${i18n.global.t('routingTraceUi.source')}: ${trace.selection_source}`,
    `${i18n.global.t('routingTraceUi.model')}: ${trace.original_model} -> ${trace.resolved_model}`,
  ]

  if (trace.selected_rules.length > 0) {
    lines.push(`${i18n.global.t('routingTraceUi.rules')}: ${trace.selected_rules.join(', ')}`)
  }

  lines.push(`${i18n.global.t('routingTraceUi.candidates')}: ${trace.global_candidates.length}`)
  return lines
}

export function sortCandidateTraces(candidates: readonly RoutingCandidateTrace[]): RoutingCandidateTrace[] {
  return [...candidates].sort((left, right) => {
    const leftOrder = left.selected_order ?? Number.MAX_SAFE_INTEGER
    const rightOrder = right.selected_order ?? Number.MAX_SAFE_INTEGER
    return leftOrder - rightOrder
  })
}
import { i18n } from '@/i18n'
