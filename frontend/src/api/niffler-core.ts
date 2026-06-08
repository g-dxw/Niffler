import apiClient from './client'

export type NifflerReadinessSeverity = 'info' | 'warning' | 'error'
export type NifflerAccountStatus =
  | 'available'
  | 'disabled'
  | 'invalid'
  | 'quota_exhausted'
  | 'cooling_down'
export type NifflerProtocolKind = 'openai' | 'anthropic' | 'gemini' | 'codex' | 'custom'
export type NifflerServiceCapabilityKind =
  | 'text'
  | 'streaming'
  | 'images_endpoint'
  | 'openai_responses_image_tool'
  | 'model_list'
  | 'model_test'
export type NifflerErrorResponseScope = 'platform' | 'upstream'
export type NifflerUpstreamErrorHandlingStep =
  | 'risk_keyword'
  | 'contact_or_marketing_replacement'
  | 'status_code_message'
  | 'default_upstream_message'
export type NifflerUserResponseMode = 'replace' | 'append' | 'redact'
export type NifflerAccountProtectionAction = 'record_only' | 'pause_scheduling' | 'disable_account'
export type NifflerPauseDuration =
  | 'ten_minutes'
  | 'one_hour'
  | 'twenty_four_hours'
  | 'manual_restore'
export type NifflerRuntimeRolloutTargetScope = 'api_key' | 'product_plan'
export type NifflerBillingReservationStatus =
  | 'active'
  | 'settled'
  | 'released'
  | 'expired'
  | 'manual_review'
export type NifflerReferralRewardLedgerStatus = 'pending' | 'paid' | 'failed' | 'cancelled'

export interface NifflerUpstreamService {
  id: string
  display_name: string
  service_kind: string
  default_api_format?: string | null
  base_url?: string | null
  cost_multiplier: number
  is_active: boolean
  config?: Record<string, unknown> | null
  created_at_unix_ms: number
  updated_at_unix_ms: number
}

export interface NifflerUpstreamServiceCapability {
  id: string
  upstream_service_id: string
  protocol_kind: NifflerProtocolKind
  capability_kind: NifflerServiceCapabilityKind
  is_enabled: boolean
  config?: Record<string, unknown> | null
  created_at_unix_ms: number
  updated_at_unix_ms: number
}

export interface NifflerUpstreamAccount {
  id: string
  upstream_service_id: string
  display_name: string
  email?: string | null
  phone?: string | null
  auth_kind: string
  status: NifflerAccountStatus
  cost_multiplier: number
  priority: number
  cooldown_until_unix_ms?: number | null
  last_tested_at_unix_ms?: number | null
  last_test_error?: string | null
  config?: Record<string, unknown> | null
  created_at_unix_ms: number
  updated_at_unix_ms: number
}

export interface NifflerProductPlan {
  id: string
  display_name: string
  is_public: boolean
  is_active: boolean
  sales_multiplier: number
  description?: string | null
  created_at_unix_ms: number
  updated_at_unix_ms: number
}

export interface NifflerProductPlanModel {
  id: string
  product_plan_id: string
  model_name: string
  is_enabled: boolean
  sales_multiplier_override?: number | null
  created_at_unix_ms: number
  updated_at_unix_ms: number
}

export interface NifflerApiKeyProductPlanBinding {
  id: string
  api_key_id: string
  product_plan_id: string
  config?: Record<string, unknown> | null
  created_at_unix_ms: number
  updated_at_unix_ms: number
}

export interface NifflerRuntimeRolloutSetting {
  id: string
  target_scope: NifflerRuntimeRolloutTargetScope
  target_id: string
  enable_new_routing: boolean
  enable_settlement_snapshot: boolean
  enable_error_return_rules: boolean
  enable_billing_reservation: boolean
  enable_referral_ledger: boolean
  is_active: boolean
  config?: Record<string, unknown> | null
  created_at_unix_ms: number
  updated_at_unix_ms: number
}

