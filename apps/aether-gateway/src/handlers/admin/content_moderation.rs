use crate::handlers::admin::request::{AdminAppState, AdminRequestContext};
use crate::handlers::admin::shared::attach_admin_audit_response;
use crate::GatewayError;
use aether_data::repository::content_moderation_evidence::StoredContentModerationEvidence;
use axum::{
    body::Body,
    http,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

const EVIDENCE_PATH_PREFIX: &str = "/api/admin/content-moderation/evidence/";

pub(crate) async fn maybe_build_local_admin_content_moderation_response(
    request: crate::handlers::admin::request::AdminRouteRequest<'_>,
) -> crate::handlers::admin::request::AdminRouteResult {
    build_content_moderation_response(&request.state(), &request.request_context()).await
}

async fn build_content_moderation_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
) -> Result<Option<Response<Body>>, GatewayError> {
    if request_context.route_family() != Some("content_moderation_manage")
        || request_context.route_kind() != Some("view_evidence")
        || request_context.method() != http::Method::GET
    {
        return Ok(None);
    }

    let Some(evidence_id) = content_moderation_evidence_id_from_path(request_context.path()) else {
        return Ok(Some(
            (
                http::StatusCode::NOT_FOUND,
                Json(json!({ "detail": "内容审查证据不存在" })),
            )
                .into_response(),
        ));
    };

    let Some(evidence) = state
        .app()
        .data
        .find_content_moderation_evidence_by_id(&evidence_id)
        .await
        .map_err(|err| GatewayError::Internal(err.to_string()))?
    else {
        return Ok(Some(
            (
                http::StatusCode::NOT_FOUND,
                Json(json!({ "detail": "内容审查证据不存在" })),
            )
                .into_response(),
        ));
    };

    let payload = content_moderation_evidence_payload(state, evidence).await?;
    Ok(Some(attach_admin_audit_response(
        Json(payload).into_response(),
        "content_moderation_evidence_viewed",
        "view_content_moderation_evidence",
        "content_moderation_evidence",
        &evidence_id,
    )))
}

fn content_moderation_evidence_id_from_path(path: &str) -> Option<String> {
    let normalized_path = path.trim_end_matches('/');
    normalized_path
        .strip_prefix(EVIDENCE_PATH_PREFIX)
        .filter(|id| !id.is_empty() && !id.contains('/'))
        .map(ToOwned::to_owned)
}

async fn content_moderation_evidence_payload(
    state: &AdminAppState<'_>,
    evidence: StoredContentModerationEvidence,
) -> Result<serde_json::Value, GatewayError> {
    let username =
        resolve_content_moderation_evidence_username(state, evidence.user_id.as_ref()).await?;
    let api_key_name =
        resolve_content_moderation_evidence_api_key_name(state, evidence.api_key_id.as_ref())
            .await?;
    let provider_name =
        resolve_content_moderation_evidence_provider_name(state, evidence.provider_id.as_ref())
            .await?;
    let upstream_service_name = resolve_content_moderation_evidence_upstream_service_name(
        state,
        evidence.upstream_service_id.as_ref(),
    )
    .await?;
    let upstream_account_name = resolve_content_moderation_evidence_upstream_account_name(
        state,
        evidence.upstream_account_id.as_ref(),
    )
    .await?;

    Ok(json!({
        "id": evidence.id,
        "request_id": evidence.request_id,
        "user_id": evidence.user_id,
        "username": username,
        "api_key_id": evidence.api_key_id,
        "api_key_name": api_key_name,
        "provider_id": evidence.provider_id,
        "provider_name": provider_name,
        "upstream_service_id": evidence.upstream_service_id,
        "upstream_service_name": upstream_service_name,
        "upstream_account_id": evidence.upstream_account_id,
        "upstream_account_name": upstream_account_name,
        "moderation_model": evidence.moderation_model,
        "input_sha256": evidence.input_sha256,
        "input_text": evidence.input_text,
        "categories": evidence.categories,
        "category_scores": evidence.category_scores,
        "flagged": evidence.flagged,
        "created_at_unix_secs": evidence.created_at_unix_secs,
        "expires_at_unix_secs": evidence.expires_at_unix_secs,
        "redacted_at_unix_secs": evidence.redacted_at_unix_secs,
    }))
}

async fn resolve_content_moderation_evidence_username(
    state: &AdminAppState<'_>,
    user_id: Option<&String>,
) -> Result<Option<String>, GatewayError> {
    let Some(user_id) = user_id else {
        return Ok(None);
    };
    let users = state
        .list_users_by_ids(std::slice::from_ref(user_id))
        .await?;
    Ok(users
        .into_iter()
        .find(|user| user.id == *user_id)
        .map(|user| user.username))
}

async fn resolve_content_moderation_evidence_api_key_name(
    state: &AdminAppState<'_>,
    api_key_id: Option<&String>,
) -> Result<Option<String>, GatewayError> {
    let Some(api_key_id) = api_key_id else {
        return Ok(None);
    };
    let mut names = state
        .resolve_auth_api_key_names_by_ids(std::slice::from_ref(api_key_id))
        .await?;
    Ok(names.remove(api_key_id))
}

async fn resolve_content_moderation_evidence_provider_name(
    state: &AdminAppState<'_>,
    provider_id: Option<&String>,
) -> Result<Option<String>, GatewayError> {
    let Some(provider_id) = provider_id else {
        return Ok(None);
    };
    let providers = state
        .read_provider_catalog_providers_by_ids(std::slice::from_ref(provider_id))
        .await?;
    Ok(providers
        .into_iter()
        .find(|provider| provider.id == *provider_id)
        .map(|provider| provider.name))
}

async fn resolve_content_moderation_evidence_upstream_service_name(
    state: &AdminAppState<'_>,
    upstream_service_id: Option<&String>,
) -> Result<Option<String>, GatewayError> {
    let Some(upstream_service_id) = upstream_service_id else {
        return Ok(None);
    };
    let endpoints = state
        .read_provider_catalog_endpoints_by_ids(std::slice::from_ref(upstream_service_id))
        .await?;
    Ok(endpoints
        .into_iter()
        .find(|endpoint| endpoint.id == *upstream_service_id)
        .map(|endpoint| endpoint.api_format))
}

async fn resolve_content_moderation_evidence_upstream_account_name(
    state: &AdminAppState<'_>,
    upstream_account_id: Option<&String>,
) -> Result<Option<String>, GatewayError> {
    let Some(upstream_account_id) = upstream_account_id else {
        return Ok(None);
    };
    let keys = state
        .list_provider_catalog_keys_by_ids(std::slice::from_ref(upstream_account_id))
        .await?;
    Ok(keys
        .into_iter()
        .find(|key| key.id == *upstream_account_id)
        .map(|key| key.name))
}
