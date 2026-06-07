use std::collections::BTreeMap;

use aether_data::repository::users::StoredUserGroup;
use aether_data_contracts::repository::global_models::{
    AdminGlobalModelListQuery, AdminProviderModelListQuery, StoredAdminGlobalModel,
    StoredAdminProviderModel,
};
use aether_data_contracts::repository::niffler_core::{
    NifflerCoreMappingSummary, NifflerCoreReadinessReport, NifflerCoreReadinessSummary,
    NifflerDisabledProviderReference, NifflerGroupPolicyGap, NifflerKeyScopeResidue,
    NifflerPriceGap, NifflerReadinessIssue, NifflerReadinessSeverity,
    NifflerRouteSkipReasonSummary, NifflerShadowTableItem, NifflerShadowTableStatus,
    NifflerUsageAnomaly,
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
use crate::handlers::admin::shared::query_param_value;
use crate::GatewayError;

const READINESS_PATH: &str = "/api/admin/niffler-core/readiness";
const MAX_ISSUE_ITEMS: usize = 50;
const MAX_USAGE_SCAN: usize = 200;
const MAX_USAGE_ITEMS: usize = 50;
const MAX_PROVIDER_MODELS_PER_PROVIDER: usize = 2_000;
const MAX_GLOBAL_MODELS: usize = 10_000;
const MAX_ROUTE_SKIP_SAMPLE: usize = 500;
const SHADOW_TABLES: &[&str] = &[
    "niffler_upstream_services",
    "niffler_upstream_accounts",
    "niffler_product_plans",
    "niffler_product_plan_models",
    "niffler_model_base_prices",
    "niffler_upstream_model_prices",
    "niffler_account_model_capabilities",
    "niffler_upstream_service_capabilities",
    "niffler_settlement_snapshots",
    "niffler_billing_reservations",
    "niffler_billing_reservation_events",
    "niffler_route_attempts",
    "niffler_error_return_settings",
    "niffler_account_risk_events",
    "niffler_api_key_pauses",
    "niffler_referral_reward_rules",
    "niffler_referral_reward_ledger",
    "niffler_referral_reward_events",
];

pub(crate) async fn maybe_build_local_admin_niffler_response(
    request: AdminRouteRequest<'_>,
) -> Result<Option<Response<Body>>, GatewayError> {
    let state = request.state();
    let request_context = request.request_context();

    if request_context.route_family() != Some("niffler_core_manage")
        || request_context.path() != READINESS_PATH
    {
        return Ok(None);
    }
    if request_context.method() != http::Method::GET {
        return Ok(Some(
            (
                http::StatusCode::METHOD_NOT_ALLOWED,
                Json(serde_json::json!({ "detail": "只支持只读检查" })),
            )
                .into_response(),
        ));
    }

    let recent_days = parse_recent_days(request_context.query_string());
    let report = build_readiness_report(&state, recent_days).await?;
    Ok(Some(Json(report).into_response()))
}

fn parse_recent_days(query_string: Option<&str>) -> u32 {
    query_param_value(query_string, "recent_days")
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| (1..=90).contains(value))
        .unwrap_or(7)
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

    let disabled_provider_references =
        collect_disabled_provider_references(&user_groups, &provider_map);
    let key_scope_residue = collect_key_scope_residue(&keys);
    let group_policy_gaps = collect_group_policy_gaps(&user_groups);
    let price_gaps = collect_price_gaps(&global_models, &provider_models, &provider_map);
    let (recent_usage_anomalies, recent_problem_usage_sample_count) =
        collect_recent_usage_anomalies(state, recent_days, generated_at_unix_secs).await?;
    let route_skip_reasons = collect_route_skip_reasons(state).await?;
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
    });
}