export interface NifflerRuntimeRolloutDecision {
  runtime_effect: 'preview_only'
  source_scope?: NifflerRuntimeRolloutTargetScope | null
  source_label?: string | null
  reason: string
  is_active: boolean
  enable_new_routing: boolean
  enable_settlement_snapshot: boolean
  enable_error_return_rules: boolean
  enable_billing_reservation: boolean
  enable_referral_ledger: boolean
}

export interface NifflerRuntimeRolloutPreview {
  api_key: {
    id: string
    name?: string | null
    owner_label: string
    user_id: string
    user_is_active: boolean
    user_is_deleted: boolean
    is_active: boolean
    is_locked: boolean
    is_standalone: boolean
  }
  product_plan?: {
    id: string
    display_name?: string | null
    is_active: boolean
    binding_id?: string | null
    binding_updated_at_unix_ms?: number | null
  } | null
  key_setting?: NifflerRuntimeRolloutSetting | null
  product_plan_setting?: NifflerRuntimeRolloutSetting | null
  decision: NifflerRuntimeRolloutDecision
  warnings: string[]
}

export interface NifflerErrorReturnSetting {
  id: string
  scope: NifflerErrorResponseScope
  upstream_service_id?: string | null
  match_status_code?: number | null
  match_text?: string | null
  handling_step?: NifflerUpstreamErrorHandlingStep | null
  response_mode: NifflerUserResponseMode
  user_message: string
  account_protection_action: NifflerAccountProtectionAction
  pause_duration?: NifflerPauseDuration | null
  is_active: boolean
  created_at_unix_ms: number
  updated_at_unix_ms: number
}

export interface NifflerBillingReservation {
  id: string
  request_id: string
  user_id?: string | null
  api_key_id?: string | null
  product_plan_id?: string | null
  status: NifflerBillingReservationStatus
  reserved_total_usd: number
  wallet_reserved_usd: number
  entitlement_reserved_usd: number
  reserved_at_unix_ms: number
  expires_at_unix_ms: number
  finalized_at_unix_ms?: number | null
  settlement_snapshot_id?: string | null
  release_reason?: string | null
  idempotency_key: string
}

export interface NifflerReferralRewardLedger {
  id: string
  order_id: string
  idempotency_key: string
  inviter_user_id: string
  invitee_user_id: string
  rule_id?: string | null
  reward_amount_usd: number
  rule_snapshot: Record<string, unknown> | unknown[]
  status: NifflerReferralRewardLedgerStatus
  failure_reason?: string | null
  retry_count: number
  paid_at_unix_ms?: number | null
  cancelled_at_unix_ms?: number | null
  created_at_unix_ms: number
  updated_at_unix_ms: number
}

export interface NifflerRouteAttempt {
  id: string
  request_id: string
  upstream_service_id?: string | null
  upstream_service_name?: string | null
  upstream_account_id?: string | null
  upstream_account_display_name?: string | null
  upstream_account_email?: string | null
  upstream_account_phone?: string | null
  product_plan_id?: string | null
  product_plan_name?: string | null
  model_name: string
  attempt_index: number
  status: string
  skip_reason?: string | null
  upstream_status_code?: number | null
  latency_ms?: number | null
  created_at_unix_ms: number
}

export interface NifflerListPage<T> {
  items: T[]
  total: number
}

export interface CreateNifflerUpstreamServicePayload {
  display_name: string
  service_kind: string
  protocol_kind?: NifflerProtocolKind
  default_api_format?: string | null
  base_url?: string | null
  cost_multiplier?: number
  is_active?: boolean
  capabilities?: {
    text?: boolean
    streaming?: boolean
    images_endpoint?: boolean
    openai_responses_image_tool?: boolean
    model_list?: boolean
    model_test?: boolean
  }
}

export interface UpdateNifflerUpstreamServiceCapabilitiesPayload {
  protocol_kind: NifflerProtocolKind
  capabilities: NonNullable<CreateNifflerUpstreamServicePayload['capabilities']>
}

