use super::{build_admin_users_bad_request_response, build_admin_users_data_unavailable_response};
use crate::handlers::admin::request::{AdminAppState, AdminRequestContext};
use crate::handlers::admin::shared::{attach_admin_audit_response, unix_secs_to_rfc3339};
use crate::handlers::shared::unix_ms_to_rfc3339;
use crate::GatewayError;
use aether_data_contracts::repository::billing::{BillingPlanRecord, UserPlanEntitlementRecord};
use axum::{
    body::{Body, Bytes},
    http,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct AdminGrantUserPlanRequest {
    plan_id: String,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    starts_at: Option<String>,
    #[serde(default)]
    expires_at: Option<String>,
    #[serde(default)]
    initial_remaining_quota_usd: Option<f64>,
}

#[derive(Debug)]
struct AdminGrantPlanOverrides {
    starts_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    initial_remaining_quota_usd: Option<f64>,
}

fn admin_user_id_from_billing_path(request_path: &str, suffix: &str) -> Option<String> {
    let trimmed = request_path.trim_end_matches('/');
    let rest = trimmed.strip_prefix("/api/admin/users/")?;
    let user_id = rest.strip_suffix(suffix)?.trim_end_matches('/');
    if user_id.is_empty() || user_id.contains('/') {
        None
    } else {
        Some(user_id.to_string())
    }
}

fn admin_user_billing_operator_id(request_context: &AdminRequestContext<'_>) -> Option<String> {
    request_context
        .decision()
        .and_then(|decision| decision.admin_principal.as_ref())
        .map(|principal| principal.user_id.clone())
}

fn normalize_admin_grant_reason(value: Option<String>) -> Result<Option<String>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > 512 {
        return Err("reason exceeds maximum length 512".to_string());
    }
    Ok(Some(value.to_string()))
}

fn parse_admin_grant_time(
    value: Option<String>,
    field_name: &str,
) -> Result<Option<DateTime<Utc>>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|value| Some(value.with_timezone(&Utc)))
        .map_err(|_| format!("{field_name} 格式不正确"))
}

fn default_plan_expires_at(plan: &BillingPlanRecord, starts_at: DateTime<Utc>) -> DateTime<Utc> {
    let duration_value = plan.duration_value.max(1);
    match plan.duration_unit.as_str() {
        "day" => starts_at + chrono::Duration::days(duration_value),
        "year" => starts_at + chrono::Duration::days(365 * duration_value),
        "custom" => starts_at + chrono::Duration::days(duration_value),
        _ => starts_at + chrono::Duration::days(30 * duration_value),
    }
}

fn normalize_admin_grant_plan_overrides(
    payload: &AdminGrantUserPlanRequest,
    plan: &BillingPlanRecord,
    now: DateTime<Utc>,
) -> Result<AdminGrantPlanOverrides, String> {
    let requested_starts_at = parse_admin_grant_time(payload.starts_at.clone(), "开始时间")?;
    let requested_expires_at = parse_admin_grant_time(payload.expires_at.clone(), "到期时间")?;
    let has_requested_starts_at = requested_starts_at.is_some();
    let has_requested_expires_at = requested_expires_at.is_some();
    let effective_starts_at = requested_starts_at.unwrap_or(now);
    if effective_starts_at > now {
        return Err("开始时间不能晚于现在".to_string());
    }

    let effective_expires_at =
        requested_expires_at.unwrap_or_else(|| default_plan_expires_at(plan, effective_starts_at));
    if effective_expires_at <= effective_starts_at {
        return Err("到期时间必须晚于开始时间".to_string());
    }
    if effective_expires_at <= now {
        return Err("按这个时间计算，套餐已经过期；请填写新的到期时间".to_string());
    }
    if effective_starts_at.timestamp() < 0 || effective_expires_at.timestamp() < 0 {
        return Err("时间不能早于 1970-01-01".to_string());
    }

    let initial_remaining_quota_usd = match payload.initial_remaining_quota_usd {
        Some(value) if value.is_finite() && value >= 0.0 => Some(value),
        Some(_) => return Err("初始剩余额度不能为负数".to_string()),
        None => None,
    };

    let has_time_override = has_requested_starts_at || has_requested_expires_at;
    Ok(AdminGrantPlanOverrides {
        starts_at: has_time_override.then_some(effective_starts_at),
        expires_at: has_requested_expires_at.then_some(effective_expires_at),
        initial_remaining_quota_usd,
    })
}