fn collect_key_scope_residue(keys: &[StoredProviderCatalogKey]) -> Vec<NifflerKeyScopeResidue> {
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
        residue.push(NifflerKeyScopeResidue {
            subject_kind: "provider_key".to_string(),
            key_id: key.id.clone(),
            key_name: Some(key.name.clone()),
            owner_label: Some(key.provider_id.clone()),
            residue_fields: fields,
            impact: "这把上游账号仍有独立限制，迁移前需要归入账号能力或调度策略。".to_string(),
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
                message:
                    "这个用户分组当前允许全部模型；迁移为产品策略前需要确认是否继续开放全部模型。"
                        .to_string(),
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
                message:
                    "这个用户分组设置为只允许指定模型，但模型列表为空；迁移前需要明确可售模型。"
                        .to_string(),
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
            provider_id: None,
            provider_name: None,
            model_id: Some(model.id.clone()),
            model_name: model.name.clone(),
            missing_fields: vec![
                "default_price_per_request".to_string(),
                "default_tiered_pricing".to_string(),
            ],
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
        anomalies.push(NifflerUsageAnomaly {
            usage_id: row.id,
            request_id: row.request_id,
            created_at_unix_secs: row.created_at_unix_ms,
            provider_name: row.provider_name,
            provider_id: row.provider_id,
            provider_api_key_id: row.provider_api_key_id,
            model: row.model,
            status: row.status,
            billing_status: row.billing_status,
            status_code: row.status_code,
            error_category: row.error_category,
            diagnosis,
        });
        if anomalies.len() >= MAX_USAGE_ITEMS {
            break;
        }
    }
    let count = anomalies.len() as u64;
    Ok((anomalies, count))
}

fn usage_anomaly_diagnosis(row: &StoredRequestUsageAudit) -> Option<String> {
    let provider_unknown = row.provider_name.trim().eq_ignore_ascii_case("unknown")
        || row.provider_name.trim().is_empty()
        || row.provider_id.is_none();
    if provider_unknown && is_api_key_concurrency_limited(row) {
        return Some(
            "平台在选择上游前拦截了这个请求：用户 API Key 并发数已达上限，所以没有实际 Provider 或账号。"
                .to_string(),
        );
    }
    if provider_unknown {
        return Some(
            "这条记录没有实际 Provider ID，失败发生在选定上游前，或旧记录没有保存可展示的上游服务。"
                .to_string(),
        );
    }
    if row.billing_status.trim().eq_ignore_ascii_case("pending")
        && row.status.trim().eq_ignore_ascii_case("completed")
    {
        return Some(
            "请求已完成，但结算没有最终完成；当前记录没有可展示的钱包扣费快照。".to_string(),
        );
    }
    if row.billing_status.trim().eq_ignore_ascii_case("pending") {
        return Some("请求仍在进行或等待超时清理，暂时没有最终扣费拆分。".to_string());
    }
    if row.status.trim().eq_ignore_ascii_case("failed") && row.provider_api_key_id.is_none() {
        return Some(
            "这条失败记录没有上游账号 ID，说明失败发生在选定账号前或旧记录缺少账号快照。"
                .to_string(),
        );
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

async fn collect_route_skip_reasons(
    state: &AdminAppState<'_>,
) -> Result<Vec<NifflerRouteSkipReasonSummary>, GatewayError> {
    if !state.has_request_candidate_data_reader() {
        return Ok(Vec::new());
    }
    let rows = state
        .read_recent_request_candidates(MAX_ROUTE_SKIP_SAMPLE)
        .await?;
    let mut counts = BTreeMap::<String, u64>::new();
    for row in rows {
        let Some(reason) = row.skip_reason else {
            continue;
        };
        if reason.trim().is_empty() {
            continue;
        }
        *counts.entry(reason).or_default() += 1;
    }
    let mut summaries = counts
        .into_iter()
        .map(|(reason, count)| NifflerRouteSkipReasonSummary { reason, count })
        .collect::<Vec<_>>();
    summaries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(left.reason.cmp(&right.reason))
    });
    summaries.truncate(MAX_ISSUE_ITEMS);
    Ok(summaries)
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
