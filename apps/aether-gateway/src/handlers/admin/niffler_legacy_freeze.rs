use crate::handlers::admin::request::AdminAppState;
use crate::handlers::admin::shared::attach_admin_audit_response;
use crate::GatewayError;
use axum::{
    body::Body,
    http,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};

const FROZEN_EVENT_NAME: &str = "niffler_legacy_write_frozen";
const FROZEN_ACTION: &str = "freeze_legacy_write";
const CORE_PAGE: &str = "/admin/niffler-core";

pub(crate) async fn maybe_freeze_migrated_legacy_provider_write(
    state: &AdminAppState<'_>,
    provider_id: &str,
) -> Result<Option<Response<Body>>, GatewayError> {
    maybe_freeze_migrated_legacy_provider_child_write(state, provider_id, "旧 Provider", "上游服务")
        .await
}

pub(crate) async fn maybe_freeze_migrated_legacy_provider_model_write(
    state: &AdminAppState<'_>,
    provider_id: &str,
) -> Result<Option<Response<Body>>, GatewayError> {
    maybe_freeze_migrated_legacy_provider_child_write(
        state,
        provider_id,
        "旧 Provider 模型价格",
        "上游服务模型价格",
    )
    .await
}

pub(crate) async fn maybe_freeze_migrated_legacy_provider_endpoint_write(
    state: &AdminAppState<'_>,
    provider_id: &str,
) -> Result<Option<Response<Body>>, GatewayError> {
    maybe_freeze_migrated_legacy_provider_child_write(
        state,
        provider_id,
        "旧 Provider 端点",
        "上游服务",
    )
    .await
}

pub(crate) async fn maybe_freeze_migrated_legacy_provider_key_collection_write(
    state: &AdminAppState<'_>,
    provider_id: &str,
) -> Result<Option<Response<Body>>, GatewayError> {
    maybe_freeze_migrated_legacy_provider_child_write(
        state,
        provider_id,
        "旧 Provider Key",
        "上游账号",
    )
    .await
}

async fn maybe_freeze_migrated_legacy_provider_child_write(
    state: &AdminAppState<'_>,
    provider_id: &str,
    legacy_name: &'static str,
    destination: &'static str,
) -> Result<Option<Response<Body>>, GatewayError> {
    if state
        .find_niffler_upstream_service_by_id(provider_id)
        .await?
        .is_none()
    {
        return Ok(None);
    }
    Ok(Some(legacy_write_frozen_response(
        "provider",
        provider_id,
        legacy_name,
        destination,
    )))
}

pub(crate) async fn maybe_freeze_migrated_legacy_provider_key_write(
    state: &AdminAppState<'_>,
    key_id: &str,
) -> Result<Option<Response<Body>>, GatewayError> {
    if state
        .find_niffler_upstream_account_by_id(key_id)
        .await?
        .is_none()
    {
        return Ok(None);
    }
    Ok(Some(legacy_write_frozen_response(
        "provider_key",
        key_id,
        "旧 Provider Key",
        "上游账号",
    )))
}

pub(crate) async fn maybe_freeze_migrated_legacy_user_group_write(
    state: &AdminAppState<'_>,
    group_id: &str,
) -> Result<Option<Response<Body>>, GatewayError> {
    if state
        .find_niffler_product_plan_by_id(group_id)
        .await?
        .is_none()
    {
        return Ok(None);
    }
    Ok(Some(legacy_write_frozen_response(
        "user_group",
        group_id,
        "旧用户分组",
        "产品策略",
    )))
}

pub(crate) async fn maybe_freeze_migrated_legacy_api_key_product_plan_write(
    state: &AdminAppState<'_>,
    api_key_id: &str,
) -> Result<Option<Response<Body>>, GatewayError> {
    if state
        .find_niffler_api_key_product_plan_binding_by_api_key_id(api_key_id)
        .await?
        .is_none()
    {
        return Ok(None);
    }
    Ok(Some(legacy_write_frozen_response(
        "user_api_key",
        api_key_id,
        "旧用户 Key 分组绑定",
        "用户 Key 产品策略绑定",
    )))
}

fn legacy_write_frozen_response(
    target_type: &'static str,
    target_id: &str,
    legacy_name: &'static str,
    destination: &'static str,
) -> Response<Body> {
    attach_admin_audit_response(
        (
            http::StatusCode::CONFLICT,
            Json(legacy_write_frozen_payload(
                target_type,
                target_id,
                legacy_name,
                destination,
            )),
        )
            .into_response(),
        FROZEN_EVENT_NAME,
        FROZEN_ACTION,
        target_type,
        target_id,
    )
}

fn legacy_write_frozen_payload(
    target_type: &'static str,
    target_id: &str,
    legacy_name: &'static str,
    destination: &'static str,
) -> Value {
    json!({
        "code": "niffler_legacy_write_frozen",
        "detail": format!("{legacy_name} 已经迁移到 Niffler Core。请到「Niffler Core / {destination}」修改，旧入口已只读。"),
        "target_type": target_type,
        "target_id": target_id,
        "destination": destination,
        "page": CORE_PAGE,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AdminAuditEvent;

    #[test]
    fn frozen_payload_points_admin_to_niffler_core_page() {
        let payload =
            legacy_write_frozen_payload("provider", "provider-1", "旧 Provider", "上游服务");

        assert_eq!(payload["code"], "niffler_legacy_write_frozen");
        assert_eq!(payload["target_type"], "provider");
        assert_eq!(payload["target_id"], "provider-1");
        assert_eq!(payload["page"], CORE_PAGE);
        assert!(payload["detail"]
            .as_str()
            .expect("detail should be string")
            .contains("Niffler Core / 上游服务"));
    }

    #[test]
    fn frozen_response_attaches_admin_audit_event() {
        let response =
            legacy_write_frozen_response("provider_key", "key-1", "旧 Provider Key", "上游账号");

        assert_eq!(response.status(), http::StatusCode::CONFLICT);
        let event = response
            .extensions()
            .get::<AdminAuditEvent>()
            .expect("frozen response should attach audit event");
        assert_eq!(event.event_name, FROZEN_EVENT_NAME);
        assert_eq!(event.action, FROZEN_ACTION);
        assert_eq!(event.target_type, "provider_key");
        assert_eq!(event.target_id, "key-1");
    }
}