fn admin_plan_grant_order_no(now: chrono::DateTime<Utc>) -> String {
    format!(
        "pg_{}_{}",
        now.format("%Y%m%d%H%M%S%6f"),
        &Uuid::new_v4().simple().to_string()[..12]
    )
}

fn billing_plan_payload(record: &BillingPlanRecord) -> serde_json::Value {
    json!({
        "id": record.id,
        "title": record.title,
        "description": record.description,
        "price_amount": record.price_amount,
        "price_currency": record.price_currency,
        "duration_unit": record.duration_unit,
        "duration_value": record.duration_value,
        "enabled": record.enabled,
        "sort_order": record.sort_order,
        "max_active_per_user": record.max_active_per_user,
        "purchase_limit_scope": record.purchase_limit_scope,
        "entitlements": record.entitlements_json,
        "created_at": record.created_at_unix_secs,
        "updated_at": record.updated_at_unix_secs,
    })
}

fn billing_plan_snapshot(record: &BillingPlanRecord) -> serde_json::Value {
    json!({
        "id": record.id,
        "title": record.title,
        "description": record.description,
        "price_amount": record.price_amount,
        "price_currency": record.price_currency,
        "duration_unit": record.duration_unit,
        "duration_value": record.duration_value,
        "max_active_per_user": record.max_active_per_user,
        "purchase_limit_scope": record.purchase_limit_scope,
        "entitlements": record.entitlements_json,
    })
}

fn billing_plan_snapshot_with_admin_grant_overrides(
    record: &BillingPlanRecord,
    overrides: &AdminGrantPlanOverrides,
) -> Value {
    let mut snapshot = billing_plan_snapshot(record);
    let mut override_values = Map::new();
    if let Some(starts_at) = overrides.starts_at.as_ref() {
        override_values.insert(
            "starts_at_unix_secs".to_string(),
            json!(starts_at.timestamp().max(0)),
        );
    }
    if let Some(expires_at) = overrides.expires_at.as_ref() {
        override_values.insert(
            "expires_at_unix_secs".to_string(),
            json!(expires_at.timestamp().max(0)),
        );
    }
    if let Some(value) = overrides.initial_remaining_quota_usd {
        override_values.insert("initial_remaining_quota_usd".to_string(), json!(value));
    }
    if !override_values.is_empty() {
        if let Some(map) = snapshot.as_object_mut() {
            map.insert(
                "admin_grant_overrides".to_string(),
                Value::Object(override_values),
            );
        }
    }
    snapshot
}

fn plan_has_package_rights(record: &BillingPlanRecord) -> bool {
    record.entitlements_json.as_array().is_some_and(|items| {
        items.iter().any(|item| {
            matches!(
                item.get("type").and_then(|value| value.as_str()),
                Some("daily_quota" | "membership_group")
            )
        })
    })
}

fn admin_payment_order_payload(record: &crate::AdminWalletPaymentOrderRecord) -> serde_json::Value {
    json!({
        "id": record.id,
        "order_no": record.order_no,
        "wallet_id": record.wallet_id,
        "user_id": record.user_id,
        "amount_usd": record.amount_usd,
        "pay_amount": record.pay_amount,
        "pay_currency": record.pay_currency,
        "exchange_rate": record.exchange_rate,
        "refunded_amount_usd": record.refunded_amount_usd,
        "refundable_amount_usd": record.refundable_amount_usd,
        "payment_method": record.payment_method,
        "gateway_order_id": record.gateway_order_id,
        "gateway_response": record.gateway_response,
        "status": record.status,
        "order_kind": "plan_purchase",
        "created_at": unix_ms_to_rfc3339(record.created_at_unix_ms),
        "paid_at": record.paid_at_unix_secs.and_then(unix_secs_to_rfc3339),
        "credited_at": record.credited_at_unix_secs.and_then(unix_secs_to_rfc3339),
        "expires_at": record.expires_at_unix_secs.and_then(unix_secs_to_rfc3339),
    })
}

