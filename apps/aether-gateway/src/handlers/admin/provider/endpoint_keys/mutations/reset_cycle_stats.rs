use crate::handlers::admin::niffler_legacy_freeze::maybe_freeze_migrated_legacy_provider_key_write;
use crate::handlers::admin::provider::pool_admin::payloads::admin_pool_codex_window_usage_requests;
use crate::handlers::admin::provider::shared::paths::admin_reset_cycle_stats_key_id;
use crate::handlers::admin::request::{AdminAppState, AdminRequestContext};
use crate::handlers::admin::shared::attach_admin_audit_response;
use crate::GatewayError;
use axum::{
    body::{Body, Bytes},
    http,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) async fn maybe_handle(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    _request_body: Option<&Bytes>,
) -> Result<Option<Response<Body>>, GatewayError> {
    let Some(decision) = request_context.decision() else {
        return Ok(None);
    };
    if decision.route_family.as_deref() != Some("endpoints_manage")
        || decision.route_kind.as_deref() != Some("reset_cycle_stats")
        || request_context.method() != http::Method::POST
        || !request_context
            .path()
            .starts_with("/api/admin/endpoints/keys/")
        || !request_context.path().ends_with("/reset-cycle-stats")
    {
        return Ok(None);
    }

    let Some(key_id) = admin_reset_cycle_stats_key_id(request_context.path()) else {
        return Ok(Some(not_found_response("Key 不存在")));
    };
    if let Some(response) = maybe_freeze_migrated_legacy_provider_key_write(state, &key_id).await? {
        return Ok(Some(response));
    }
    let Some(key) = state
        .read_provider_catalog_keys_by_ids(std::slice::from_ref(&key_id))
        .await?
        .into_iter()
        .next()
    else {
        return Ok(Some(not_found_response(format!("Key {key_id} 不存在"))));
    };
    let Some(provider) = state
        .read_provider_catalog_providers_by_ids(std::slice::from_ref(&key.provider_id))
        .await?
        .into_iter()
        .next()
    else {
        return Ok(Some(not_found_response(format!(
            "Provider {} 不存在",
            key.provider_id
        ))));
    };
    if !provider.provider_type.trim().eq_ignore_ascii_case("codex") {
        return Ok(Some(bad_request_response(
            "仅 Codex Provider 支持重置周期统计",
        )));
    }

    let now_unix_secs = current_unix_secs();
    let windows = admin_pool_codex_window_usage_requests(&key, &provider.provider_type);
    let reset_windows = state
        .reset_provider_api_key_codex_window_usage_stats(&windows, now_unix_secs)
        .await?;
    if reset_windows == 0 {
        return Ok(Some(bad_request_response("当前账号没有可重置的周期窗口")));
    }

    Ok(Some(attach_admin_audit_response(
        Json(json!({
            "message": "已重置周期统计",
            "reset_at": now_unix_secs,
            "windows": reset_windows,
        }))
        .into_response(),
        "admin_provider_key_cycle_stats_reset",
        "reset_provider_key_cycle_stats",
        "provider_key",
        &key_id,
    )))
}

fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn bad_request_response(detail: impl Into<String>) -> Response<Body> {
    (
        http::StatusCode::BAD_REQUEST,
        Json(json!({ "detail": detail.into() })),
    )
        .into_response()
}

fn not_found_response(detail: impl Into<String>) -> Response<Body> {
    (
        http::StatusCode::NOT_FOUND,
        Json(json!({ "detail": detail.into() })),
    )
        .into_response()
}
