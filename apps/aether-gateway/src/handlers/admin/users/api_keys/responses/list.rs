use super::super::super::build_admin_users_bad_request_response;
use super::super::helpers::{
    build_admin_user_api_key_detail_payload, project_admin_user_api_key_product_plan_binding,
};
use super::super::paths::admin_user_id_from_api_keys_path;

use crate::handlers::admin::request::{AdminAppState, AdminRequestContext};
use crate::handlers::admin::shared::query_param_optional_bool;
use crate::GatewayError;
use axum::{
    body::Body,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub(crate) async fn build_admin_list_user_api_keys_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Response<Body>, GatewayError> {
    let Some(user_id) = admin_user_id_from_api_keys_path(request_context.path()) else {
        return Ok(build_admin_users_bad_request_response("缺少 user_id"));
    };
    let Some(user) = state.find_user_auth_by_id(&user_id).await? else {
        return Ok((
            axum::http::StatusCode::NOT_FOUND,
            Json(json!({ "detail": "用户不存在" })),
        )
            .into_response());
    };

    let active_filter = query_param_optional_bool(request_context.query_string(), "is_active");
    let mut export_records = state
        .list_auth_api_key_export_records_by_user_ids(std::slice::from_ref(&user_id))
        .await?;
    if let Some(is_active) = active_filter {
        export_records.retain(|record| record.is_active == is_active);
    }

    let snapshot_ids = export_records
        .iter()
        .map(|record| record.api_key_id.clone())
        .collect::<Vec<_>>();
    let snapshot_by_id = state
        .list_auth_api_key_snapshots_by_ids(&snapshot_ids)
        .await?
        .into_iter()
        .map(|snapshot| (snapshot.api_key_id.clone(), snapshot))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut api_keys = Vec::with_capacity(export_records.len());
    for record in export_records {
        let is_locked = snapshot_by_id
            .get(&record.api_key_id)
            .map(|snapshot| snapshot.api_key_is_locked)
            .unwrap_or(false);
        let mut payload = build_admin_user_api_key_detail_payload(state, &record, is_locked);
        project_admin_user_api_key_product_plan_binding(state, &record.api_key_id, &mut payload)
            .await?;
        api_keys.push(payload);
    }

    Ok(Json(json!({
        "api_keys": api_keys,
        "total": api_keys.len(),
        "user_email": user.email,
        "username": user.username,
    }))
    .into_response())
}