fn entitlement_payload(
    record: &UserPlanEntitlementRecord,
    plan: Option<&BillingPlanRecord>,
    now_unix_secs: u64,
) -> serde_json::Value {
    json!({
        "id": record.id,
        "user_id": record.user_id,
        "plan_id": record.plan_id,
        "payment_order_id": record.payment_order_id,
        "status": record.status,
        "starts_at": unix_secs_to_rfc3339(record.starts_at_unix_secs),
        "expires_at": unix_secs_to_rfc3339(record.expires_at_unix_secs),
        "entitlements": record.entitlements_snapshot,
        "active": record.status == "active"
            && record.starts_at_unix_secs <= now_unix_secs
            && record.expires_at_unix_secs > now_unix_secs,
        "plan_title": plan.map(|plan| plan.title.clone()),
        "plan": plan.map(billing_plan_payload),
        "created_at": unix_secs_to_rfc3339(record.created_at_unix_secs),
        "updated_at": unix_secs_to_rfc3339(record.updated_at_unix_secs),
    })
}

async fn load_admin_user_entitlements_payload(
    state: &AdminAppState<'_>,
    user_id: &str,
) -> Result<Option<serde_json::Value>, GatewayError> {
    let entitlements = match state.app().list_user_plan_entitlements(user_id).await? {
        Some(value) => value,
        None => return Ok(None),
    };
    let plans = state
        .app()
        .list_billing_plans(true)
        .await?
        .unwrap_or_default()
        .into_iter()
        .map(|plan| (plan.id.clone(), plan))
        .collect::<BTreeMap<_, _>>();
    let now = Utc::now().timestamp().max(0) as u64;
    let items = entitlements
        .iter()
        .map(|record| entitlement_payload(record, plans.get(&record.plan_id), now))
        .collect::<Vec<_>>();
    Ok(Some(json!({"items": items, "total": items.len()})))
}

pub(in super::super) async fn build_admin_list_user_billing_entitlements_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    let Some(user_id) =
        admin_user_id_from_billing_path(request_context.path(), "/billing/entitlements")
    else {
        return Ok(build_admin_users_bad_request_response("缺少 user_id"));
    };
    if state.find_user_auth_by_id(&user_id).await?.is_none() {
        return Ok((
            http::StatusCode::NOT_FOUND,
            Json(json!({ "detail": "用户不存在" })),
        )
            .into_response());
    }
    match load_admin_user_entitlements_payload(state, &user_id).await? {
        Some(payload) => Ok(Json(payload).into_response()),
        None => Ok(build_admin_users_data_unavailable_response()),
    }
}