export interface CreateNifflerUpstreamAccountPayload {
  display_name: string
  email?: string | null
  phone?: string | null
  auth_kind: 'api_key' | 'oauth' | 'custom_header'
  cost_multiplier?: number
  priority?: number
}

export interface CreateNifflerProductPlanPayload {
  display_name: string
  is_public?: boolean
  is_active?: boolean
  sales_multiplier?: number
  description?: string | null
}

export interface UpsertNifflerProductPlanModelPayload {
  model_name: string
  is_enabled?: boolean
  sales_multiplier_override?: number | null
}

export interface UpsertNifflerApiKeyProductPlanBindingPayload {
  api_key_id: string
}

export interface UpsertNifflerRuntimeRolloutSettingPayload {
  target_scope: NifflerRuntimeRolloutTargetScope
  target_id: string
  enable_new_routing?: boolean
  enable_settlement_snapshot?: boolean
  enable_error_return_rules?: boolean
  enable_billing_reservation?: boolean
  enable_referral_ledger?: boolean
  is_active?: boolean
}

export interface CreateNifflerErrorReturnSettingPayload {
  scope: NifflerErrorResponseScope
  upstream_service_id?: string | null
  match_status_code?: number | null
  match_text?: string | null
  handling_step?: NifflerUpstreamErrorHandlingStep | null
  response_mode?: NifflerUserResponseMode
  user_message: string
  account_protection_action?: NifflerAccountProtectionAction
  pause_duration?: NifflerPauseDuration | null
  is_active?: boolean
}

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
  product_plans_total: number
  product_plans_public: number
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
  product_plan_id: string
  product_plan_name: string
  provider_id: string
  provider_name: string
  source_field: string
  source_field_label: string
  reason: string
  impact: string
  recommended_action: string
}

export interface NifflerKeyScopeResidue {
  subject_kind: string
  key_id: string
  key_name?: string | null
  owner_label?: string | null
  display_name: string
  provider_id?: string | null
  provider_name?: string | null
  account_label?: string | null
  residue_fields: string[]
  field_labels: string[]
  reason: string
  impact: string
  recommended_action: string
}

export interface NifflerGroupPolicyGap {
  product_plan_id: string
  product_plan_name: string
  gap_kind: string
  gap_label: string
  message: string
  impact: string
  recommended_action: string
}

export interface NifflerPriceGap {
  scope: string
  scope_label: string
  provider_id?: string | null
  provider_name?: string | null
  model_id?: string | null
  model_name: string
  missing_fields: string[]
  reason: string
  impact: string
  recommended_action: string
}

export interface NifflerUsageAnomaly {
  usage_id: string
  request_id: string
  created_at_unix_secs: number
  provider_name: string
  provider_id?: string | null
  provider_api_key_id?: string | null
  provider_display_name: string
  provider_api_key_name?: string | null
  provider_account_label?: string | null
  model: string
  status: string
  billing_status: string
  status_code?: number | null
  error_category?: string | null
  anomaly_kind: string
  anomaly_label: string
  diagnosis: string
  impact: string
  recommended_action: string
  total_cost_usd: number
  actual_total_cost_usd: number
  package_debit_usd?: number | null
  wallet_debit_usd?: number | null
}

export interface NifflerRouteSkipReasonSummary {
  reason: string
  label: string
  category: string
  count: number
  impact: string
  recommended_action: string
}

