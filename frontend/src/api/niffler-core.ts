import apiClient from './client'

export type NifflerReadinessSeverity = 'info' | 'warning' | 'error'

export interface NifflerShadowTableItem {
  table_name: string
  exists: boolean
}

export interface NifflerShadowTableStatus {
  database_driver?: string | null
  expected_tables: number
  existing_tables: number
  all_present: boolean
  tables: NifflerShadowTableItem[]
}

export interface NifflerCoreReadinessSummary {
  providers_total: number
  providers_active: number
  provider_keys_total: number
  provider_keys_active: number
  routing_groups_total: number
  routing_groups_enabled: number
  global_models_total: number
  global_models_active: number
  recent_problem_usage_sample_count: number
}

export interface NifflerCoreMappingSummary {
  legacy_count: number
  mapped_count: number
  blocked_count: number
  notes: string[]
}

export interface NifflerDisabledProviderReference {
  routing_group_id: string
  routing_group_name: string
  provider_id: string
  provider_name: string
  source_field: string
}

export interface NifflerKeyScopeResidue {
  subject_kind: string
  key_id: string
  key_name?: string | null
  owner_label?: string | null
  residue_fields: string[]
  impact: string
}

export interface NifflerGroupPolicyGap {
  routing_group_id: string
  routing_group_name: string
  gap_kind: string
  message: string
}

export interface NifflerPriceGap {
  scope: string
  provider_id?: string | null
  provider_name?: string | null
  model_id?: string | null
  model_name: string
  missing_fields: string[]
}

export interface NifflerUsageAnomaly {
  usage_id: string
  request_id: string
  created_at_unix_ms: number
  provider_name: string
  provider_id?: string | null
  provider_api_key_id?: string | null
  model: string
  status: string
  billing_status: string
  status_code?: number | null
  error_category?: string | null
  diagnosis: string
}

export interface NifflerRouteSkipReasonSummary {
  reason: string
  count: number
}

export interface NifflerReadinessIssue {
  severity: NifflerReadinessSeverity
  code: string
  title: string
  message: string
}

export interface NifflerCoreReadinessReport {
  schema_version: number
  generated_at_unix_secs: number
  recent_days: number
  shadow_tables: NifflerShadowTableStatus
  summary: NifflerCoreReadinessSummary
  provider_mapping: NifflerCoreMappingSummary
  account_mapping: NifflerCoreMappingSummary
  product_plan_mapping: NifflerCoreMappingSummary
  provider_status_counts: Record<string, number>
  account_status_counts: Record<string, number>
  disabled_provider_references: NifflerDisabledProviderReference[]
  key_scope_residue: NifflerKeyScopeResidue[]
  group_policy_gaps: NifflerGroupPolicyGap[]
  price_gaps: NifflerPriceGap[]
  recent_usage_anomalies: NifflerUsageAnomaly[]
  route_skip_reasons: NifflerRouteSkipReasonSummary[]
  issues: NifflerReadinessIssue[]
}

export async function getNifflerCoreReadiness(params?: {
  recent_days?: number
}): Promise<NifflerCoreReadinessReport> {
  const response = await apiClient.get<NifflerCoreReadinessReport>(
    '/api/admin/niffler-core/readiness',
    { params }
  )
  return response.data
}