pub(in super::super) async fn build_admin_grant_user_billing_plan_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    request_body: Option<&Bytes>,
) -> Result<Response<Body>, GatewayError> {
    let Some(user_id) =
        admin_user_id_from_billing_path(request_context.path(), "/billing/grant-plan")
    else {
        return Ok(build_admin_users_bad_request_response("缺少 user_id"));
    };
    if state.find_user_auth_by_id(&user_id).await?.is_none() {
        return Ok((
            http::StatusCode::NOT_FOUND,
            Json(json!({ "detail": "用户不存在" })),
        )
            .into_response());
    }
    let Some(body) = request_body else {
        return Ok(build_admin_users_bad_request_response("缺少请求体"));
    };
    let payload = match serde_json::from_slice::<AdminGrantUserPlanRequest>(body) {
        Ok(value) => value,
        Err(_) => return Ok(build_admin_users_bad_request_response("输入验证失败")),
    };
    let plan_id = payload.plan_id.trim();
    if plan_id.is_empty() {
        return Ok(build_admin_users_bad_request_response("plan_id 不能为空"));
    }
    let reason = match normalize_admin_grant_reason(payload.reason.clone()) {
        Ok(value) => value,
        Err(detail) => return Ok(build_admin_users_bad_request_response(detail)),
    };
    let Some(plan) = state.app().find_billing_plan(plan_id).await? else {
        return Ok((
            http::StatusCode::NOT_FOUND,
            Json(json!({ "detail": "套餐不存在" })),
        )
            .into_response());
    };
    if !plan_has_package_rights(&plan) {
        return Ok(build_admin_users_bad_request_response(
            "余额包已移除，请使用钱包充值功能",
        ));
    }

    let now = Utc::now();
    let grant_overrides = match normalize_admin_grant_plan_overrides(&payload, &plan, now) {
        Ok(value) => value,
        Err(detail) => return Ok(build_admin_users_bad_request_response(detail)),
    };
    let order_no = admin_plan_grant_order_no(now);
    let operator_id = admin_user_billing_operator_id(request_context);
    let gateway_response = json!({
        "source": "admin_grant",
        "operator_id": operator_id.as_deref(),
        "reason": reason,
        "granted_at": now.to_rfc3339(),
        "starts_at": grant_overrides.starts_at.as_ref().map(|value| value.to_rfc3339()),
        "expires_at": grant_overrides.expires_at.as_ref().map(|value| value.to_rfc3339()),
        "initial_remaining_quota_usd": grant_overrides.initial_remaining_quota_usd,
    });
    let outcome = match state
        .app()
        .create_plan_purchase_order(
            aether_data::repository::wallet::CreatePlanPurchaseOrderInput {
                preferred_wallet_id: None,
                user_id: user_id.clone(),
                amount_usd: 0.0,
                pay_amount: 0.0,
                pay_currency: plan.price_currency.clone(),
                exchange_rate: 1.0,
                payment_method: "admin_grant".to_string(),
                payment_provider: Some("admin".to_string()),
                payment_channel: Some("manual".to_string()),
                gateway_order_id: order_no.clone(),
                gateway_response,
                order_no: order_no.clone(),
                product_id: plan.id.clone(),
                product_snapshot: billing_plan_snapshot_with_admin_grant_overrides(
                    &plan,
                    &grant_overrides,
                ),
                expires_at_unix_secs: (now + chrono::Duration::minutes(30)).timestamp().max(0)
                    as u64,
            },
        )
        .await?
    {
        Some(value) => value,
        None => return Ok(build_admin_users_data_unavailable_response()),
    };
    let order = match outcome {
        aether_data::repository::wallet::CreatePlanPurchaseOrderOutcome::Created(order) => order,
        aether_data::repository::wallet::CreatePlanPurchaseOrderOutcome::WalletInactive => {
            return Ok(build_admin_users_bad_request_response(
                "wallet is not active",
            ));
        }
        aether_data::repository::wallet::CreatePlanPurchaseOrderOutcome::ActivePlanLimitReached => {
            return Ok((
                http::StatusCode::CONFLICT,
                Json(json!({ "detail": "套餐购买限制已达到上限" })),
            )
                .into_response());
        }
    };

    let credit_result = state
        .admin_credit_payment_order(
            &order.id,
            Some(&order_no),
            Some(0.0),
            Some(&plan.price_currency),
            Some(1.0),
            Some(json!({ "admin_grant": true })),
            operator_id.as_deref(),
        )
        .await?;
    let (credited_order, credited) = match credit_result {
        crate::AdminWalletMutationOutcome::Applied(value) => value,
        crate::AdminWalletMutationOutcome::NotFound => {
            return Ok(build_admin_users_data_unavailable_response());
        }
        crate::AdminWalletMutationOutcome::Invalid(detail) => {
            return Ok((
                http::StatusCode::CONFLICT,
                Json(json!({ "detail": detail })),
            )
                .into_response());
        }
        crate::AdminWalletMutationOutcome::Unavailable => {
            return Ok(build_admin_users_data_unavailable_response());
        }
    };
    let entitlements = match load_admin_user_entitlements_payload(state, &user_id).await? {
        Some(value) => value,
        None => return Ok(build_admin_users_data_unavailable_response()),
    };
    Ok(attach_admin_audit_response(
        Json(json!({
            "order": admin_payment_order_payload(&credited_order),
            "credited": credited,
            "items": entitlements["items"].clone(),
            "entitlements": entitlements["items"].clone(),
            "total": entitlements["total"].clone(),
        }))
        .into_response(),
        "admin_user_plan_granted",
        "grant_user_billing_plan",
        "user",
        &user_id,
    ))
}
