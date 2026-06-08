use std::collections::{BTreeMap, BTreeSet};

use aether_data::repository::auth::{
    StandaloneApiKeyExportListQuery, StoredAuthApiKeyExportRecord,
};
use aether_data::repository::users::StoredUserGroup;
use aether_data_contracts::repository::candidates::StoredRequestCandidate;
use aether_data_contracts::repository::global_models::{
    AdminGlobalModelListQuery, AdminProviderModelListQuery, StoredAdminGlobalModel,
    StoredAdminProviderModel,
};
use aether_data_contracts::repository::niffler_core::{
    CreateNifflerErrorReturnSettingRecord, CreateNifflerProductPlanRecord,
    CreateNifflerUpstreamAccountRecord, CreateNifflerUpstreamServiceRecord,
    NifflerAccountProtectionAction, NifflerAccountStatus, NifflerApiKeyProductPlanBindingListQuery,
    NifflerBillingReservationDryRunListQuery, NifflerBillingReservationListQuery,
    NifflerBillingReservationStatus, NifflerConsistencyCheckListQuery, NifflerCoreMappingSummary,
    NifflerCoreReadinessReport, NifflerCoreReadinessSummary, NifflerDisabledProviderReference,
    NifflerErrorResponseScope, NifflerErrorReturnSettingListQuery, NifflerGroupPolicyGap,
    NifflerKeyScopeResidue, NifflerPauseDuration, NifflerPriceGap, NifflerProductPlanListQuery,
    NifflerProductPlanModelListQuery, NifflerProtocolKind, NifflerReadinessIssue,
    NifflerReadinessSeverity, NifflerReferralRewardLedgerListQuery,
    NifflerReferralRewardLedgerStatus, NifflerRouteAttemptListQuery, NifflerRouteSkipReasonSummary,
    NifflerRouteSkipSample, NifflerRuntimeRolloutSettingListQuery,
    NifflerRuntimeRolloutTargetScope, NifflerServiceCapabilityKind,
    NifflerSettlementSnapshotListQuery, NifflerShadowTableItem, NifflerShadowTableStatus,
    NifflerUpstreamAccountListQuery, NifflerUpstreamErrorHandlingStep,
    NifflerUpstreamServiceCapabilityListQuery, NifflerUpstreamServiceListQuery,
    NifflerUsageAnomaly, NifflerUserResponseMode, UpsertNifflerApiKeyProductPlanBindingRecord,
    UpsertNifflerProductPlanModelRecord, UpsertNifflerRuntimeRolloutSettingRecord,
    UpsertNifflerUpstreamServiceCapabilityRecord,
};
use aether_data_contracts::repository::provider_catalog::{
    StoredProviderCatalogKey, StoredProviderCatalogProvider,
};
use aether_data_contracts::repository::usage::{StoredRequestUsageAudit, UsageAuditListQuery};
use axum::{
    body::Body,
    http,
    response::{IntoResponse, Response},
    Json,
};

use crate::clock::current_unix_secs;
use crate::handlers::admin::request::{AdminAppState, AdminRequestContext, AdminRouteRequest};
use crate::handlers::admin::shared::{attach_admin_audit_response, query_param_value};
use crate::handlers::shared::{
    parse_catalog_auth_config_json, provider_key_account_label_from_auth_config,
};
use crate::GatewayError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

const READINESS_PATH: &str = "/api/admin/niffler-core/readiness";
const LEGACY_DEPENDENCY_AUDIT_PATH: &str = "/api/admin/niffler-core/legacy-dependency-audit";
const UPSTREAM_SERVICES_PATH: &str = "/api/admin/niffler-core/upstream-services";
const PRODUCT_PLANS_PATH: &str = "/api/admin/niffler-core/product-plans";
const API_KEY_PRODUCT_PLAN_BINDINGS_PATH: &str =
    "/api/admin/niffler-core/api-key-product-plan-bindings";
const RUNTIME_ROLLOUT_SETTINGS_PATH: &str = "/api/admin/niffler-core/runtime-rollout-settings";
const RUNTIME_ROLLOUT_PREVIEW_PATH: &str = "/api/admin/niffler-core/runtime-rollout-preview";
const ERROR_RETURN_SETTINGS_PATH: &str = "/api/admin/niffler-core/error-return-settings";
const BILLING_RESERVATIONS_PATH: &str = "/api/admin/niffler-core/billing-reservations";
const BILLING_RESERVATION_DRY_RUNS_PATH: &str =
    "/api/admin/niffler-core/billing-reservation-dry-runs";
const SETTLEMENT_SNAPSHOTS_PATH: &str = "/api/admin/niffler-core/settlement-snapshots";
const REFERRAL_REWARD_LEDGER_PATH: &str = "/api/admin/niffler-core/referral-reward-ledger";
const ROUTE_ATTEMPTS_PATH: &str = "/api/admin/niffler-core/route-attempts";
const CONSISTENCY_CHECKS_PATH: &str = "/api/admin/niffler-core/consistency-checks";
const MAX_ISSUE_ITEMS: usize = 50;
const MAX_USAGE_SCAN: usize = 200;
const MAX_USAGE_ITEMS: usize = 50;
const MAX_PROVIDER_MODELS_PER_PROVIDER: usize = 2_000;
const MAX_GLOBAL_MODELS: usize = 10_000;
const MAX_ROUTE_SKIP_SAMPLE: usize = 500;
const MAX_LEGACY_AUDIT_LIMIT: usize = 100;
const SHADOW_TABLES: &[&str] = &[
    "niffler_upstream_services",
    "niffler_upstream_accounts",
    "niffler_product_plans",
    "niffler_product_plan_models",
    "niffler_api_key_product_plan_bindings",
    "niffler_model_base_prices",
    "niffler_upstream_model_prices",
    "niffler_account_model_capabilities",
    "niffler_upstream_service_capabilities",
    "niffler_settlement_snapshots",
    "niffler_billing_reservations",
    "niffler_billing_reservation_dry_runs",
    "niffler_billing_reservation_events",
    "niffler_route_attempts",
    "niffler_error_return_settings",
    "niffler_account_risk_events",
    "niffler_api_key_pauses",
    "niffler_runtime_rollout_settings",
    "niffler_referral_reward_rules",
    "niffler_referral_reward_ledger",
    "niffler_referral_reward_events",
];

pub(crate) async fn maybe_build_local_admin_niffler_response(
    request: AdminRouteRequest<'_>,
) -> Result<Option<Response<Body>>, GatewayError> {
    let state = request.state();
    let request_context = request.request_context();

    if request_context.route_family() != Some("niffler_core_manage") {
        return Ok(None);
    }

    if request_context.path() == READINESS_PATH {
        if request_context.method() != http::Method::GET {
            return Ok(Some(niffler_method_not_allowed("只支持只读检查")));
        }
        let recent_days = parse_recent_days(request_context.query_string());
        let report = build_readiness_report(&state, recent_days).await?;
        return Ok(Some(Json(report).into_response()));
    }

    if request_context.path().trim_end_matches('/') == LEGACY_DEPENDENCY_AUDIT_PATH {
        if request_context.method() != http::Method::GET {
            return Ok(Some(niffler_method_not_allowed("只支持只读稽核")));
        }
        return Ok(Some(
            build_legacy_dependency_audit_response(&state, &request_context).await?,
        ));
    }

    if request_context.path().trim_end_matches('/') == UPSTREAM_SERVICES_PATH {
        return Ok(Some(
            build_upstream_services_response(&state, &request_context, request.request_body())
                .await?,
        ));
    }

    if let Some(upstream_service_id) = upstream_service_capabilities_path_id(request_context.path())
    {
        return Ok(Some(
            build_upstream_service_capabilities_response(
                &state,
                &request_context,
                request.request_body(),
                upstream_service_id,
            )
            .await?,
        ));
    }

    if let Some(upstream_service_id) = upstream_service_accounts_path_id(request_context.path()) {
        return Ok(Some(
            build_upstream_accounts_response(
                &state,
                &request_context,
                request.request_body(),
                upstream_service_id,
            )
            .await?,
        ));
    }

    if request_context.path().trim_end_matches('/') == PRODUCT_PLANS_PATH {
        return Ok(Some(
            build_product_plans_response(&state, &request_context, request.request_body()).await?,
        ));
    }

    if let Some(product_plan_id) = product_plan_models_path_id(request_context.path()) {
        return Ok(Some(
            build_product_plan_models_response(
                &state,
                &request_context,
                request.request_body(),
                product_plan_id,
            )
            .await?,
        ));
    }

    if let Some(product_plan_id) = product_plan_api_key_bindings_path_id(request_context.path()) {
        return Ok(Some(
            build_product_plan_api_key_bindings_response(
                &state,
                &request_context,
                request.request_body(),
                product_plan_id,
            )
            .await?,
        ));
    }

    if request_context.path().trim_end_matches('/') == API_KEY_PRODUCT_PLAN_BINDINGS_PATH {
        return Ok(Some(
            build_all_api_key_product_plan_bindings_response(&state, &request_context).await?,
        ));
    }

    if request_context.path().trim_end_matches('/') == RUNTIME_ROLLOUT_SETTINGS_PATH {
        return Ok(Some(
            build_runtime_rollout_settings_response(
                &state,
                &request_context,
                request.request_body(),
            )
            .await?,
        ));
    }

    if request_context.path().trim_end_matches('/') == RUNTIME_ROLLOUT_PREVIEW_PATH {
        return Ok(Some(
            build_runtime_rollout_preview_response(&state, &request_context).await?,
        ));
    }

    if request_context.path().trim_end_matches('/') == ERROR_RETURN_SETTINGS_PATH {
        return Ok(Some(
            build_error_return_settings_response(&state, &request_context, request.request_body())
                .await?,
        ));
    }

    if request_context.path().trim_end_matches('/') == BILLING_RESERVATIONS_PATH {
        return Ok(Some(
            build_billing_reservations_response(&state, &request_context).await?,
        ));
    }

    if request_context.path().trim_end_matches('/') == BILLING_RESERVATION_DRY_RUNS_PATH {
        return Ok(Some(
            build_billing_reservation_dry_runs_response(&state, &request_context).await?,
        ));
    }

    if request_context.path().trim_end_matches('/') == SETTLEMENT_SNAPSHOTS_PATH {
        return Ok(Some(
            build_settlement_snapshots_response(&state, &request_context).await?,
        ));
    }

    if let Some(ledger_id) = referral_reward_ledger_action_path_id(request_context.path(), "retry")
    {
        return Ok(Some(
            build_referral_reward_ledger_retry_response(
                &state,
                &request_context,
                request.request_body(),
                ledger_id,
            )
            .await?,
        ));
    }

    if let Some(ledger_id) = referral_reward_ledger_action_path_id(request_context.path(), "cancel")
    {
        return Ok(Some(
            build_referral_reward_ledger_cancel_response(
                &state,
                &request_context,
                request.request_body(),
                ledger_id,
            )
            .await?,
        ));
    }

    if request_context.path().trim_end_matches('/') == REFERRAL_REWARD_LEDGER_PATH {
        return Ok(Some(
            build_referral_reward_ledger_response(&state, &request_context).await?,
        ));
    }

    if request_context.path().trim_end_matches('/') == ROUTE_ATTEMPTS_PATH {
        return Ok(Some(
            build_route_attempts_response(&state, &request_context).await?,
        ));
    }

    if request_context.path().trim_end_matches('/') == CONSISTENCY_CHECKS_PATH {
        return Ok(Some(
            build_consistency_checks_response(&state, &request_context).await?,
        ));
    }

    Ok(None)
}

fn parse_recent_days(query_string: Option<&str>) -> u32 {
    query_param_value(query_string, "recent_days")
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| (1..=90).contains(value))
        .unwrap_or(7)
}

#[derive(Debug, Deserialize)]
struct AdminNifflerCreateUpstreamServiceRequest {
    display_name: String,
    service_kind: String,
    #[serde(default)]
    protocol_kind: Option<NifflerProtocolKind>,
    #[serde(default)]
    default_api_format: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default = "default_multiplier")]
    cost_multiplier: f64,
    #[serde(default = "default_true")]
    is_active: bool,
    #[serde(default)]
    capabilities: AdminNifflerServiceCapabilityRequest,
}

#[derive(Debug, Deserialize)]
struct AdminNifflerUpdateUpstreamServiceCapabilitiesRequest {
    protocol_kind: NifflerProtocolKind,
    #[serde(default)]
    capabilities: AdminNifflerServiceCapabilityRequest,
}

#[derive(Debug, Default, Deserialize)]
struct AdminNifflerServiceCapabilityRequest {
    #[serde(default = "default_true")]
    text: bool,
    #[serde(default = "default_true")]
    streaming: bool,
    #[serde(default)]
    images_endpoint: bool,
    #[serde(default)]
    openai_responses_image_tool: bool,
    #[serde(default = "default_true")]
    model_list: bool,
    #[serde(default = "default_true")]
    model_test: bool,
}

#[derive(Debug, Deserialize)]
struct AdminNifflerCreateUpstreamAccountRequest {
    display_name: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    phone: Option<String>,
    auth_kind: String,
    #[serde(default = "default_multiplier")]
    cost_multiplier: f64,
    #[serde(default)]
    priority: i32,
}