export interface NifflerRouteSkipSample {
  request_id: string
  created_at_unix_secs: number
  provider_id?: string | null
  provider_name?: string | null
  key_id?: string | null
  key_name?: string | null
  account_label?: string | null
  reason: string
  label: string
  impact: string
  recommended_action: string
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
  route_skip_samples: NifflerRouteSkipSample[]
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

export async function listNifflerUpstreamServices(params?: {
  include_inactive?: boolean
  search?: string
  offset?: number
  limit?: number
}): Promise<NifflerListPage<NifflerUpstreamService>> {
  const response = await apiClient.get<NifflerListPage<NifflerUpstreamService>>(
    '/api/admin/niffler-core/upstream-services',
    { params }
  )
  return response.data
}

export async function createNifflerUpstreamService(
  payload: CreateNifflerUpstreamServicePayload
): Promise<NifflerUpstreamService> {
  const response = await apiClient.post<NifflerUpstreamService>(
    '/api/admin/niffler-core/upstream-services',
    payload
  )
  return response.data
}

export async function listNifflerUpstreamServiceCapabilities(
  upstreamServiceId: string
): Promise<NifflerListPage<NifflerUpstreamServiceCapability>> {
  const response = await apiClient.get<NifflerListPage<NifflerUpstreamServiceCapability>>(
    `/api/admin/niffler-core/upstream-services/${encodeURIComponent(upstreamServiceId)}/capabilities`
  )
  return response.data
}

export async function updateNifflerUpstreamServiceCapabilities(
  upstreamServiceId: string,
  payload: UpdateNifflerUpstreamServiceCapabilitiesPayload
): Promise<NifflerListPage<NifflerUpstreamServiceCapability>> {
  const response = await apiClient.put<NifflerListPage<NifflerUpstreamServiceCapability>>(
    `/api/admin/niffler-core/upstream-services/${encodeURIComponent(upstreamServiceId)}/capabilities`,
    payload
  )
  return response.data
}

export async function listNifflerUpstreamAccounts(
  upstreamServiceId: string,
  params?: {
    status?: NifflerAccountStatus
    search?: string
    offset?: number
    limit?: number
  }
): Promise<NifflerListPage<NifflerUpstreamAccount>> {
  const response = await apiClient.get<NifflerListPage<NifflerUpstreamAccount>>(
    `/api/admin/niffler-core/upstream-services/${encodeURIComponent(upstreamServiceId)}/accounts`,
    { params }
  )
  return response.data
}

export async function createNifflerUpstreamAccount(
  upstreamServiceId: string,
  payload: CreateNifflerUpstreamAccountPayload
): Promise<NifflerUpstreamAccount> {
  const response = await apiClient.post<NifflerUpstreamAccount>(
    `/api/admin/niffler-core/upstream-services/${encodeURIComponent(upstreamServiceId)}/accounts`,
    payload
  )
  return response.data
}

export async function listNifflerProductPlans(params?: {
  include_inactive?: boolean
  public_only?: boolean
  search?: string
  offset?: number
  limit?: number
}): Promise<NifflerListPage<NifflerProductPlan>> {
  const response = await apiClient.get<NifflerListPage<NifflerProductPlan>>(
    '/api/admin/niffler-core/product-plans',
    { params }
  )
  return response.data
}

export async function createNifflerProductPlan(
  payload: CreateNifflerProductPlanPayload
): Promise<NifflerProductPlan> {
  const response = await apiClient.post<NifflerProductPlan>(
    '/api/admin/niffler-core/product-plans',
    payload
  )
  return response.data
}

export async function listNifflerProductPlanModels(
  productPlanId: string,
  params?: {
    enabled_only?: boolean
    search?: string
    offset?: number
    limit?: number
  }
): Promise<NifflerListPage<NifflerProductPlanModel>> {
  const response = await apiClient.get<NifflerListPage<NifflerProductPlanModel>>(
    `/api/admin/niffler-core/product-plans/${encodeURIComponent(productPlanId)}/models`,
    { params }
  )
  return response.data
}

export async function upsertNifflerProductPlanModel(
  productPlanId: string,
  payload: UpsertNifflerProductPlanModelPayload
): Promise<NifflerProductPlanModel> {
  const response = await apiClient.post<NifflerProductPlanModel>(
    `/api/admin/niffler-core/product-plans/${encodeURIComponent(productPlanId)}/models`,
    payload
  )
  return response.data
}

export async function listNifflerApiKeyProductPlanBindings(
  params?: {
    offset?: number
    limit?: number
  }
): Promise<NifflerListPage<NifflerApiKeyProductPlanBinding>> {
  const response = await apiClient.get<NifflerListPage<NifflerApiKeyProductPlanBinding>>(
    '/api/admin/niffler-core/api-key-product-plan-bindings',
    { params }
  )
  return response.data
}

export async function upsertNifflerApiKeyProductPlanBinding(
  productPlanId: string,
  payload: UpsertNifflerApiKeyProductPlanBindingPayload
): Promise<NifflerApiKeyProductPlanBinding> {
  const response = await apiClient.post<NifflerApiKeyProductPlanBinding>(
    `/api/admin/niffler-core/product-plans/${encodeURIComponent(productPlanId)}/api-key-bindings`,
    payload
  )
  return response.data
}

export async function listNifflerRuntimeRolloutSettings(params?: {
  target_scope?: NifflerRuntimeRolloutTargetScope
  include_inactive?: boolean
  offset?: number
  limit?: number
}): Promise<NifflerListPage<NifflerRuntimeRolloutSetting>> {
  const response = await apiClient.get<NifflerListPage<NifflerRuntimeRolloutSetting>>(
    '/api/admin/niffler-core/runtime-rollout-settings',
    { params }
  )
  return response.data
}

export async function upsertNifflerRuntimeRolloutSetting(
  payload: UpsertNifflerRuntimeRolloutSettingPayload
): Promise<NifflerRuntimeRolloutSetting> {
  const response = await apiClient.post<NifflerRuntimeRolloutSetting>(
    '/api/admin/niffler-core/runtime-rollout-settings',
    payload
  )
  return response.data
}

export async function getNifflerRuntimeRolloutPreview(
  apiKeyId: string
): Promise<NifflerRuntimeRolloutPreview> {
  const response = await apiClient.get<NifflerRuntimeRolloutPreview>(
    '/api/admin/niffler-core/runtime-rollout-preview',
    { params: { api_key_id: apiKeyId } }
  )
  return response.data
}

export async function listNifflerErrorReturnSettings(params?: {
  scope?: NifflerErrorResponseScope
  upstream_service_id?: string
  include_inactive?: boolean
  offset?: number
  limit?: number
}): Promise<NifflerListPage<NifflerErrorReturnSetting>> {
  const response = await apiClient.get<NifflerListPage<NifflerErrorReturnSetting>>(
    '/api/admin/niffler-core/error-return-settings',
    { params }
  )
  return response.data
}

export async function createNifflerErrorReturnSetting(
  payload: CreateNifflerErrorReturnSettingPayload
): Promise<NifflerErrorReturnSetting> {
  const response = await apiClient.post<NifflerErrorReturnSetting>(
    '/api/admin/niffler-core/error-return-settings',
    payload
  )
  return response.data
}

export async function listNifflerBillingReservations(params?: {
  status?: NifflerBillingReservationStatus
  user_id?: string
  api_key_id?: string
  request_id?: string
  offset?: number
  limit?: number
}): Promise<NifflerListPage<NifflerBillingReservation>> {
  const response = await apiClient.get<NifflerListPage<NifflerBillingReservation>>(
    '/api/admin/niffler-core/billing-reservations',
    { params }
  )
  return response.data
}

export async function listNifflerReferralRewardLedger(params?: {
  status?: NifflerReferralRewardLedgerStatus
  inviter_user_id?: string
  invitee_user_id?: string
  order_id?: string
  offset?: number
  limit?: number
}): Promise<NifflerListPage<NifflerReferralRewardLedger>> {
  const response = await apiClient.get<NifflerListPage<NifflerReferralRewardLedger>>(
    '/api/admin/niffler-core/referral-reward-ledger',
    { params }
  )
  return response.data
}

export async function listNifflerRouteAttempts(params?: {
  request_id?: string
  upstream_service_id?: string
  upstream_account_id?: string
  status?: string
  offset?: number
  limit?: number
}): Promise<NifflerListPage<NifflerRouteAttempt>> {
  const response = await apiClient.get<NifflerListPage<NifflerRouteAttempt>>(
    '/api/admin/niffler-core/route-attempts',
    { params }
  )
  return response.data
}