#[derive(Debug, Deserialize)]
struct AdminNifflerCreateProductPlanRequest {
    display_name: String,
    #[serde(default)]
    is_public: bool,
    #[serde(default = "default_true")]
    is_active: bool,
    #[serde(default = "default_multiplier")]
    sales_multiplier: f64,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdminNifflerUpsertProductPlanModelRequest {
    model_name: String,
    #[serde(default = "default_true")]
    is_enabled: bool,
    #[serde(default)]
    sales_multiplier_override: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct AdminNifflerUpsertApiKeyProductPlanBindingRequest {
    api_key_id: String,
}

#[derive(Debug, Deserialize)]
struct AdminNifflerUpsertRuntimeRolloutSettingRequest {
    target_scope: NifflerRuntimeRolloutTargetScope,
    target_id: String,
    #[serde(default)]
    enable_new_routing: bool,
    #[serde(default)]
    enable_settlement_snapshot: bool,
    #[serde(default)]
    enable_error_return_rules: bool,
    #[serde(default)]
    enable_billing_reservation: bool,
    #[serde(default)]
    enable_referral_ledger: bool,
    #[serde(default = "default_true")]
    is_active: bool,
}

#[derive(Debug, Deserialize)]
struct AdminNifflerCreateErrorReturnSettingRequest {
    scope: NifflerErrorResponseScope,
    #[serde(default)]
    upstream_service_id: Option<String>,
    #[serde(default)]
    match_status_code: Option<u16>,
    #[serde(default)]
    match_text: Option<String>,
    #[serde(default)]
    handling_step: Option<NifflerUpstreamErrorHandlingStep>,
    #[serde(default = "default_response_mode")]
    response_mode: NifflerUserResponseMode,
    user_message: String,
    #[serde(default = "default_account_protection_action")]
    account_protection_action: NifflerAccountProtectionAction,
    #[serde(default)]
    pause_duration: Option<NifflerPauseDuration>,
    #[serde(default = "default_true")]
    is_active: bool,
}

#[derive(Debug, Deserialize)]
struct AdminNifflerReferralLedgerMutationRequest {
    #[serde(default)]
    note: Option<String>,
}

#[derive(Debug, Serialize)]
struct NifflerLegacyDependencyAuditReport {
    schema_version: u32,
    generated_at_unix_secs: u64,
    offset: usize,
    limit: usize,
    has_more_user_keys: bool,
    summary: NifflerLegacyDependencyAuditSummary,
    user_key_legacy_restrictions: Vec<NifflerLegacyUserKeyRestriction>,
    user_group_legacy_policies: Vec<NifflerLegacyGroupPolicy>,
    provider_key_legacy_restrictions: Vec<NifflerKeyScopeResidue>,
    provider_model_price_dependencies: Vec<NifflerLegacyProviderModelPriceDependency>,
    legacy_write_entrypoints: Vec<NifflerLegacyCodeDependency>,
    runtime_read_dependencies: Vec<NifflerLegacyCodeDependency>,
    notes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct NifflerLegacyDependencyAuditSummary {
    user_key_restrictions_in_page: u64,
    user_group_policy_items: u64,
    provider_key_restriction_items: u64,
    provider_model_price_dependency_items: u64,
    legacy_write_entrypoints: u64,
    runtime_read_dependencies: u64,
}

#[derive(Debug, Serialize)]
struct NifflerLegacyUserKeyRestriction {
    key_id: String,
    key_name: Option<String>,
    owner_label: String,
    is_standalone: bool,
    group_id: Option<String>,
    group_name: Option<String>,
    field_names: Vec<String>,
    field_labels: Vec<String>,
    reason: String,
    impact: String,
    recommended_action: String,
}

#[derive(Debug, Serialize)]
struct NifflerLegacyGroupPolicy {
    group_id: String,
    group_name: String,
    field_name: String,
    field_label: String,
    mode: String,
    item_count: u64,
    reason: String,
    impact: String,
    recommended_action: String,
}

#[derive(Debug, Serialize)]
struct NifflerLegacyProviderModelPriceDependency {
    provider_id: String,
    provider_name: Option<String>,
    model_id: String,
    model_name: String,
    dependency_kind: String,
    dependency_label: String,
    reason: String,
    impact: String,
    recommended_action: String,
}

#[derive(Debug, Serialize)]
struct NifflerLegacyCodeDependency {
    area: String,
    label: String,
    method: Option<String>,
    path: String,
    current_status: String,
    reason: String,
    next_action: String,
}

fn default_multiplier() -> f64 {
    1.0
}

fn default_true() -> bool {
    true
}

fn default_response_mode() -> NifflerUserResponseMode {
    NifflerUserResponseMode::Replace
}

fn default_account_protection_action() -> NifflerAccountProtectionAction {
    NifflerAccountProtectionAction::RecordOnly
}

async fn build_upstream_services_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    request_body: Option<&axum::body::Bytes>,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() || !state.has_niffler_core_writer() {
        return Ok(niffler_data_unavailable_response());
    }

    match request_context.method() {
        method if method == http::Method::GET => {
            let query = NifflerUpstreamServiceListQuery {
                include_inactive: parse_bool_query(
                    request_context.query_string(),
                    "include_inactive",
                )
                .unwrap_or(false),
                search: query_param_value(request_context.query_string(), "search")
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty()),
                offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
                limit: parse_usize_query(request_context.query_string(), "limit").unwrap_or(50),
            };
            let page = state.list_niffler_upstream_services(&query).await?;
            Ok(Json(page).into_response())
        }
        method if method == http::Method::POST => {
            create_upstream_service_response(state, request_body).await
        }
        _ => Ok(niffler_method_not_allowed("只支持列表和创建上游服务")),
    }
}

async fn create_upstream_service_response(
    state: &AdminAppState<'_>,
    request_body: Option<&axum::body::Bytes>,
) -> Result<Response<Body>, GatewayError> {
    let payload =
        match parse_required_body::<AdminNifflerCreateUpstreamServiceRequest>(request_body) {
            Ok(payload) => payload,
            Err(response) => return Ok(response),
        };
    let display_name = match normalize_required_text(&payload.display_name, "服务名称", 200) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    let service_kind = match normalize_required_text(&payload.service_kind, "服务类型", 64) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    let default_api_format = match normalize_optional_text(payload.default_api_format, 64) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    let base_url = match normalize_optional_text(payload.base_url, 2_000) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    if !payload.cost_multiplier.is_finite() || payload.cost_multiplier < 0.0 {
        return Ok(niffler_bad_request("成本倍率必须是非负数字"));
    }

    let now_unix_ms = current_unix_secs().saturating_mul(1_000);
    let service_id = Uuid::new_v4().to_string();
    let protocol_kind = payload.protocol_kind.unwrap_or(NifflerProtocolKind::Openai);
    let record = CreateNifflerUpstreamServiceRecord {
        id: service_id.clone(),
        display_name,
        service_kind,
        default_api_format,
        base_url,
        cost_multiplier: payload.cost_multiplier,
        is_active: payload.is_active,
        config: Some(json!({
            "created_from": "niffler_core_admin",
            "credential_storage": "not_collected_in_first_slice",
            "protocol_kind": protocol_kind.as_str()
        })),
        created_at_unix_ms: now_unix_ms,
        updated_at_unix_ms: now_unix_ms,
    };
    let capability_records = match build_capability_records(
        &service_id,
        protocol_kind,
        payload.capabilities,
        now_unix_ms,
    ) {
        Ok(records) => records,
        Err(response) => return Ok(response),
    };

    let Some(created) = state.create_niffler_upstream_service(record).await? else {
        return Ok(niffler_data_unavailable_response());
    };
    for capability in capability_records {
        state
            .upsert_niffler_upstream_service_capability(capability)
            .await?;
    }

    Ok(attach_admin_audit_response(
        (http::StatusCode::CREATED, Json(created)).into_response(),
        "niffler_upstream_service_created",
        "create_niffler_upstream_service",
        "niffler_upstream_service",
        &service_id,
    ))
}

async fn build_upstream_service_capabilities_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    request_body: Option<&axum::body::Bytes>,
    upstream_service_id: String,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() || !state.has_niffler_core_writer() {
        return Ok(niffler_data_unavailable_response());
    }

    match request_context.method() {
        method if method == http::Method::GET => {
            if state
                .find_niffler_upstream_service_by_id(&upstream_service_id)
                .await?
                .is_none()
            {
                return Ok(niffler_not_found("上游服务不存在"));
            }
            list_upstream_service_capabilities_response(state, upstream_service_id).await
        }
        method if method == http::Method::PUT => {
            update_upstream_service_capabilities_response(state, request_body, upstream_service_id)
                .await
        }
        _ => Ok(niffler_method_not_allowed("只支持读取和保存服务能力")),
    }
}

async fn list_upstream_service_capabilities_response(
    state: &AdminAppState<'_>,
    upstream_service_id: String,
) -> Result<Response<Body>, GatewayError> {
    let query = NifflerUpstreamServiceCapabilityListQuery {
        upstream_service_id,
    };
    let page = state
        .list_niffler_upstream_service_capabilities(&query)
        .await?;
    Ok(Json(page).into_response())
}

async fn update_upstream_service_capabilities_response(
    state: &AdminAppState<'_>,
    request_body: Option<&axum::body::Bytes>,
    upstream_service_id: String,
) -> Result<Response<Body>, GatewayError> {
    if state
        .find_niffler_upstream_service_by_id(&upstream_service_id)
        .await?
        .is_none()
    {
        return Ok(niffler_not_found("上游服务不存在"));
    }
    let payload = match parse_required_body::<AdminNifflerUpdateUpstreamServiceCapabilitiesRequest>(
        request_body,
    ) {
        Ok(payload) => payload,
        Err(response) => return Ok(response),
    };
    let now_unix_ms = current_unix_secs().saturating_mul(1_000);
    let capability_records = match build_capability_records(
        &upstream_service_id,
        payload.protocol_kind,
        payload.capabilities,
        now_unix_ms,
    ) {
        Ok(records) => records,
        Err(response) => return Ok(response),
    };

    for capability in capability_records {
        state
            .upsert_niffler_upstream_service_capability(capability)
            .await?;
    }

    let response =
        list_upstream_service_capabilities_response(state, upstream_service_id.clone()).await?;
    Ok(attach_admin_audit_response(
        response,
        "niffler_upstream_service_capabilities_updated",
        "update_niffler_upstream_service_capabilities",
        "niffler_upstream_service",
        &upstream_service_id,
    ))
}

async fn build_upstream_accounts_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    request_body: Option<&axum::body::Bytes>,
    upstream_service_id: String,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() || !state.has_niffler_core_writer() {
        return Ok(niffler_data_unavailable_response());
    }

    match request_context.method() {
        method if method == http::Method::GET => {
            let status = match query_param_value(request_context.query_string(), "status")
                .as_deref()
                .map(NifflerAccountStatus::from_database)
                .transpose()
            {
                Ok(value) => value,
                Err(_) => return Ok(niffler_bad_request("账号状态不合法")),
            };
            let query = NifflerUpstreamAccountListQuery {
                upstream_service_id: Some(upstream_service_id),
                status,
                search: query_param_value(request_context.query_string(), "search")
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty()),
                offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
                limit: parse_usize_query(request_context.query_string(), "limit").unwrap_or(50),
            };
            let page = state.list_niffler_upstream_accounts(&query).await?;
            Ok(Json(page).into_response())
        }
        method if method == http::Method::POST => {
            create_upstream_account_response(state, request_body, &upstream_service_id).await
        }
        _ => Ok(niffler_method_not_allowed("只支持列表和创建上游账号")),
    }
}

async fn create_upstream_account_response(
    state: &AdminAppState<'_>,
    request_body: Option<&axum::body::Bytes>,
    upstream_service_id: &str,
) -> Result<Response<Body>, GatewayError> {
    if state
        .find_niffler_upstream_service_by_id(upstream_service_id)
        .await?
        .is_none()
    {
        return Ok(niffler_not_found("上游服务不存在"));
    }
    let payload =
        match parse_required_body::<AdminNifflerCreateUpstreamAccountRequest>(request_body) {
            Ok(payload) => payload,
            Err(response) => return Ok(response),
        };
    let display_name = match normalize_required_text(&payload.display_name, "账号名称", 200) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    let auth_kind = match normalize_required_text(&payload.auth_kind, "认证方式", 64) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    if !matches!(auth_kind.as_str(), "api_key" | "oauth" | "custom_header") {
        return Ok(niffler_bad_request(
            "认证方式只能是 api_key、oauth 或 custom_header",
        ));
    }
    if !payload.cost_multiplier.is_finite() || payload.cost_multiplier < 0.0 {
        return Ok(niffler_bad_request("成本倍率必须是非负数字"));
    }
    let email = match normalize_optional_text(payload.email, 320) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    let phone = match normalize_optional_text(payload.phone, 64) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    let now_unix_ms = current_unix_secs().saturating_mul(1_000);
    let account_id = Uuid::new_v4().to_string();
    let record = CreateNifflerUpstreamAccountRecord {
        id: account_id.clone(),
        upstream_service_id: upstream_service_id.to_string(),
        display_name,
        email,
        phone,
        auth_kind,
        status: NifflerAccountStatus::Available,
        cost_multiplier: payload.cost_multiplier,
        priority: payload.priority,
        cooldown_until_unix_ms: None,
        last_tested_at_unix_ms: None,
        last_test_error: None,
        config: Some(json!({
            "created_from": "niffler_core_admin",
            "credential_storage": "not_collected_in_first_slice"
        })),
        created_at_unix_ms: now_unix_ms,
        updated_at_unix_ms: now_unix_ms,
    };
    let Some(created) = state.create_niffler_upstream_account(record).await? else {
        return Ok(niffler_data_unavailable_response());
    };
    Ok(attach_admin_audit_response(
        (http::StatusCode::CREATED, Json(created)).into_response(),
        "niffler_upstream_account_created",
        "create_niffler_upstream_account",
        "niffler_upstream_account",
        &account_id,
    ))
}

async fn build_product_plans_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    request_body: Option<&axum::body::Bytes>,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() || !state.has_niffler_core_writer() {
        return Ok(niffler_data_unavailable_response());
    }

    match request_context.method() {
        method if method == http::Method::GET => {
            let query = NifflerProductPlanListQuery {
                include_inactive: parse_bool_query(
                    request_context.query_string(),
                    "include_inactive",
                )
                .unwrap_or(false),
                public_only: parse_bool_query(request_context.query_string(), "public_only")
                    .unwrap_or(false),
                search: query_param_value(request_context.query_string(), "search")
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty()),
                offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
                limit: parse_usize_query(request_context.query_string(), "limit").unwrap_or(50),
            };
            let page = state.list_niffler_product_plans(&query).await?;
            Ok(Json(page).into_response())
        }
        method if method == http::Method::POST => {
            create_product_plan_response(state, request_body).await
        }
        _ => Ok(niffler_method_not_allowed("只支持列表和创建产品策略")),
    }
}

async fn create_product_plan_response(
    state: &AdminAppState<'_>,
    request_body: Option<&axum::body::Bytes>,
) -> Result<Response<Body>, GatewayError> {
    let payload = match parse_required_body::<AdminNifflerCreateProductPlanRequest>(request_body) {
        Ok(payload) => payload,
        Err(response) => return Ok(response),
    };
    let display_name = match normalize_required_text(&payload.display_name, "产品策略名称", 200)
    {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    if !payload.sales_multiplier.is_finite() || payload.sales_multiplier < 0.0 {
        return Ok(niffler_bad_request("销售倍率必须是非负数字"));
    }
    let description = match normalize_optional_text(payload.description, 2_000) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };

    let now_unix_ms = current_unix_secs().saturating_mul(1_000);
    let product_plan_id = Uuid::new_v4().to_string();
    let record = CreateNifflerProductPlanRecord {
        id: product_plan_id.clone(),
        display_name,
        is_public: payload.is_public,
        is_active: payload.is_active,
        sales_multiplier: payload.sales_multiplier,
        description,
        created_at_unix_ms: now_unix_ms,
        updated_at_unix_ms: now_unix_ms,
    };
    let Some(created) = state.create_niffler_product_plan(record).await? else {
        return Ok(niffler_data_unavailable_response());
    };
    Ok(attach_admin_audit_response(
        (http::StatusCode::CREATED, Json(created)).into_response(),
        "niffler_product_plan_created",
        "create_niffler_product_plan",
        "niffler_product_plan",
        &product_plan_id,
    ))
}

async fn build_product_plan_models_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    request_body: Option<&axum::body::Bytes>,
    product_plan_id: String,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() || !state.has_niffler_core_writer() {
        return Ok(niffler_data_unavailable_response());
    }
    if state
        .find_niffler_product_plan_by_id(&product_plan_id)
        .await?
        .is_none()
    {
        return Ok(niffler_not_found("产品策略不存在"));
    }

    match request_context.method() {
        method if method == http::Method::GET => {
            let query = NifflerProductPlanModelListQuery {
                product_plan_id,
                enabled_only: parse_bool_query(request_context.query_string(), "enabled_only")
                    .unwrap_or(false),
                search: query_param_value(request_context.query_string(), "search")
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty()),
                offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
                limit: parse_usize_query(request_context.query_string(), "limit").unwrap_or(100),
            };
            let page = state.list_niffler_product_plan_models(&query).await?;
            Ok(Json(page).into_response())
        }
        method if method == http::Method::POST => {
            upsert_product_plan_model_response(state, request_body, &product_plan_id).await
        }
        _ => Ok(niffler_method_not_allowed("只支持列表和保存可售模型")),
    }
}

async fn upsert_product_plan_model_response(
    state: &AdminAppState<'_>,
    request_body: Option<&axum::body::Bytes>,
    product_plan_id: &str,
) -> Result<Response<Body>, GatewayError> {
    let payload =
        match parse_required_body::<AdminNifflerUpsertProductPlanModelRequest>(request_body) {
            Ok(payload) => payload,
            Err(response) => return Ok(response),
        };
    let model_name = match normalize_required_text(&payload.model_name, "模型名称", 200) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    if let Some(value) = payload.sales_multiplier_override {
        if !value.is_finite() || value < 0.0 {
            return Ok(niffler_bad_request("模型销售倍率覆盖必须是非负数字"));
        }
    }

    let now_unix_ms = current_unix_secs().saturating_mul(1_000);
    let model_record_id = Uuid::new_v4().to_string();
    let record = UpsertNifflerProductPlanModelRecord {
        id: model_record_id.clone(),
        product_plan_id: product_plan_id.to_string(),
        model_name,
        is_enabled: payload.is_enabled,
        sales_multiplier_override: payload.sales_multiplier_override,
        created_at_unix_ms: now_unix_ms,
        updated_at_unix_ms: now_unix_ms,
    };
    let Some(saved) = state.upsert_niffler_product_plan_model(record).await? else {
        return Ok(niffler_data_unavailable_response());
    };
    let saved_id = saved.id.clone();
    Ok(attach_admin_audit_response(
        Json(saved).into_response(),
        "niffler_product_plan_model_saved",
        "upsert_niffler_product_plan_model",
        "niffler_product_plan_model",
        &saved_id,
    ))
}

async fn build_product_plan_api_key_bindings_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    request_body: Option<&axum::body::Bytes>,
    product_plan_id: String,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() || !state.has_niffler_core_writer() {
        return Ok(niffler_data_unavailable_response());
    }
    let Some(product_plan) = state
        .find_niffler_product_plan_by_id(&product_plan_id)
        .await?
    else {
        return Ok(niffler_not_found("产品策略不存在"));
    };

    match request_context.method() {
        method if method == http::Method::GET => {
            let query = NifflerApiKeyProductPlanBindingListQuery {
                product_plan_id: Some(product_plan_id),
                offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
                limit: parse_usize_query(request_context.query_string(), "limit").unwrap_or(100),
            };
            let page = state
                .list_niffler_api_key_product_plan_bindings(&query)
                .await?;
            Ok(Json(page).into_response())
        }
        method if method == http::Method::POST => {
            if !product_plan.is_active {
                return Ok(niffler_bad_request("只能绑定启用的产品策略"));
            }
            upsert_product_plan_api_key_binding_response(state, request_body, &product_plan_id)
                .await
        }
        _ => Ok(niffler_method_not_allowed("只支持列表和绑定 Key")),
    }
}

async fn build_all_api_key_product_plan_bindings_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() || !state.has_niffler_core_writer() {
        return Ok(niffler_data_unavailable_response());
    }
    if request_context.method() != http::Method::GET {
        return Ok(niffler_method_not_allowed("只支持读取 Key 绑定"));
    }
    let query = NifflerApiKeyProductPlanBindingListQuery {
        product_plan_id: None,
        offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
        limit: parse_usize_query(request_context.query_string(), "limit").unwrap_or(200),
    };
    let page = state
        .list_niffler_api_key_product_plan_bindings(&query)
        .await?;
    Ok(Json(page).into_response())
}

async fn upsert_product_plan_api_key_binding_response(
    state: &AdminAppState<'_>,
    request_body: Option<&axum::body::Bytes>,
    product_plan_id: &str,
) -> Result<Response<Body>, GatewayError> {
    let payload = match parse_required_body::<AdminNifflerUpsertApiKeyProductPlanBindingRequest>(
        request_body,
    ) {
        Ok(payload) => payload,
        Err(response) => return Ok(response),
    };
    let api_key_id = match normalize_required_text(&payload.api_key_id, "API Key", 64) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    let snapshots = state
        .list_auth_api_key_snapshots_by_ids(std::slice::from_ref(&api_key_id))
        .await?;
    let Some(snapshot) = snapshots
        .iter()
        .find(|snapshot| snapshot.api_key_id == api_key_id)
    else {
        return Ok(niffler_not_found("API Key 不存在"));
    };
    if !snapshot.api_key_is_standalone {
        return Ok(niffler_bad_request(
            "只能绑定独立 API Key，旧分组 Key 暂不接入影子产品策略",
        ));
    }

    let now_unix_ms = current_unix_secs().saturating_mul(1_000);
    let record = UpsertNifflerApiKeyProductPlanBindingRecord {
        id: Uuid::new_v4().to_string(),
        api_key_id: api_key_id.clone(),
        product_plan_id: product_plan_id.to_string(),
        config: Some(json!({
            "created_from": "niffler_core_admin",
            "runtime_effect": "shadow_only"
        })),
        created_at_unix_ms: now_unix_ms,
        updated_at_unix_ms: now_unix_ms,
    };
    let Some(saved) = state
        .upsert_niffler_api_key_product_plan_binding(record)
        .await?
    else {
        return Ok(niffler_data_unavailable_response());
    };
    let saved_id = saved.id.clone();
    Ok(attach_admin_audit_response(
        Json(saved).into_response(),
        "niffler_api_key_product_plan_binding_saved",
        "upsert_niffler_api_key_product_plan_binding",
        "niffler_api_key_product_plan_binding",
        &saved_id,
    ))
}

async fn build_runtime_rollout_settings_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    request_body: Option<&axum::body::Bytes>,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() || !state.has_niffler_core_writer() {
        return Ok(niffler_data_unavailable_response());
    }

    match request_context.method() {
        method if method == http::Method::GET => {
            let target_scope =
                match parse_runtime_rollout_target_scope_query(request_context.query_string()) {
                    Ok(value) => value,
                    Err(response) => return Ok(response),
                };
            let query = NifflerRuntimeRolloutSettingListQuery {
                target_scope,
                include_inactive: parse_bool_query(
                    request_context.query_string(),
                    "include_inactive",
                )
                .unwrap_or(false),
                offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
                limit: parse_usize_query(request_context.query_string(), "limit").unwrap_or(50),
            };
            let page = state.list_niffler_runtime_rollout_settings(&query).await?;
            Ok(Json(page).into_response())
        }
        method if method == http::Method::POST => {
            upsert_runtime_rollout_setting_response(state, request_body).await
        }
        _ => Ok(niffler_method_not_allowed("只支持列表和保存灰度开关")),
    }
}

async fn build_runtime_rollout_preview_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() {
        return Ok(niffler_data_unavailable_response());
    }
    if request_context.method() != http::Method::GET {
        return Ok(niffler_method_not_allowed("只支持只读预览"));
    }

    let api_key_id = match query_param_value(request_context.query_string(), "api_key_id") {
        Some(value) => match normalize_required_text(&value, "API Key", 64) {
            Ok(value) => value,
            Err(response) => return Ok(response),
        },
        None => return Ok(niffler_bad_request("API Key 不能为空")),
    };
    let snapshots = state
        .list_auth_api_key_snapshots_by_ids(std::slice::from_ref(&api_key_id))
        .await?;
    let Some(snapshot) = snapshots
        .iter()
        .find(|snapshot| snapshot.api_key_id == api_key_id)
    else {
        return Ok(niffler_not_found("API Key 不存在"));
    };

    let key_setting = state
        .find_niffler_runtime_rollout_setting(NifflerRuntimeRolloutTargetScope::ApiKey, &api_key_id)
        .await?;
    let binding = state
        .find_niffler_api_key_product_plan_binding_by_api_key_id(&api_key_id)
        .await?;

    let mut warnings = Vec::new();
    let mut product_plan_payload = serde_json::Value::Null;
    let mut product_plan_label = None;
    let mut product_plan_can_apply = false;
    let mut product_plan_setting = None;

    if let Some(binding) = binding.as_ref() {
        match state
            .find_niffler_product_plan_by_id(&binding.product_plan_id)
            .await?
        {
            Some(product_plan) => {
                product_plan_label = Some(product_plan.display_name.clone());
                product_plan_can_apply = product_plan.is_active;
                if !product_plan.is_active {
                    warnings.push("影子产品策略已停用，策略级灰度开关不会生效。".to_string());
                }
                product_plan_payload = json!({
                    "id": product_plan.id,
                    "display_name": product_plan.display_name,
                    "is_active": product_plan.is_active,
                    "binding_id": binding.id,
                    "binding_updated_at_unix_ms": binding.updated_at_unix_ms,
                });
                product_plan_setting = state
                    .find_niffler_runtime_rollout_setting(
                        NifflerRuntimeRolloutTargetScope::ProductPlan,
                        &binding.product_plan_id,
                    )
                    .await?;
            }
            None => {
                warnings.push("影子绑定引用的产品策略不存在，策略级灰度开关不会生效。".to_string());
                product_plan_payload = json!({
                    "id": binding.product_plan_id,
                    "display_name": null,
                    "is_active": false,
                    "binding_id": binding.id,
                    "binding_updated_at_unix_ms": binding.updated_at_unix_ms,
                });
            }
        }
    } else {
        warnings.push("这把 Key 还没有绑定影子产品策略。".to_string());
    }

    if key_setting
        .as_ref()
        .is_some_and(|setting| !setting.is_active)
    {
        warnings.push("Key 级灰度开关已停用，预览会继续检查产品策略级开关。".to_string());
    }
    if product_plan_setting
        .as_ref()
        .is_some_and(|setting| !setting.is_active)
    {
        warnings.push("产品策略级灰度开关已停用。".to_string());
    }

    let key_can_enter_runtime = snapshot.api_key_is_active
        && !snapshot.api_key_is_locked
        && snapshot.user_is_active
        && !snapshot.user_is_deleted;
    if !snapshot.user_is_active {
        warnings.push("用户已停用，这把 Key 不会进入灰度运行时。".to_string());
    }
    if snapshot.user_is_deleted {
        warnings.push("用户已删除，这把 Key 不会进入灰度运行时。".to_string());
    }
    if !snapshot.api_key_is_active {
        warnings.push("API Key 已停用，不会进入灰度运行时。".to_string());
    }
    if snapshot.api_key_is_locked {
        warnings.push("API Key 已锁定，不会进入灰度运行时。".to_string());
    }

    let active_key_setting = key_setting
        .as_ref()
        .filter(|setting| setting.is_active && key_can_enter_runtime);
    let active_product_plan_setting = product_plan_setting
        .as_ref()
        .filter(|setting| setting.is_active && product_plan_can_apply && key_can_enter_runtime);

    let (source_scope, source_label, effective_setting, reason) =
        if let Some(setting) = active_key_setting {
            (
                Some("api_key"),
                Some("Key 级灰度开关".to_string()),
                Some(setting),
                "命中 Key 级灰度开关；后续接入运行时时会优先使用这条配置。".to_string(),
            )
        } else if let Some(setting) = active_product_plan_setting {
            (
                Some("product_plan"),
                Some(format!(
                    "产品策略：{}",
                    product_plan_label.as_deref().unwrap_or(&setting.target_id)
                )),
                Some(setting),
                "Key 没有启用的 Key 级开关，命中产品策略级灰度开关。".to_string(),
            )
        } else if !key_can_enter_runtime {
            (
                None,
                None,
                None,
                "Key 或用户当前不可用，后续运行时不会启用任何新链路。".to_string(),
            )
        } else if binding.is_none() {
            (
                None,
                None,
                None,
                "Key 未绑定影子产品策略，也没有启用的 Key 级开关。".to_string(),
            )
        } else {
            (
                None,
                None,
                None,
                "没有启用的 Key 级或产品策略级灰度开关。".to_string(),
            )
        };

    let payload = json!({
        "api_key": {
            "id": snapshot.api_key_id,
            "name": snapshot.api_key_name,
            "owner_label": snapshot.email.as_deref().unwrap_or(&snapshot.username),
            "user_id": snapshot.user_id,
            "user_is_active": snapshot.user_is_active,
            "user_is_deleted": snapshot.user_is_deleted,
            "is_active": snapshot.api_key_is_active,
            "is_locked": snapshot.api_key_is_locked,
            "is_standalone": snapshot.api_key_is_standalone,
        },
        "product_plan": product_plan_payload,
        "key_setting": key_setting.clone(),
        "product_plan_setting": product_plan_setting.clone(),
        "decision": {
            "runtime_effect": "preview_only",
            "source_scope": source_scope,
            "source_label": source_label,
            "reason": reason,
            "is_active": effective_setting.is_some(),
            "enable_new_routing": effective_setting
                .is_some_and(|setting| setting.enable_new_routing),
            "enable_settlement_snapshot": effective_setting
                .is_some_and(|setting| setting.enable_settlement_snapshot),
            "enable_error_return_rules": effective_setting
                .is_some_and(|setting| setting.enable_error_return_rules),
            "enable_billing_reservation": effective_setting
                .is_some_and(|setting| setting.enable_billing_reservation),
            "enable_referral_ledger": effective_setting
                .is_some_and(|setting| setting.enable_referral_ledger),
        },
        "warnings": warnings,
    });
    Ok(Json(payload).into_response())
}

async fn upsert_runtime_rollout_setting_response(
    state: &AdminAppState<'_>,
    request_body: Option<&axum::body::Bytes>,
) -> Result<Response<Body>, GatewayError> {
    let payload =
        match parse_required_body::<AdminNifflerUpsertRuntimeRolloutSettingRequest>(request_body) {
            Ok(payload) => payload,
            Err(response) => return Ok(response),
        };
    let target_id = match normalize_required_text(&payload.target_id, "灰度对象", 64) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    if let Err(response) =
        validate_runtime_rollout_target(state, payload.target_scope, &target_id).await
    {
        return Ok(response);
    }

    let now_unix_ms = current_unix_secs().saturating_mul(1_000);
    let record = UpsertNifflerRuntimeRolloutSettingRecord {
        id: Uuid::new_v4().to_string(),
        target_scope: payload.target_scope,
        target_id: target_id.clone(),
        enable_new_routing: payload.enable_new_routing,
        enable_settlement_snapshot: payload.enable_settlement_snapshot,
        enable_error_return_rules: payload.enable_error_return_rules,
        enable_billing_reservation: payload.enable_billing_reservation,
        enable_referral_ledger: payload.enable_referral_ledger,
        is_active: payload.is_active,
        config: Some(json!({
            "created_from": "niffler_core_admin",
            "runtime_effect": "shadow_only"
        })),
        created_at_unix_ms: now_unix_ms,
        updated_at_unix_ms: now_unix_ms,
    };
    let Some(saved) = state.upsert_niffler_runtime_rollout_setting(record).await? else {
        return Ok(niffler_data_unavailable_response());
    };
    let saved_id = saved.id.clone();
    Ok(attach_admin_audit_response(
        Json(saved).into_response(),
        "niffler_runtime_rollout_setting_saved",
        "upsert_niffler_runtime_rollout_setting",
        "niffler_runtime_rollout_setting",
        &saved_id,
    ))
}

async fn validate_runtime_rollout_target(
    state: &AdminAppState<'_>,
    target_scope: NifflerRuntimeRolloutTargetScope,
    target_id: &str,
) -> Result<(), Response<Body>> {
    match target_scope {
        NifflerRuntimeRolloutTargetScope::ApiKey => {
            let snapshots = state
                .list_auth_api_key_snapshots_by_ids(&[target_id.to_string()])
                .await
                .map_err(|err| niffler_internal_error(format!("{err:?}")))?;
            let Some(snapshot) = snapshots
                .iter()
                .find(|snapshot| snapshot.api_key_id == target_id)
            else {
                return Err(niffler_not_found("API Key 不存在"));
            };
            if !snapshot.api_key_is_active {
                return Err(niffler_bad_request("只能给启用的 API Key 设置灰度开关"));
            }
        }
        NifflerRuntimeRolloutTargetScope::ProductPlan => {
            let Some(product_plan) = state
                .find_niffler_product_plan_by_id(target_id)
                .await
                .map_err(|err| niffler_internal_error(format!("{err:?}")))?
            else {
                return Err(niffler_not_found("产品策略不存在"));
            };
            if !product_plan.is_active {
                return Err(niffler_bad_request("只能给启用的产品策略设置灰度开关"));
            }
        }
    }
    Ok(())
}

async fn build_error_return_settings_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    request_body: Option<&axum::body::Bytes>,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() || !state.has_niffler_core_writer() {
        return Ok(niffler_data_unavailable_response());
    }

    match request_context.method() {
        method if method == http::Method::GET => {
            let query = NifflerErrorReturnSettingListQuery {
                scope: parse_error_response_scope_query(request_context.query_string(), "scope"),
                upstream_service_id: query_param_value(
                    request_context.query_string(),
                    "upstream_service_id",
                )
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
                include_inactive: parse_bool_query(
                    request_context.query_string(),
                    "include_inactive",
                )
                .unwrap_or(false),
                offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
                limit: parse_usize_query(request_context.query_string(), "limit").unwrap_or(50),
            };
            let page = state.list_niffler_error_return_settings(&query).await?;
            Ok(Json(page).into_response())
        }
        method if method == http::Method::POST => {
            create_error_return_setting_response(state, request_body).await
        }
        _ => Ok(niffler_method_not_allowed("只支持列表和创建错误文案规则")),
    }
}

async fn create_error_return_setting_response(
    state: &AdminAppState<'_>,
    request_body: Option<&axum::body::Bytes>,
) -> Result<Response<Body>, GatewayError> {
    let payload =
        match parse_required_body::<AdminNifflerCreateErrorReturnSettingRequest>(request_body) {
            Ok(payload) => payload,
            Err(response) => return Ok(response),
        };
    let user_message =
        match normalize_required_text(&payload.user_message, "返回给用户的文案", 2_000) {
            Ok(value) => value,
            Err(response) => return Ok(response),
        };
    let match_text = match normalize_optional_text(payload.match_text, 2_000) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    if payload
        .match_status_code
        .is_some_and(|code| !(100..=599).contains(&code))
    {
        return Ok(niffler_bad_request("状态码必须在 100 到 599 之间"));
    }
    if payload.scope == NifflerErrorResponseScope::Upstream && payload.handling_step.is_none() {
        return Ok(niffler_bad_request("上游级规则必须选择处理类型"));
    }
    if payload.scope == NifflerErrorResponseScope::Platform
        && payload.account_protection_action != NifflerAccountProtectionAction::RecordOnly
    {
        return Ok(niffler_bad_request("平台级规则不能触发上游账号保护"));
    }
    if payload.account_protection_action == NifflerAccountProtectionAction::PauseScheduling
        && payload.pause_duration.is_none()
    {
        return Ok(niffler_bad_request("暂停调度必须选择暂停时长"));
    }

    let upstream_service_id = if payload.scope == NifflerErrorResponseScope::Upstream {
        match normalize_optional_text(payload.upstream_service_id, 36) {
            Ok(value) => value,
            Err(response) => return Ok(response),
        }
    } else {
        None
    };
    if let Some(service_id) = upstream_service_id.as_deref() {
        if state
            .find_niffler_upstream_service_by_id(service_id)
            .await?
            .is_none()
        {
            return Ok(niffler_not_found("上游服务不存在"));
        }
    }

    let now_unix_ms = current_unix_secs().saturating_mul(1_000);
    let setting_id = Uuid::new_v4().to_string();
    let record = CreateNifflerErrorReturnSettingRecord {
        id: setting_id.clone(),
        scope: payload.scope,
        upstream_service_id,
        match_status_code: payload.match_status_code,
        match_text,
        handling_step: if payload.scope == NifflerErrorResponseScope::Upstream {
            payload.handling_step
        } else {
            None
        },
        response_mode: payload.response_mode,
        user_message,
        account_protection_action: if payload.scope == NifflerErrorResponseScope::Upstream {
            payload.account_protection_action
        } else {
            NifflerAccountProtectionAction::RecordOnly
        },
        pause_duration: if payload.account_protection_action
            == NifflerAccountProtectionAction::PauseScheduling
        {
            payload.pause_duration
        } else {
            None
        },
        is_active: payload.is_active,
        created_at_unix_ms: now_unix_ms,
        updated_at_unix_ms: now_unix_ms,
    };
    let Some(created) = state.create_niffler_error_return_setting(record).await? else {
        return Ok(niffler_data_unavailable_response());
    };
    Ok(attach_admin_audit_response(
        (http::StatusCode::CREATED, Json(created)).into_response(),
        "niffler_error_return_setting_created",
        "create_niffler_error_return_setting",
        "niffler_error_return_setting",
        &setting_id,
    ))
}

async fn build_billing_reservations_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() {
        return Ok(niffler_data_unavailable_response());
    }
    if request_context.method() != http::Method::GET {
        return Ok(niffler_method_not_allowed("只支持读取计费预占"));
    }
    let status = match parse_billing_reservation_status_query(request_context.query_string()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    let query = NifflerBillingReservationListQuery {
        status,
        user_id: optional_query_text(request_context.query_string(), "user_id"),
        api_key_id: optional_query_text(request_context.query_string(), "api_key_id"),
        request_id: optional_query_text(request_context.query_string(), "request_id"),
        expires_at_lte_unix_ms: None,
        offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
        limit: parse_usize_query(request_context.query_string(), "limit").unwrap_or(50),
    };
    let page = state.list_niffler_billing_reservations(&query).await?;
    Ok(Json(page).into_response())
}

async fn build_billing_reservation_dry_runs_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() {
        return Ok(niffler_data_unavailable_response());
    }
    if request_context.method() != http::Method::GET {
        return Ok(niffler_method_not_allowed("只支持读取预占干跑"));
    }
    let query = NifflerBillingReservationDryRunListQuery {
        status: optional_query_text(request_context.query_string(), "status"),
        user_id: optional_query_text(request_context.query_string(), "user_id"),
        api_key_id: optional_query_text(request_context.query_string(), "api_key_id"),
        product_plan_id: optional_query_text(request_context.query_string(), "product_plan_id"),
        request_id: optional_query_text(request_context.query_string(), "request_id"),
        offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
        limit: parse_usize_query(request_context.query_string(), "limit")
            .unwrap_or(50)
            .clamp(1, 100),
    };
    let page = state
        .list_niffler_billing_reservation_dry_runs(&query)
        .await?;
    Ok(Json(page).into_response())
}

async fn build_settlement_snapshots_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() {
        return Ok(niffler_data_unavailable_response());
    }
    if request_context.method() != http::Method::GET {
        return Ok(niffler_method_not_allowed("只支持读取结算快照"));
    }
    let query = NifflerSettlementSnapshotListQuery {
        request_id: optional_query_text(request_context.query_string(), "request_id"),
        user_id: optional_query_text(request_context.query_string(), "user_id"),
        api_key_id: optional_query_text(request_context.query_string(), "api_key_id"),
        product_plan_id: optional_query_text(request_context.query_string(), "product_plan_id"),
        offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
        limit: parse_usize_query(request_context.query_string(), "limit")
            .unwrap_or(50)
            .clamp(1, 100),
    };
    let page = state.list_niffler_settlement_snapshots(&query).await?;
    Ok(Json(page).into_response())
}

async fn build_referral_reward_ledger_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() {
        return Ok(niffler_data_unavailable_response());
    }
    if request_context.method() != http::Method::GET {
        return Ok(niffler_method_not_allowed("只支持读取返利流水"));
    }
    let status = match parse_referral_reward_ledger_status_query(request_context.query_string()) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    let query = NifflerReferralRewardLedgerListQuery {
        status,
        inviter_user_id: optional_query_text(request_context.query_string(), "inviter_user_id"),
        invitee_user_id: optional_query_text(request_context.query_string(), "invitee_user_id"),
        order_id: optional_query_text(request_context.query_string(), "order_id"),
        offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
        limit: parse_usize_query(request_context.query_string(), "limit").unwrap_or(50),
    };
    let page = state.list_niffler_referral_reward_ledger(&query).await?;
    Ok(Json(page).into_response())
}

async fn build_referral_reward_ledger_retry_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    request_body: Option<&axum::body::Bytes>,
    ledger_id: String,
) -> Result<Response<Body>, GatewayError> {
    if request_context.method() != http::Method::POST {
        return Ok(niffler_method_not_allowed("只支持重试返利流水"));
    }
    let note = match parse_referral_ledger_mutation_note(request_body) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    match state
        .app()
        .retry_niffler_referral_reward_ledger(
            &ledger_id,
            niffler_admin_operator_id(request_context).as_deref(),
            note.as_deref(),
        )
        .await?
    {
        crate::data::state::ReferralMutationStatus::Applied => Ok(attach_admin_audit_response(
            Json(json!({ "status": "applied" })).into_response(),
            "admin_niffler_referral_ledger_retry",
            "retry_niffler_referral_reward_ledger",
            "niffler_referral_reward_ledger",
            &ledger_id,
        )),
        crate::data::state::ReferralMutationStatus::NotFound => {
            Ok(niffler_not_found("返利流水不存在"))
        }
        crate::data::state::ReferralMutationStatus::Invalid => {
            Ok(niffler_bad_request("当前返利流水不能重试"))
        }
        crate::data::state::ReferralMutationStatus::Unavailable => {
            Ok(niffler_data_unavailable_response())
        }
    }
}

async fn build_referral_reward_ledger_cancel_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    request_body: Option<&axum::body::Bytes>,
    ledger_id: String,
) -> Result<Response<Body>, GatewayError> {
    if request_context.method() != http::Method::POST {
        return Ok(niffler_method_not_allowed("只支持取消返利流水"));
    }
    let note = match parse_referral_ledger_mutation_note(request_body) {
        Ok(value) => value,
        Err(response) => return Ok(response),
    };
    match state
        .app()
        .cancel_niffler_referral_reward_ledger(
            &ledger_id,
            niffler_admin_operator_id(request_context).as_deref(),
            note.as_deref(),
        )
        .await?
    {
        crate::data::state::ReferralMutationStatus::Applied => Ok(attach_admin_audit_response(
            Json(json!({ "status": "cancelled" })).into_response(),
            "admin_niffler_referral_ledger_cancel",
            "cancel_niffler_referral_reward_ledger",
            "niffler_referral_reward_ledger",
            &ledger_id,
        )),
        crate::data::state::ReferralMutationStatus::NotFound => {
            Ok(niffler_not_found("返利流水不存在"))
        }
        crate::data::state::ReferralMutationStatus::Invalid => {
            Ok(niffler_bad_request("当前返利流水不能取消"))
        }
        crate::data::state::ReferralMutationStatus::Unavailable => {
            Ok(niffler_data_unavailable_response())
        }
    }
}

async fn build_route_attempts_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() {
        return Ok(niffler_data_unavailable_response());
    }
    if request_context.method() != http::Method::GET {
        return Ok(niffler_method_not_allowed("只支持读取影子路由尝试"));
    }
    let query = NifflerRouteAttemptListQuery {
        request_id: optional_query_text(request_context.query_string(), "request_id"),
        upstream_service_id: optional_query_text(
            request_context.query_string(),
            "upstream_service_id",
        ),
        upstream_account_id: optional_query_text(
            request_context.query_string(),
            "upstream_account_id",
        ),
        status: optional_query_text(request_context.query_string(), "status"),
        offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
        limit: parse_usize_query(request_context.query_string(), "limit")
            .unwrap_or(50)
            .clamp(1, 100),
    };
    let page = state.list_niffler_route_attempts(&query).await?;
    Ok(Json(page).into_response())
}

async fn build_consistency_checks_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    if !state.has_niffler_core_reader() {
        return Ok(niffler_data_unavailable_response());
    }
    if request_context.method() != http::Method::GET {
        return Ok(niffler_method_not_allowed("只支持读取一致性看板"));
    }
    let query = NifflerConsistencyCheckListQuery {
        request_id: optional_query_text(request_context.query_string(), "request_id"),
        user_id: optional_query_text(request_context.query_string(), "user_id"),
        api_key_id: optional_query_text(request_context.query_string(), "api_key_id"),
        product_plan_id: optional_query_text(request_context.query_string(), "product_plan_id"),
        offset: parse_usize_query(request_context.query_string(), "offset").unwrap_or(0),
        limit: parse_usize_query(request_context.query_string(), "limit")
            .unwrap_or(50)
            .clamp(1, 100),
    };
    let page = state.list_niffler_consistency_checks(&query).await?;
    Ok(Json(page).into_response())
}

fn build_capability_records(
    upstream_service_id: &str,
    protocol_kind: NifflerProtocolKind,
    payload: AdminNifflerServiceCapabilityRequest,
    now_unix_ms: u64,
) -> Result<Vec<UpsertNifflerUpstreamServiceCapabilityRecord>, Response<Body>> {
    let capabilities = [
        (NifflerServiceCapabilityKind::Text, payload.text),
        (NifflerServiceCapabilityKind::Streaming, payload.streaming),
        (
            NifflerServiceCapabilityKind::ImagesEndpoint,
            payload.images_endpoint,
        ),
        (
            NifflerServiceCapabilityKind::OpenaiResponsesImageTool,
            payload.openai_responses_image_tool,
        ),
        (NifflerServiceCapabilityKind::ModelList, payload.model_list),
        (NifflerServiceCapabilityKind::ModelTest, payload.model_test),
    ];
    capabilities
        .into_iter()
        .map(|(capability_kind, is_enabled)| {
            let record = UpsertNifflerUpstreamServiceCapabilityRecord {
                id: Uuid::new_v4().to_string(),
                upstream_service_id: upstream_service_id.to_string(),
                protocol_kind,
                capability_kind,
                is_enabled,
                config: None,
                created_at_unix_ms: now_unix_ms,
                updated_at_unix_ms: now_unix_ms,
            };
            record
                .validate()
                .map(|_| record)
                .map_err(|_| niffler_bad_request("对话内图片工具只允许 OpenAI 或 Codex 协议启用"))
        })
        .collect()
}

fn upstream_service_accounts_path_id(path: &str) -> Option<String> {
    let rest = path
        .trim_end_matches('/')
        .strip_prefix("/api/admin/niffler-core/upstream-services/")?;
    let id = rest.strip_suffix("/accounts")?;
    (!id.is_empty() && !id.contains('/')).then_some(id.to_string())
}

fn upstream_service_capabilities_path_id(path: &str) -> Option<String> {
    let rest = path
        .trim_end_matches('/')
        .strip_prefix("/api/admin/niffler-core/upstream-services/")?;
    let id = rest.strip_suffix("/capabilities")?;
    (!id.is_empty() && !id.contains('/')).then_some(id.to_string())
}

fn product_plan_models_path_id(path: &str) -> Option<String> {
    let rest = path
        .trim_end_matches('/')
        .strip_prefix("/api/admin/niffler-core/product-plans/")?;
    let id = rest.strip_suffix("/models")?;
    (!id.is_empty() && !id.contains('/')).then_some(id.to_string())
}

fn product_plan_api_key_bindings_path_id(path: &str) -> Option<String> {
    let rest = path
        .trim_end_matches('/')
        .strip_prefix("/api/admin/niffler-core/product-plans/")?;
    let id = rest.strip_suffix("/api-key-bindings")?;
    (!id.is_empty() && !id.contains('/')).then_some(id.to_string())
}

fn referral_reward_ledger_action_path_id(path: &str, action: &str) -> Option<String> {
    let rest = path
        .trim_end_matches('/')
        .strip_prefix("/api/admin/niffler-core/referral-reward-ledger/")?;
    let suffix = format!("/{action}");
    let id = rest.strip_suffix(&suffix)?;
    (!id.is_empty() && !id.contains('/')).then_some(id.to_string())
}

fn parse_required_body<T: for<'de> Deserialize<'de>>(
    request_body: Option<&axum::body::Bytes>,
) -> Result<T, Response<Body>> {
    let Some(request_body) = request_body.filter(|body| !body.is_empty()) else {
        return Err(niffler_bad_request("请求体不能为空"));
    };
    serde_json::from_slice(request_body)
        .map_err(|err| niffler_bad_request(format!("请求体不是合法 JSON：{err}")))
}

fn parse_referral_ledger_mutation_note(
    request_body: Option<&axum::body::Bytes>,
) -> Result<Option<String>, Response<Body>> {
    let Some(request_body) = request_body.filter(|body| !body.is_empty()) else {
        return Ok(None);
    };
    let payload = serde_json::from_slice::<AdminNifflerReferralLedgerMutationRequest>(request_body)
        .map_err(|err| niffler_bad_request(format!("请求体不是合法 JSON：{err}")))?;
    normalize_optional_text(payload.note, 500)
}

fn niffler_admin_operator_id(request_context: &AdminRequestContext<'_>) -> Option<String> {
    request_context
        .decision()
        .and_then(|decision| decision.admin_principal.as_ref())
        .map(|principal| principal.user_id.clone())
}

fn normalize_required_text(
    value: &str,
    label: &str,
    max_len: usize,
) -> Result<String, Response<Body>> {
    let value = value.trim();
    if value.is_empty() {
        return Err(niffler_bad_request(format!("{label}不能为空")));
    }
    if value.chars().count() > max_len {
        return Err(niffler_bad_request(format!(
            "{label}不能超过 {max_len} 个字符"
        )));
    }
    Ok(value.to_string())
}

fn normalize_optional_text(
    value: Option<String>,
    max_len: usize,
) -> Result<Option<String>, Response<Body>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > max_len {
        return Err(niffler_bad_request(format!(
            "字段不能超过 {max_len} 个字符"
        )));
    }
    Ok(Some(value.to_string()))
}

fn parse_usize_query(query: Option<&str>, key: &str) -> Option<usize> {
    query_param_value(query, key).and_then(|value| value.parse::<usize>().ok())
}

fn parse_bool_query(query: Option<&str>, key: &str) -> Option<bool> {
    query_param_value(query, key).and_then(|value| match value.as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    })
}

fn optional_query_text(query: Option<&str>, key: &str) -> Option<String> {
    query_param_value(query, key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_error_response_scope_query(
    query: Option<&str>,
    key: &str,
) -> Option<NifflerErrorResponseScope> {
    query_param_value(query, key).and_then(|value| match value.as_str() {
        "platform" => Some(NifflerErrorResponseScope::Platform),
        "upstream" => Some(NifflerErrorResponseScope::Upstream),
        _ => None,
    })
}

fn parse_runtime_rollout_target_scope_query(
    query: Option<&str>,
) -> Result<Option<NifflerRuntimeRolloutTargetScope>, Response<Body>> {
    query_param_value(query, "target_scope")
        .map(|value| {
            NifflerRuntimeRolloutTargetScope::from_database(value.as_str())
                .map_err(|_| niffler_bad_request("灰度对象范围只能是 api_key 或 product_plan"))
        })
        .transpose()
}

fn parse_billing_reservation_status_query(
    query: Option<&str>,
) -> Result<Option<NifflerBillingReservationStatus>, Response<Body>> {
    query_param_value(query, "status")
        .map(|value| {
            NifflerBillingReservationStatus::from_database(value.as_str()).map_err(|_| {
                niffler_bad_request(
                    "计费预占状态只能是 active、settled、released、expired 或 manual_review",
                )
            })
        })
        .transpose()
}

fn parse_referral_reward_ledger_status_query(
    query: Option<&str>,
) -> Result<Option<NifflerReferralRewardLedgerStatus>, Response<Body>> {
    query_param_value(query, "status")
        .map(|value| {
            NifflerReferralRewardLedgerStatus::from_database(value.as_str()).map_err(|_| {
                niffler_bad_request("返利流水状态只能是 pending、paid、failed 或 cancelled")
            })
        })
        .transpose()
}

fn niffler_bad_request(detail: impl Into<String>) -> Response<Body> {
    (
        http::StatusCode::BAD_REQUEST,
        Json(json!({ "detail": detail.into() })),
    )
        .into_response()
}

fn niffler_not_found(detail: impl Into<String>) -> Response<Body> {
    (
        http::StatusCode::NOT_FOUND,
        Json(json!({ "detail": detail.into() })),
    )
        .into_response()
}

fn niffler_internal_error(detail: impl Into<String>) -> Response<Body> {
    (
        http::StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "detail": detail.into() })),
    )
        .into_response()
}

fn niffler_method_not_allowed(detail: impl Into<String>) -> Response<Body> {
    (
        http::StatusCode::METHOD_NOT_ALLOWED,
        Json(json!({ "detail": detail.into() })),
    )
        .into_response()
}

fn niffler_data_unavailable_response() -> Response<Body> {
    (
        http::StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "detail": "Niffler 核心数据暂不可用" })),
    )
        .into_response()
}

async fn build_legacy_dependency_audit_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    let generated_at_unix_secs = current_unix_secs();
    let offset = parse_usize_query(request_context.query_string(), "offset").unwrap_or(0);
    let limit = parse_legacy_audit_limit(request_context.query_string());
    let mut notes = vec![
        "这个接口只读稽核旧依赖，不冻结、不删除、不修改线上请求链路。".to_string(),
        "普通用户 Key 目前没有跨用户分页仓储，本片不会新增无界扫描；后续先接入访问审计再判断是否还能下线旧字段。".to_string(),
    ];

    let mut user_key_page = if state.has_auth_api_key_data_reader() {
        state
            .app()
            .data
            .list_auth_api_key_export_standalone_records_page(&StandaloneApiKeyExportListQuery {
                skip: offset,
                limit: limit.saturating_add(1),
                is_active: None,
            })
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))?
    } else {
        notes.push("Auth API Key 只读仓储未启用，不能读取独立 Key 旧限制样本。".to_string());
        Vec::new()
    };
    let has_more_user_keys = user_key_page.len() > limit;
    user_key_page.truncate(limit);
    let user_key_legacy_restrictions = collect_user_key_legacy_restrictions(&user_key_page);

    let user_groups = state.list_user_groups().await?;
    let user_group_legacy_policies = collect_user_group_legacy_policies(&user_groups);

    let providers = if state.has_provider_catalog_data_reader() {
        state.list_provider_catalog_providers(false).await?
    } else {
        notes.push(
            "Provider 只读仓储未启用，不能读取旧 Provider 和 Provider Key 样本。".to_string(),
        );
        Vec::new()
    };
    let provider_ids = providers
        .iter()
        .map(|provider| provider.id.clone())
        .collect::<Vec<_>>();
    let provider_map = providers
        .iter()
        .map(|provider| (provider.id.as_str(), provider))
        .collect::<BTreeMap<_, _>>();
    let keys = if state.has_provider_catalog_data_reader() && !provider_ids.is_empty() {
        state
            .list_provider_catalog_key_summaries_by_provider_ids(&provider_ids)
            .await?
    } else {
        Vec::new()
    };
    let mut provider_key_legacy_restrictions = collect_key_scope_residue(&keys, &provider_map);
    let account_labels =
        load_account_labels_for_readiness(state, &provider_key_legacy_restrictions, &[], &[])
            .await?;
    apply_account_labels_to_key_residue(&mut provider_key_legacy_restrictions, &account_labels);

    let provider_models = read_provider_models(state, &providers).await?;
    let provider_model_price_dependencies =
        collect_provider_model_price_dependencies(&provider_models, &provider_map);
    let legacy_write_entrypoints = legacy_write_entrypoints();
    let runtime_read_dependencies = runtime_read_dependencies();

    let report = NifflerLegacyDependencyAuditReport {
        schema_version: 1,
        generated_at_unix_secs,
        offset,
        limit,
        has_more_user_keys,
        summary: NifflerLegacyDependencyAuditSummary {
            user_key_restrictions_in_page: user_key_legacy_restrictions.len() as u64,
            user_group_policy_items: user_group_legacy_policies.len() as u64,
            provider_key_restriction_items: provider_key_legacy_restrictions.len() as u64,
            provider_model_price_dependency_items: provider_model_price_dependencies.len() as u64,
            legacy_write_entrypoints: legacy_write_entrypoints.len() as u64,
            runtime_read_dependencies: runtime_read_dependencies.len() as u64,
        },
        user_key_legacy_restrictions,
        user_group_legacy_policies,
        provider_key_legacy_restrictions,
        provider_model_price_dependencies,
        legacy_write_entrypoints,
        runtime_read_dependencies,
        notes,
    };
    Ok(Json(report).into_response())
}

fn parse_legacy_audit_limit(query_string: Option<&str>) -> usize {
    parse_usize_query(query_string, "limit")
        .unwrap_or(50)
        .clamp(1, MAX_LEGACY_AUDIT_LIMIT)
}

fn collect_user_key_legacy_restrictions(
    records: &[StoredAuthApiKeyExportRecord],
) -> Vec<NifflerLegacyUserKeyRestriction> {
    let mut items = Vec::new();
    for record in records {
        let mut fields = Vec::new();
        push_string_list_field_if_present(
            &mut fields,
            "allowed_providers",
            &record.allowed_providers,
        );
        push_string_list_field_if_present(
            &mut fields,
            "allowed_api_formats",
            &record.allowed_api_formats,
        );
        push_string_list_field_if_present(&mut fields, "allowed_models", &record.allowed_models);
        if fields.is_empty() {
            continue;
        }
        let field_labels = fields
            .iter()
            .map(|field| user_key_legacy_field_label(field).to_string())
            .collect::<Vec<_>>();
        items.push(NifflerLegacyUserKeyRestriction {
            key_id: record.api_key_id.clone(),
            key_name: record.name.clone(),
            owner_label: format!("用户 {}", record.user_id),
            is_standalone: record.is_standalone,
            group_id: record.group_id.clone(),
            group_name: record.group_name.clone(),
            field_names: fields,
            field_labels,
            reason: "这把用户 Key 自身仍保存可用 Provider、API 格式或模型限制。".to_string(),
            impact: "新模型里用户 Key 只绑定一个产品策略；如果 Key 自身继续保存限制，页面看到的策略和实际可用范围可能不一致。".to_string(),
            recommended_action: "迁移前把这些限制并入产品策略；确认无用后再清空旧 Key 字段。".to_string(),
        });
        if items.len() >= MAX_ISSUE_ITEMS {
            break;
        }
    }
    items
}

fn push_string_list_field_if_present(
    fields: &mut Vec<String>,
    field_name: &str,
    value: &Option<Vec<String>>,
) {
    if value
        .as_ref()
        .is_some_and(|values| values.iter().any(|item| !item.trim().is_empty()))
    {
        fields.push(field_name.to_string());
    }
}

fn user_key_legacy_field_label(field: &str) -> &'static str {
    match field {
        "allowed_providers" => "Key 自身可用 Provider",
        "allowed_api_formats" => "Key 自身 API 格式",
        "allowed_models" => "Key 自身允许模型",
        _ => "未归类 Key 字段",
    }
}

fn collect_user_group_legacy_policies(groups: &[StoredUserGroup]) -> Vec<NifflerLegacyGroupPolicy> {
    let mut items = Vec::new();
    for group in groups {
        push_group_policy_item(
            &mut items,
            group,
            "allowed_providers",
            "分组可用 Provider",
            &group.allowed_providers_mode,
            group.allowed_providers.as_ref().map_or(0, Vec::len) as u64,
        );
        push_group_policy_item(
            &mut items,
            group,
            "allowed_api_formats",
            "分组 API 格式",
            &group.allowed_api_formats_mode,
            group.allowed_api_formats.as_ref().map_or(0, Vec::len) as u64,
        );
        push_group_policy_item(
            &mut items,
            group,
            "allowed_models",
            "分组允许模型",
            &group.allowed_models_mode,
            group.allowed_models.as_ref().map_or(0, Vec::len) as u64,
        );
        if (group.sales_multiplier - 1.0).abs() > f64::EPSILON {
            push_group_policy_item(
                &mut items,
                group,
                "sales_multiplier",
                "分组销售倍率",
                "configured",
                1,
            );
        }
        if group
            .model_sales_multipliers
            .as_ref()
            .is_some_and(value_has_content)
        {
            push_group_policy_item(
                &mut items,
                group,
                "model_sales_multipliers",
                "分组模型销售倍率",
                "configured",
                1,
            );
        }
        if items.len() >= MAX_ISSUE_ITEMS {
            break;
        }
    }
    items.truncate(MAX_ISSUE_ITEMS);
    items
}

fn push_group_policy_item(
    items: &mut Vec<NifflerLegacyGroupPolicy>,
    group: &StoredUserGroup,
    field_name: &str,
    field_label: &str,
    mode: &str,
    item_count: u64,
) {
    let mode = mode.trim();
    if mode.eq_ignore_ascii_case("inherit") && item_count == 0 {
        return;
    }
    items.push(NifflerLegacyGroupPolicy {
        group_id: group.id.clone(),
        group_name: group.name.clone(),
        field_name: field_name.to_string(),
        field_label: field_label.to_string(),
        mode: mode.to_string(),
        item_count,
        reason: "旧用户分组仍在表达模型、Provider、API 格式或销售价格规则。".to_string(),
        impact: "第 5 批切换后，这些规则应该由产品策略表达；如果旧分组继续可写，管理员会同时维护两套规则。".to_string(),
        recommended_action: "迁移成产品策略字段，并在第二片冻结已迁移分组的旧写入口。".to_string(),
    });
}

fn collect_provider_model_price_dependencies(
    provider_models: &[StoredAdminProviderModel],
    provider_map: &BTreeMap<&str, &StoredProviderCatalogProvider>,
) -> Vec<NifflerLegacyProviderModelPriceDependency> {
    let mut items = Vec::new();
    for model in provider_models {
        if !model.is_active {
            continue;
        }
        let has_provider_price =
            has_model_price(model.price_per_request, model.tiered_pricing.as_ref());
        let has_inherited_price = has_model_price(
            model.global_model_default_price_per_request,
            model.global_model_default_tiered_pricing.as_ref(),
        );
        let (dependency_kind, dependency_label) = if has_provider_price {
            ("provider_model_price", "Provider 模型自身价格")
        } else if has_inherited_price {
            ("global_model_price", "继承全局模型基础价格")
        } else {
            continue;
        };
        let provider = provider_map.get(model.provider_id.as_str()).copied();
        items.push(NifflerLegacyProviderModelPriceDependency {
            provider_id: model.provider_id.clone(),
            provider_name: provider.map(|item| item.name.clone()),
            model_id: model.id.clone(),
            model_name: model
                .global_model_name
                .clone()
                .unwrap_or_else(|| model.provider_model_name.clone()),
            dependency_kind: dependency_kind.to_string(),
            dependency_label: dependency_label.to_string(),
            reason: "旧 Provider 模型价格仍可能参与上游成本和展示。".to_string(),
            impact: "新价格模型切换前，如果旧 Provider 模型价格继续可写，会造成成本价、销售价和历史结算快照口径不一致。".to_string(),
            recommended_action: "迁移为模型基础价格、上游成本倍率或账号成本倍率；迁移完成后冻结旧 Provider 模型价格写入口。".to_string(),
        });
        if items.len() >= MAX_ISSUE_ITEMS {
            break;
        }
    }
    items
}

fn legacy_write_entrypoints() -> Vec<NifflerLegacyCodeDependency> {
    vec![
        legacy_code_dependency(
            "旧 Provider 写入口",
            "新增、更新、删除旧 Provider",
            Some("POST/PUT/DELETE"),
            "/api/admin/providers",
            "仍存在",
            "Provider 旧页面仍能维护供给侧配置。",
            "第二片对已迁移对象改为只读或跳转到新上游服务页面。",
        ),
        legacy_code_dependency(
            "旧 Provider Key 写入口",
            "新增、更新、恢复、测试旧账号",
            Some("POST/PUT/PATCH"),
            "/api/admin/endpoints/keys",
            "仍存在",
            "旧账号配置仍在 Provider Key 上维护。",
            "第二片对已迁移账号冻结旧写入口，提示去上游账号页面处理。",
        ),
        legacy_code_dependency(
            "旧 Provider 模型价格写入口",
            "维护 Provider 模型和价格",
            Some("POST/PUT/DELETE"),
            "/api/admin/providers/{providerId}/models",
            "仍存在",
            "旧模型价格入口仍能影响上游成本和页面展示。",
            "第二片对已迁移 Provider 的模型价格改为只读。",
        ),
        legacy_code_dependency(
            "旧用户分组写入口",
            "维护分组可用 Provider、API 格式、模型和倍率",
            Some("POST/PUT/DELETE"),
            "/api/admin/users/groups",
            "仍存在",
            "产品策略已经接管这些业务语义，但旧分组仍可写。",
            "第二片对已迁移分组冻结旧写入口，提示去产品策略页面处理。",
        ),
        legacy_code_dependency(
            "旧用户 Key 限制写入口",
            "维护 Key 自身可用 Provider、API 格式和模型",
            Some("POST/PUT/PATCH"),
            "/api/admin/users/{userId}/api-keys",
            "仍存在",
            "用户 Key 仍可能绕过产品策略保存自身限制。",
            "第二片对已迁移 Key 冻结旧限制字段，只允许绑定产品策略。",
        ),
    ]
}

fn runtime_read_dependencies() -> Vec<NifflerLegacyCodeDependency> {
    vec![
        legacy_code_dependency(
            "运行时鉴权",
            "读取用户和 Key 的 allowed_providers、allowed_api_formats、allowed_models",
            None,
            "apps/aether-gateway/src/control/auth/resolution.rs",
            "仍读取",
            "运行时仍从旧用户和旧 Key 字段生成有效权限。",
            "第三片对已迁移产品策略切到新产品策略读源。",
        ),
        legacy_code_dependency(
            "模型权限判断",
            "检查请求模型是否在旧 allowed_models 内",
            None,
            "apps/aether-gateway/src/control/auth/gate.rs",
            "仍读取",
            "模型权限仍可能由旧字段决定。",
            "第三片对已迁移 Key 只按产品策略模型判断。",
        ),
        legacy_code_dependency(
            "调度服务选择",
            "读取旧 Provider、Provider Key、Key 模型限制和 API 格式",
            None,
            "apps/aether-gateway/src/ai_serving/planner/candidate_source.rs",
            "仍读取",
            "调度仍从旧 Provider/Key 结构选择实际服务和账号。",
            "第三片把已迁移策略切到新上游服务和上游账号。",
        ),
        legacy_code_dependency(
            "上游执行快照",
            "读取 provider_api_keys.allowed_models、api_formats 和认证配置",
            None,
            "crates/aether-provider-transport/src/snapshot_mapping.rs",
            "仍读取",
            "上游执行仍依赖旧账号字段构造请求。",
            "第三片把已迁移账号切到新账号能力和凭证快照。",
        ),
        legacy_code_dependency(
            "旧价格读源",
            "读取全局模型和 Provider 模型价格",
            None,
            "apps/aether-gateway/src/handlers/admin/model/payloads.rs",
            "仍读取",
            "旧价格仍用于成本展示和部分对账。",
            "第三片切到基础价、成本倍率和结算快照读源。",
        ),
    ]
}

fn legacy_code_dependency(
    area: &str,
    label: &str,
    method: Option<&str>,
    path: &str,
    current_status: &str,
    reason: &str,
    next_action: &str,
) -> NifflerLegacyCodeDependency {
    NifflerLegacyCodeDependency {
        area: area.to_string(),
        label: label.to_string(),
        method: method.map(ToOwned::to_owned),
        path: path.to_string(),
        current_status: current_status.to_string(),
        reason: reason.to_string(),
        next_action: next_action.to_string(),
    }
}

async fn build_readiness_report(
    state: &AdminAppState<'_>,
    recent_days: u32,
) -> Result<NifflerCoreReadinessReport, GatewayError> {
    let generated_at_unix_secs = current_unix_secs();
    let shadow_tables = build_shadow_table_status(state).await?;

    let providers = if state.has_provider_catalog_data_reader() {
        state.list_provider_catalog_providers(false).await?
    } else {
        Vec::new()
    };
    let provider_ids = providers
        .iter()
        .map(|provider| provider.id.clone())
        .collect::<Vec<_>>();
    let keys = if state.has_provider_catalog_data_reader() && !provider_ids.is_empty() {
        state
            .list_provider_catalog_key_summaries_by_provider_ids(&provider_ids)
            .await?
    } else {
        Vec::new()
    };
    let user_groups = state.list_user_groups().await?;
    let global_models = if state.has_global_model_data_reader() {
        state
            .list_admin_global_models(&AdminGlobalModelListQuery {
                offset: 0,
                limit: MAX_GLOBAL_MODELS,
                is_active: None,
                search: None,
            })
            .await?
            .items
    } else {
        Vec::new()
    };
    let provider_models = read_provider_models(state, &providers).await?;
    let provider_map = providers
        .iter()
        .map(|provider| (provider.id.as_str(), provider))
        .collect::<BTreeMap<_, _>>();
    let key_map = keys
        .iter()
        .map(|key| (key.id.as_str(), key))
        .collect::<BTreeMap<_, _>>();

    let disabled_provider_references =
        collect_disabled_provider_references(&user_groups, &provider_map);
    let mut key_scope_residue = collect_key_scope_residue(&keys, &provider_map);
    let group_policy_gaps = collect_group_policy_gaps(&user_groups);
    let price_gaps = collect_price_gaps(&global_models, &provider_models, &provider_map);
    let (mut recent_usage_anomalies, recent_problem_usage_sample_count) =
        collect_recent_usage_anomalies(
            state,
            recent_days,
            generated_at_unix_secs,
            &provider_map,
            &key_map,
        )
        .await?;
    let (route_skip_reasons, mut route_skip_samples) =
        collect_route_skip_reports(state, &provider_map, &key_map).await?;
    let account_labels = load_account_labels_for_readiness(
        state,
        &key_scope_residue,
        &recent_usage_anomalies,
        &route_skip_samples,
    )
    .await?;
    apply_account_labels_to_key_residue(&mut key_scope_residue, &account_labels);
    apply_account_labels_to_usage_anomalies(&mut recent_usage_anomalies, &account_labels);
    apply_account_labels_to_route_skip_samples(&mut route_skip_samples, &account_labels);
    let provider_status_counts = provider_status_counts(&providers);
    let account_status_counts = account_status_counts(&keys);
    let issues = collect_issues(
        state,
        &shadow_tables,
        &disabled_provider_references,
        &key_scope_residue,
        &group_policy_gaps,
        &price_gaps,
        &recent_usage_anomalies,
    );

    Ok(NifflerCoreReadinessReport {
        schema_version: 1,
        generated_at_unix_secs,
        recent_days,
        shadow_tables,
        summary: NifflerCoreReadinessSummary {
            providers_total: providers.len() as u64,
            providers_active: providers
                .iter()
                .filter(|provider| provider.is_active)
                .count() as u64,
            provider_keys_total: keys.len() as u64,
            provider_keys_active: keys.iter().filter(|key| key.is_active).count() as u64,
            product_plans_total: user_groups.len() as u64,
            product_plans_public: user_groups
                .iter()
                .filter(|group| group.visibility.trim().eq_ignore_ascii_case("public"))
                .count() as u64,
            global_models_total: global_models.len() as u64,
            global_models_active: global_models.iter().filter(|model| model.is_active).count()
                as u64,
            recent_problem_usage_sample_count,
        },
        provider_mapping: NifflerCoreMappingSummary {
            legacy_count: providers.len() as u64,
            mapped_count: providers
                .iter()
                .filter(|provider| provider.is_active)
                .count() as u64,
            blocked_count: providers
                .iter()
                .filter(|provider| !provider.is_active)
                .count() as u64,
            notes: vec![
                "启用 Provider 可以映射为上游服务；停用 Provider 不能被新产品策略选择。"
                    .to_string(),
            ],
        },
        account_mapping: NifflerCoreMappingSummary {
            legacy_count: keys.len() as u64,
            mapped_count: keys
                .iter()
                .filter(|key| key_status_label(key) == "available")
                .count() as u64,
            blocked_count: keys
                .iter()
                .filter(|key| key_status_label(key) != "available")
                .count() as u64,
            notes: vec!["启用且未标记 OAuth 失效的 Provider Key 可以映射为上游账号。".to_string()],
        },
        product_plan_mapping: NifflerCoreMappingSummary {
            legacy_count: user_groups.len() as u64,
            mapped_count: user_groups.len() as u64,
            blocked_count: 0,
            notes: vec![
                "旧用户分组可以映射为产品策略；公开/内部只影响是否允许用户 Key 公开绑定。"
                    .to_string(),
            ],
        },
        provider_status_counts,
        account_status_counts,
        disabled_provider_references,
        key_scope_residue,
        group_policy_gaps,
        price_gaps,
        recent_usage_anomalies,
        route_skip_reasons,
        route_skip_samples,
        issues,
    })
}

async fn build_shadow_table_status(
    state: &AdminAppState<'_>,
) -> Result<NifflerShadowTableStatus, GatewayError> {
    let rows = state
        .app()
        .data
        .check_table_existence(SHADOW_TABLES)
        .await
        .map_err(|err| GatewayError::Internal(err.to_string()))?;
    let tables = rows
        .into_iter()
        .map(|(table_name, exists)| NifflerShadowTableItem { table_name, exists })
        .collect::<Vec<_>>();
    let existing_tables = tables.iter().filter(|table| table.exists).count() as u64;
    Ok(NifflerShadowTableStatus {
        database_driver: state.app().data.database_driver_name(),
        expected_tables: SHADOW_TABLES.len() as u64,
        existing_tables,
        all_present: existing_tables == SHADOW_TABLES.len() as u64,
        tables,
    })
}

async fn read_provider_models(
    state: &AdminAppState<'_>,
    providers: &[StoredProviderCatalogProvider],
) -> Result<Vec<StoredAdminProviderModel>, GatewayError> {
    if !state.has_global_model_data_reader() {
        return Ok(Vec::new());
    }
    let mut models = Vec::new();
    for provider in providers {
        let mut provider_models = state
            .list_admin_provider_models(&AdminProviderModelListQuery {
                provider_id: provider.id.clone(),
                is_active: None,
                offset: 0,
                limit: MAX_PROVIDER_MODELS_PER_PROVIDER,
            })
            .await?;
        models.append(&mut provider_models);
    }
    Ok(models)
}

fn collect_disabled_provider_references(
    user_groups: &[StoredUserGroup],
    provider_map: &BTreeMap<&str, &StoredProviderCatalogProvider>,
) -> Vec<NifflerDisabledProviderReference> {
    let mut references = Vec::new();
    for group in user_groups {
        if !group
            .allowed_providers_mode
            .trim()
            .eq_ignore_ascii_case("specific")
        {
            continue;
        }
        if let Some(provider_ids) = &group.allowed_providers {
            for provider_id in provider_ids {
                push_disabled_provider_reference(
                    &mut references,
                    group,
                    provider_id,
                    "allowed_providers",
                    provider_map,
                );
            }
        }
    }
    references.truncate(MAX_ISSUE_ITEMS);
    references
}

fn push_disabled_provider_reference(
    references: &mut Vec<NifflerDisabledProviderReference>,
    group: &StoredUserGroup,
    provider_id: &str,
    source_field: &str,
    provider_map: &BTreeMap<&str, &StoredProviderCatalogProvider>,
) {
    let Some(provider) = provider_map.get(provider_id).copied().or_else(|| {
        provider_map
            .values()
            .copied()
            .find(|provider| provider.name == provider_id)
    }) else {
        return;
    };
    if provider.is_active {
        return;
    }
    let exists = references.iter().any(|item| {
        item.product_plan_id == group.id
            && item.provider_id == provider.id
            && item.source_field == source_field
    });
    if exists {
        return;
    }
    references.push(NifflerDisabledProviderReference {
        product_plan_id: group.id.clone(),
        product_plan_name: group.name.clone(),
        provider_id: provider.id.clone(),
        provider_name: provider.name.clone(),
        source_field: source_field.to_string(),
        source_field_label: source_field_label(source_field).to_string(),
        reason: "分组的可用 Provider 列表里仍包含已停用 Provider。".to_string(),
        impact: "迁移到新产品策略后，停用 Provider 不允许被选择；如果不处理，这个分组实际可用服务会比页面配置少。".to_string(),
        recommended_action: "从分组里移除这个 Provider，或先恢复 Provider 再迁移。".to_string(),
    });
}

fn collect_key_scope_residue(
    keys: &[StoredProviderCatalogKey],
    provider_map: &BTreeMap<&str, &StoredProviderCatalogProvider>,
) -> Vec<NifflerKeyScopeResidue> {
    let mut residue = Vec::new();
    for key in keys {
        let mut fields = Vec::new();
        push_json_field_if_present(&mut fields, "api_formats", &key.api_formats);
        push_json_field_if_present(&mut fields, "auth_type_by_format", &key.auth_type_by_format);
        push_json_field_if_present(
            &mut fields,
            "allow_auth_channel_mismatch_formats",
            &key.allow_auth_channel_mismatch_formats,
        );
        push_json_field_if_present(&mut fields, "rate_multipliers", &key.rate_multipliers);
        push_json_field_if_present(
            &mut fields,
            "global_priority_by_format",
            &key.global_priority_by_format,
        );
        push_json_field_if_present(&mut fields, "allowed_models", &key.allowed_models);
        push_json_field_if_present(&mut fields, "locked_models", &key.locked_models);
        push_json_field_if_present(
            &mut fields,
            "model_include_patterns",
            &key.model_include_patterns,
        );
        push_json_field_if_present(
            &mut fields,
            "model_exclude_patterns",
            &key.model_exclude_patterns,
        );
        if fields.is_empty() {
            continue;
        }
        let provider_name = provider_map
            .get(key.provider_id.as_str())
            .map(|provider| provider.name.clone());
        let display_name = non_empty_string(&key.name).unwrap_or_else(|| key.id.clone());
        let field_labels = fields
            .iter()
            .map(|field| residue_field_label(field).to_string())
            .collect::<Vec<_>>();
        residue.push(NifflerKeyScopeResidue {
            subject_kind: "provider_key".to_string(),
            key_id: key.id.clone(),
            key_name: Some(key.name.clone()),
            owner_label: provider_name.clone().or_else(|| Some(key.provider_id.clone())),
            display_name,
            provider_id: Some(key.provider_id.clone()),
            provider_name,
            account_label: None,
            residue_fields: fields,
            field_labels,
            reason: "这把上游账号仍在 Key 自身保存模型、格式或优先级限制。".to_string(),
            impact: "新模型里这些限制应该归到账号能力或调度策略；如果继续散落在 Key 上，页面和后端调度容易不一致。".to_string(),
            recommended_action: "迁移前确认这些限制是否还需要保留，需要保留的迁到账号能力或调度策略，不需要的清理掉。".to_string(),
        });
    }
    residue.truncate(MAX_ISSUE_ITEMS);
    residue
}

fn push_json_field_if_present(
    fields: &mut Vec<String>,
    field_name: &str,
    value: &Option<serde_json::Value>,
) {
    if value.as_ref().is_some_and(value_has_content) {
        fields.push(field_name.to_string());
    }
}

fn value_has_content(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => false,
        serde_json::Value::Array(values) => !values.is_empty(),
        serde_json::Value::Object(object) => !object.is_empty(),
        serde_json::Value::String(value) => !value.trim().is_empty(),
        _ => true,
    }
}

fn collect_group_policy_gaps(user_groups: &[StoredUserGroup]) -> Vec<NifflerGroupPolicyGap> {
    let mut gaps = Vec::new();
    for group in user_groups {
        if !group
            .allowed_models_mode
            .trim()
            .eq_ignore_ascii_case("specific")
        {
            gaps.push(NifflerGroupPolicyGap {
                product_plan_id: group.id.clone(),
                product_plan_name: group.name.clone(),
                gap_kind: "unrestricted_models".to_string(),
                gap_label: "允许全部模型".to_string(),
                message: "这个用户分组当前允许全部模型；迁移为产品策略前需要确认是否继续开放全部模型。"
                    .to_string(),
                impact: "如果直接迁移，会变成一个可售模型范围很大的产品策略，用户可能看到不该开放的模型。".to_string(),
                recommended_action: "确认这个分组是否真的要开放全部模型；如果不是，先收敛为明确的可售模型列表。".to_string(),
            });
            if gaps.len() >= MAX_ISSUE_ITEMS {
                break;
            }
            continue;
        }
        if group
            .allowed_models
            .as_ref()
            .is_none_or(|models| models.is_empty())
        {
            gaps.push(NifflerGroupPolicyGap {
                product_plan_id: group.id.clone(),
                product_plan_name: group.name.clone(),
                gap_kind: "empty_specific_models".to_string(),
                gap_label: "指定模型为空".to_string(),
                message: "这个用户分组设置为只允许指定模型，但模型列表为空；迁移前需要明确可售模型。"
                    .to_string(),
                impact: "迁移后这个产品策略会没有可售模型，绑定到这个策略的用户 Key 将无法正常使用模型。".to_string(),
                recommended_action: "补齐可售模型列表，或把这个分组停用后再迁移。".to_string(),
            });
        }
        if gaps.len() >= MAX_ISSUE_ITEMS {
            break;
        }
    }
    gaps
}

fn collect_price_gaps(
    global_models: &[StoredAdminGlobalModel],
    provider_models: &[StoredAdminProviderModel],
    provider_map: &BTreeMap<&str, &StoredProviderCatalogProvider>,
) -> Vec<NifflerPriceGap> {
    let mut gaps = Vec::new();
    for model in global_models {
        if has_model_price(
            model.default_price_per_request,
            model.default_tiered_pricing.as_ref(),
        ) {
            continue;
        }
        gaps.push(NifflerPriceGap {
            scope: "global_model".to_string(),
            scope_label: "模型基础价格".to_string(),
            provider_id: None,
            provider_name: None,
            model_id: Some(model.id.clone()),
            model_name: model.name.clone(),
            missing_fields: vec![
                "default_price_per_request".to_string(),
                "default_tiered_pricing".to_string(),
            ],
            reason: "全局模型没有基础价格。".to_string(),
            impact:
                "钱包销售价和套餐消耗都依赖基础价格；缺少基础价格会导致迁移后的计费规则不明确。"
                    .to_string(),
            recommended_action:
                "按官方 API 最新定价补齐模型基础价格，再配置销售倍率或单模型覆盖价格。".to_string(),
        });
        if gaps.len() >= MAX_ISSUE_ITEMS {
            return gaps;
        }
    }
    for model in provider_models {
        let has_own_price = has_model_price(model.price_per_request, model.tiered_pricing.as_ref());
        let has_global_price = has_model_price(
            model.global_model_default_price_per_request,
            model.global_model_default_tiered_pricing.as_ref(),
        );
        if has_own_price || has_global_price {
            continue;
        }
        let provider = provider_map.get(model.provider_id.as_str());
        gaps.push(NifflerPriceGap {
            scope: "provider_model".to_string(),
            scope_label: "上游模型成本价格".to_string(),
            provider_id: Some(model.provider_id.clone()),
            provider_name: provider.map(|item| item.name.clone()),
            model_id: Some(model.id.clone()),
            model_name: model
                .global_model_name
                .clone()
                .unwrap_or_else(|| model.provider_model_name.clone()),
            missing_fields: vec![
                "price_per_request".to_string(),
                "tiered_pricing".to_string(),
            ],
            reason: "Provider 模型没有自身价格，也没有可继承的全局模型价格。".to_string(),
            impact: "迁移后无法计算这个上游模型的成本价，成本对账和账号池成本窗口都会不准确。".to_string(),
            recommended_action: "先补齐全局模型基础价格；如果这个 Provider 成本不同，再配置上游成本倍率或 Provider 模型价格。".to_string(),
        });
        if gaps.len() >= MAX_ISSUE_ITEMS {
            return gaps;
        }
    }
    gaps
}

fn has_model_price(
    price_per_request: Option<f64>,
    tiered_pricing: Option<&serde_json::Value>,
) -> bool {
    price_per_request.is_some_and(|price| price.is_finite() && price >= 0.0)
        || tiered_pricing
            .and_then(|value| value.get("tiers"))
            .and_then(serde_json::Value::as_array)
            .is_some_and(|tiers| {
                tiers.iter().any(|tier| {
                    [
                        "input_price_per_1m",
                        "output_price_per_1m",
                        "cache_creation_price_per_1m",
                        "cache_read_price_per_1m",
                    ]
                    .iter()
                    .any(|field| {
                        tier.get(*field)
                            .and_then(serde_json::Value::as_f64)
                            .is_some_and(|price| price.is_finite() && price >= 0.0)
                    })
                })
            })
}

async fn collect_recent_usage_anomalies(
    state: &AdminAppState<'_>,
    recent_days: u32,
    now_unix_secs: u64,
    provider_map: &BTreeMap<&str, &StoredProviderCatalogProvider>,
    key_map: &BTreeMap<&str, &StoredProviderCatalogKey>,
) -> Result<(Vec<NifflerUsageAnomaly>, u64), GatewayError> {
    if !state.has_usage_data_reader() {
        return Ok((Vec::new(), 0));
    }
    let from = now_unix_secs.saturating_sub(u64::from(recent_days) * 24 * 60 * 60);
    let rows = state
        .list_usage_audits(&UsageAuditListQuery {
            created_from_unix_secs: Some(from),
            created_until_unix_secs: Some(now_unix_secs),
            user_id: None,
            provider_name: None,
            model: None,
            api_format: None,
            statuses: None,
            is_stream: None,
            error_only: false,
            limit: Some(MAX_USAGE_SCAN),
            offset: Some(0),
            newest_first: true,
        })
        .await?;
    let mut anomalies = Vec::new();
    for row in rows {
        let Some(diagnosis) = usage_anomaly_diagnosis(&row) else {
            continue;
        };
        let key = row
            .provider_api_key_id
            .as_deref()
            .and_then(|key_id| key_map.get(key_id).copied());
        let provider_name = row
            .provider_id
            .as_deref()
            .and_then(|provider_id| provider_map.get(provider_id).copied())
            .map(|provider| provider.name.clone());
        let provider_display_name = provider_name
            .clone()
            .or_else(|| non_empty_string(&row.provider_name))
            .unwrap_or_else(|| "未选定上游".to_string());
        let provider_api_key_name = key
            .and_then(|key| non_empty_string(&key.name))
            .or_else(|| row.routing_key_name().map(ToOwned::to_owned));
        let package_debit_usd = row.settlement_package_debit_usd();
        let wallet_debit_usd = row.settlement_wallet_debit_usd();
        anomalies.push(NifflerUsageAnomaly {
            usage_id: row.id,
            request_id: row.request_id,
            created_at_unix_secs: unix_millis_or_secs_to_secs(row.created_at_unix_ms),
            provider_name: row.provider_name,
            provider_id: row.provider_id,
            provider_api_key_id: row.provider_api_key_id,
            provider_display_name,
            provider_api_key_name,
            provider_account_label: None,
            model: row.model,
            status: row.status,
            billing_status: row.billing_status,
            status_code: row.status_code,
            error_category: row.error_category,
            anomaly_kind: diagnosis.kind.to_string(),
            anomaly_label: diagnosis.label.to_string(),
            diagnosis: diagnosis.diagnosis.to_string(),
            impact: diagnosis.impact.to_string(),
            recommended_action: diagnosis.recommended_action.to_string(),
            total_cost_usd: row.total_cost_usd,
            actual_total_cost_usd: row.actual_total_cost_usd,
            package_debit_usd,
            wallet_debit_usd,
        });
        if anomalies.len() >= MAX_USAGE_ITEMS {
            break;
        }
    }
    let count = anomalies.len() as u64;
    Ok((anomalies, count))
}

struct UsageAnomalyDiagnosis {
    kind: &'static str,
    label: &'static str,
    diagnosis: &'static str,
    impact: &'static str,
    recommended_action: &'static str,
}

fn usage_anomaly_diagnosis(row: &StoredRequestUsageAudit) -> Option<UsageAnomalyDiagnosis> {
    let provider_unknown = row.provider_name.trim().eq_ignore_ascii_case("unknown")
        || row.provider_name.trim().is_empty()
        || row.provider_id.is_none();
    if provider_unknown && is_api_key_concurrency_limited(row) {
        return Some(UsageAnomalyDiagnosis {
            kind: "api_key_concurrency_limited",
            label: "平台并发拦截",
            diagnosis: "平台在选择上游前拦截了这个请求：用户 API Key 并发数已达上限，所以没有实际 Provider 或账号。",
            impact: "这类 unknown 不代表 Provider 丢失；请求没有进入上游，也不会消耗上游账号。",
            recommended_action: "检查用户 Key 的并发限制，或等待该用户的并发请求结束。",
        });
    }
    if provider_unknown {
        return Some(UsageAnomalyDiagnosis {
            kind: "provider_unknown",
            label: "未选定上游",
            diagnosis: "这条记录没有实际 Provider ID，失败发生在选定上游前，或旧记录没有保存可展示的上游服务。",
            impact: "管理员无法从使用记录直接定位上游账号，需要结合路由跳过原因判断是策略、额度、冷却还是配置问题。",
            recommended_action: "查看同页的路由跳过原因；如果是新近记录，需要优先修复调度前失败路径的错误记录归因。",
        });
    }
    if row.billing_status.trim().eq_ignore_ascii_case("pending")
        && row.status.trim().eq_ignore_ascii_case("completed")
    {
        return Some(UsageAnomalyDiagnosis {
            kind: "completed_billing_pending",
            label: "完成但未结算",
            diagnosis: "请求已完成，但结算没有最终完成；当前记录没有可展示的钱包扣费快照。",
            impact: "用户可能已经看到成功响应，但后台暂时无法确认套餐或钱包扣费是否完成。",
            recommended_action:
                "检查 usage 结算任务和 pending 清理任务；长期停留 pending 的记录需要进入人工对账。",
        });
    }
    if row.billing_status.trim().eq_ignore_ascii_case("pending") {
        return Some(UsageAnomalyDiagnosis {
            kind: "billing_pending",
            label: "等待结算",
            diagnosis: "请求仍在进行或等待超时清理，暂时没有最终扣费拆分。",
            impact: "这类记录在结算完成前不应用来判断最终扣费。",
            recommended_action: "等待请求结束或清理任务处理；超过预期时间仍 pending 时再人工检查。",
        });
    }
    if row.status.trim().eq_ignore_ascii_case("failed") && row.provider_api_key_id.is_none() {
        return Some(UsageAnomalyDiagnosis {
            kind: "failed_before_account_selected",
            label: "未选定账号失败",
            diagnosis:
                "这条失败记录没有上游账号 ID，说明失败发生在选定账号前或旧记录缺少账号快照。",
            impact: "管理员看不到具体账号，无法判断是哪个上游账号失败。",
            recommended_action:
                "结合路由跳过原因和请求错误信息定位；迁移后需要保证失败记录保存实际尝试链路。",
        });
    }
    let has_charge_snapshot =
        row.settlement_package_debit_usd().is_some() || row.settlement_wallet_debit_usd().is_some();
    if row.status.trim().eq_ignore_ascii_case("completed")
        && row.billing_status.trim().eq_ignore_ascii_case("settled")
        && row.total_cost_usd > 0.0
        && !has_charge_snapshot
    {
        return Some(UsageAnomalyDiagnosis {
            kind: "settled_without_charge_breakdown",
            label: "已结算但缺扣费拆分",
            diagnosis: "这条记录显示已结算且有销售金额，但没有套餐或钱包扣费拆分快照。",
            impact:
                "使用记录页面可能显示不出钱包扣款，管理员需要通过钱包流水或结算快照确认实际扣费。",
            recommended_action:
                "迁移结算快照前先对这类记录做只读对账；后续新结算必须强制写入扣费拆分。",
        });
    }
    None
}

fn is_api_key_concurrency_limited(row: &StoredRequestUsageAudit) -> bool {
    row.error_message
        .as_deref()
        .is_some_and(|message| message.contains("API Key 并发请求数已达上限"))
        || row
            .routing_local_execution_runtime_miss_reason()
            .is_some_and(|reason| reason == "api_key_concurrency_limit_reached")
        || row
            .routing_execution_path()
            .is_some_and(|path| path == "local_api_key_concurrency_limited")
}

async fn collect_route_skip_reports(
    state: &AdminAppState<'_>,
    provider_map: &BTreeMap<&str, &StoredProviderCatalogProvider>,
    key_map: &BTreeMap<&str, &StoredProviderCatalogKey>,
) -> Result<
    (
        Vec<NifflerRouteSkipReasonSummary>,
        Vec<NifflerRouteSkipSample>,
    ),
    GatewayError,
> {
    if !state.has_request_candidate_data_reader() {
        return Ok((Vec::new(), Vec::new()));
    }
    let rows = state
        .read_recent_request_candidates(MAX_ROUTE_SKIP_SAMPLE)
        .await?;
    let mut counts = BTreeMap::<String, u64>::new();
    let mut samples = Vec::new();
    for row in rows {
        let Some(reason) = row
            .skip_reason
            .as_deref()
            .map(str::trim)
            .filter(|reason| !reason.is_empty())
        else {
            continue;
        };
        *counts.entry(reason.to_string()).or_default() += 1;
        if samples.len() < MAX_ISSUE_ITEMS {
            samples.push(route_skip_sample(&row, reason, provider_map, key_map));
        }
    }
    let mut summaries = counts
        .into_iter()
        .map(|(reason, count)| {
            let info = route_skip_reason_info(&reason);
            NifflerRouteSkipReasonSummary {
                reason,
                label: info.label.to_string(),
                category: info.category.to_string(),
                count,
                impact: info.impact.to_string(),
                recommended_action: info.recommended_action.to_string(),
            }
        })
        .collect::<Vec<_>>();
    summaries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(left.reason.cmp(&right.reason))
    });
    summaries.truncate(MAX_ISSUE_ITEMS);
    Ok((summaries, samples))
}

fn route_skip_sample(
    row: &StoredRequestCandidate,
    reason: &str,
    provider_map: &BTreeMap<&str, &StoredProviderCatalogProvider>,
    key_map: &BTreeMap<&str, &StoredProviderCatalogKey>,
) -> NifflerRouteSkipSample {
    let provider = row
        .provider_id
        .as_deref()
        .and_then(|provider_id| provider_map.get(provider_id).copied());
    let key = row
        .key_id
        .as_deref()
        .and_then(|key_id| key_map.get(key_id).copied());
    let info = route_skip_reason_info(reason);
    NifflerRouteSkipSample {
        request_id: row.request_id.clone(),
        created_at_unix_secs: unix_millis_or_secs_to_secs(row.created_at_unix_ms),
        provider_id: row.provider_id.clone(),
        provider_name: provider.map(|provider| provider.name.clone()),
        key_id: row.key_id.clone(),
        key_name: key.and_then(|key| non_empty_string(&key.name)),
        account_label: None,
        reason: reason.to_string(),
        label: info.label.to_string(),
        impact: info.impact.to_string(),
        recommended_action: info.recommended_action.to_string(),
    }
}

async fn load_account_labels_for_readiness(
    state: &AdminAppState<'_>,
    key_scope_residue: &[NifflerKeyScopeResidue],
    usage_anomalies: &[NifflerUsageAnomaly],
    route_skip_samples: &[NifflerRouteSkipSample],
) -> Result<BTreeMap<String, String>, GatewayError> {
    if !state.has_provider_catalog_data_reader() {
        return Ok(BTreeMap::new());
    }
    let mut key_ids = BTreeSet::new();
    key_ids.extend(key_scope_residue.iter().map(|item| item.key_id.clone()));
    key_ids.extend(
        usage_anomalies
            .iter()
            .filter_map(|item| item.provider_api_key_id.clone()),
    );
    key_ids.extend(
        route_skip_samples
            .iter()
            .filter_map(|item| item.key_id.clone()),
    );
    if key_ids.is_empty() {
        return Ok(BTreeMap::new());
    }

    let key_ids = key_ids.into_iter().collect::<Vec<_>>();
    let keys = state.list_provider_catalog_keys_by_ids(&key_ids).await?;
    Ok(keys
        .into_iter()
        .filter_map(|key| {
            let label = provider_key_account_label(state, &key)?;
            Some((key.id, label))
        })
        .collect())
}

fn apply_account_labels_to_key_residue(
    items: &mut [NifflerKeyScopeResidue],
    labels: &BTreeMap<String, String>,
) {
    for item in items {
        let Some(label) = labels.get(&item.key_id).cloned() else {
            continue;
        };
        item.account_label = Some(label.clone());
        item.display_name = label;
    }
}

fn apply_account_labels_to_usage_anomalies(
    items: &mut [NifflerUsageAnomaly],
    labels: &BTreeMap<String, String>,
) {
    for item in items {
        let Some(key_id) = item.provider_api_key_id.as_deref() else {
            continue;
        };
        item.provider_account_label = labels.get(key_id).cloned();
    }
}

fn apply_account_labels_to_route_skip_samples(
    items: &mut [NifflerRouteSkipSample],
    labels: &BTreeMap<String, String>,
) {
    for item in items {
        let Some(key_id) = item.key_id.as_deref() else {
            continue;
        };
        item.account_label = labels.get(key_id).cloned();
    }
}

struct RouteSkipReasonInfo {
    label: &'static str,
    category: &'static str,
    impact: &'static str,
    recommended_action: &'static str,
}

fn route_skip_reason_info(reason: &str) -> RouteSkipReasonInfo {
    match reason {
        "pool_cooldown" => RouteSkipReasonInfo {
            label: "账号冷却中",
            category: "账号状态",
            impact: "调度器不会选择冷却中的上游账号。",
            recommended_action: "等待冷却结束；如果频繁出现，检查上游错误、冷却时间和账号状态。",
        },
        "pool_account_blocked" => RouteSkipReasonInfo {
            label: "账号被阻止调度",
            category: "账号状态",
            impact: "这把上游账号当前不会参与调度。",
            recommended_action: "检查账号是否被手动停用、风控暂停或标记为不可调度。",
        },
        "pool_account_exhausted" => RouteSkipReasonInfo {
            label: "账号额度耗尽",
            category: "账号额度",
            impact: "调度器会跳过额度耗尽的账号。",
            recommended_action: "等待额度周期重置，或补充可用账号后再处理这个服务。",
        },
        "pool_temporary_unavailable" => RouteSkipReasonInfo {
            label: "账号暂不可用",
            category: "账号状态",
            impact: "账号最近健康检查或调度反馈不可用，暂时不会被选择。",
            recommended_action: "查看账号测试结果和最近上游错误；确认恢复后再让账号参与调度。",
        },
        "pool_cost_limit_reached" => RouteSkipReasonInfo {
            label: "成本窗口超限",
            category: "成本控制",
            impact: "账号在当前成本窗口内达到限制，调度器会跳过它。",
            recommended_action: "检查账号成本窗口、上游成本倍率和模型价格配置。",
        },
        "key_inactive" => RouteSkipReasonInfo {
            label: "账号已停用",
            category: "账号状态",
            impact: "停用账号不会参与调度。",
            recommended_action:
                "如果这个账号仍要使用，在账号管理里恢复；否则从策略或账号池里移除。",
        },
        "oauth_invalid" => RouteSkipReasonInfo {
            label: "OAuth 已失效",
            category: "账号状态",
            impact: "OAuth 失效账号不会参与调度。",
            recommended_action: "重新登录这个 OAuth 账号，或移除失效账号。",
        },
        "key_model_disabled" => RouteSkipReasonInfo {
            label: "账号不允许该模型",
            category: "模型能力",
            impact: "这把账号自己的模型限制排除了本次请求模型。",
            recommended_action: "把模型能力迁到统一账号能力里，确认这个账号是否确实支持该模型。",
        },
        "api_key_concurrency_limit_reached" => RouteSkipReasonInfo {
            label: "用户 Key 并发已满",
            category: "平台限制",
            impact: "请求在选择上游前被平台并发限制拦截。",
            recommended_action: "调整用户 Key 并发限制，或等待该用户正在运行的请求结束。",
        },
        "provider_key_concurrency_limit_reached" => RouteSkipReasonInfo {
            label: "上游账号并发已满",
            category: "账号限制",
            impact: "这把上游账号达到并发限制，调度器会尝试其他可用账号。",
            recommended_action: "检查账号并发配置；如果经常满载，需要增加账号或调整调度权重。",
        },
        "routing_profile_disallowed_key" => RouteSkipReasonInfo {
            label: "产品策略不允许这把账号",
            category: "策略限制",
            impact: "当前用户 Key 绑定的策略不允许使用这把上游账号。",
            recommended_action: "检查产品策略、可用服务和账号范围配置是否符合预期。",
        },
        "transport_snapshot_missing" => RouteSkipReasonInfo {
            label: "连接配置缺失",
            category: "配置缺失",
            impact: "缺少执行请求所需的上游连接信息，无法发起上游请求。",
            recommended_action: "检查 Provider、端点、账号密钥和认证配置是否完整。",
        },
        "pool_active_probe_sealed" => RouteSkipReasonInfo {
            label: "未进入探测热池",
            category: "账号池策略",
            impact: "开启主动探测保护后，未进入热池的账号不会被本次请求选择。",
            recommended_action: "等待探测补充热池；如果长期不足，检查主动探测配置和账号健康状态。",
        },
        "transport_unsupported" => RouteSkipReasonInfo {
            label: "协议不支持",
            category: "协议能力",
            impact: "这个上游服务不支持本次请求需要的协议或能力。",
            recommended_action: "检查 Provider 支持的 API 格式、模型能力和请求类型是否匹配。",
        },
        "transport_api_format_mismatch" => RouteSkipReasonInfo {
            label: "API 格式不匹配",
            category: "协议能力",
            impact: "这次请求的 API 格式和上游账号支持的格式不一致。",
            recommended_action: "检查上游账号支持的 API 格式；必要时启用正确的格式或选择其他服务。",
        },
        "format_conversion_disabled" => RouteSkipReasonInfo {
            label: "格式转换未启用",
            category: "协议能力",
            impact: "请求需要格式转换，但当前服务或账号没有启用对应转换。",
            recommended_action:
                "确认是否允许转换；如果不允许，为这个请求类型配置原生支持的上游服务。",
        },
        _ => RouteSkipReasonInfo {
            label: "未归类跳过原因",
            category: "未归类",
            impact: "后台保留了原始跳过代码，但还没有对应的中文说明。",
            recommended_action:
                "保留原始代码并检查路由记录；如果反复出现，把这个原因补入对账说明。",
        },
    }
}

fn source_field_label(source_field: &str) -> &'static str {
    match source_field {
        "allowed_providers" => "可用 Provider",
        _ => "未知字段",
    }
}

fn residue_field_label(field: &str) -> &'static str {
    match field {
        "api_formats" => "API 格式限制",
        "auth_type_by_format" => "按格式认证方式",
        "allow_auth_channel_mismatch_formats" => "允许认证通道不一致",
        "rate_multipliers" => "成本/倍率覆盖",
        "global_priority_by_format" => "按格式优先级",
        "allowed_models" => "允许模型",
        "locked_models" => "锁定模型",
        "model_include_patterns" => "模型包含规则",
        "model_exclude_patterns" => "模型排除规则",
        _ => "未归类字段",
    }
}

fn provider_key_account_label(
    state: &AdminAppState<'_>,
    key: &StoredProviderCatalogKey,
) -> Option<String> {
    let auth_config = parse_catalog_auth_config_json(state.app(), key);
    provider_key_account_label_from_auth_config(auth_config.as_ref())
}

fn non_empty_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn unix_millis_or_secs_to_secs(value: u64) -> u64 {
    if value > 10_000_000_000 {
        value / 1000
    } else {
        value
    }
}

fn provider_status_counts(providers: &[StoredProviderCatalogProvider]) -> BTreeMap<String, u64> {
    let mut counts = BTreeMap::new();
    for provider in providers {
        let status = if provider.is_active {
            "active"
        } else {
            "disabled"
        };
        *counts.entry(status.to_string()).or_default() += 1;
    }
    counts
}

fn account_status_counts(keys: &[StoredProviderCatalogKey]) -> BTreeMap<String, u64> {
    let mut counts = BTreeMap::new();
    for key in keys {
        *counts.entry(key_status_label(key).to_string()).or_default() += 1;
    }
    counts
}

fn key_status_label(key: &StoredProviderCatalogKey) -> &'static str {
    if !key.is_active {
        "disabled"
    } else if key.oauth_invalid_at_unix_secs.is_some() {
        "invalid"
    } else {
        "available"
    }
}

fn collect_issues(
    state: &AdminAppState<'_>,
    shadow_tables: &NifflerShadowTableStatus,
    disabled_provider_references: &[NifflerDisabledProviderReference],
    key_scope_residue: &[NifflerKeyScopeResidue],
    group_policy_gaps: &[NifflerGroupPolicyGap],
    price_gaps: &[NifflerPriceGap],
    usage_anomalies: &[NifflerUsageAnomaly],
) -> Vec<NifflerReadinessIssue> {
    let mut issues = Vec::new();
    if !shadow_tables.all_present {
        issues.push(issue(
            NifflerReadinessSeverity::Error,
            "shadow_tables_missing",
            "影子表不完整",
            "新模型影子表没有全部创建，不能进入后续迁移。",
        ));
    }
    if !state.has_provider_catalog_data_reader() {
        issues.push(issue(
            NifflerReadinessSeverity::Error,
            "provider_reader_missing",
            "Provider 数据不可读",
            "后台无法读取旧 Provider 和上游账号数据。",
        ));
    }
    if !state.has_global_model_data_reader() {
        issues.push(issue(
            NifflerReadinessSeverity::Error,
            "model_reader_missing",
            "模型数据不可读",
            "后台无法读取模型和价格数据。",
        ));
    }
    if !disabled_provider_references.is_empty() {
        issues.push(issue(
            NifflerReadinessSeverity::Warning,
            "disabled_provider_referenced",
            "停用 Provider 仍被分组引用",
            "用户分组里仍引用了停用 Provider，迁移后这些 Provider 不能被产品策略选择。",
        ));
    }
    if !key_scope_residue.is_empty() {
        issues.push(issue(
            NifflerReadinessSeverity::Warning,
            "key_scope_residue",
            "Key 仍有独立限制",
            "部分上游账号还有模型、格式或优先级限制，需要归入新账号能力或调度策略。",
        ));
    }
    if !group_policy_gaps.is_empty() {
        issues.push(issue(
            NifflerReadinessSeverity::Warning,
            "group_policy_gaps",
            "分组策略需要确认",
            "部分用户分组存在全部模型开放或指定模型列表为空，迁移为产品策略前需要确认。",
        ));
    }
    if !price_gaps.is_empty() {
        issues.push(issue(
            NifflerReadinessSeverity::Warning,
            "price_gaps",
            "价格配置不完整",
            "部分模型没有可用的基础价或 Provider 模型价格，迁移计费前需要补齐。",
        ));
    }
    if !usage_anomalies.is_empty() {
        issues.push(issue(
            NifflerReadinessSeverity::Warning,
            "usage_anomalies",
            "请求记录仍有旧字段问题",
            "最近请求记录里还有 unknown、账号缺失或 pending 结算记录。",
        ));
    }
    issues
}

fn issue(
    severity: NifflerReadinessSeverity,
    code: &str,
    title: &str,
    message: &str,
) -> NifflerReadinessIssue {
    NifflerReadinessIssue {
        severity,
        code: code.to_string(),
        title: title.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_recent_days;

    #[test]
    fn recent_days_is_bounded() {
        assert_eq!(parse_recent_days(Some("recent_days=30")), 30);
        assert_eq!(parse_recent_days(Some("recent_days=0")), 7);
        assert_eq!(parse_recent_days(Some("recent_days=91")), 7);
        assert_eq!(parse_recent_days(None), 7);
    }
}
