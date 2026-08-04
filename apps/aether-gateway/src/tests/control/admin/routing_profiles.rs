use http::StatusCode;
use serde_json::json;

use super::super::{build_router_with_state, start_server, AppState};
use crate::constants::{
    GATEWAY_HEADER, TRUSTED_ADMIN_SESSION_ID_HEADER, TRUSTED_ADMIN_USER_ID_HEADER,
    TRUSTED_ADMIN_USER_ROLE_HEADER,
};

#[tokio::test]
async fn gateway_returns_managed_instruction_profile_registry_from_routing_admin() {
    let gateway = build_router_with_state(AppState::new().expect("gateway should build"));
    let (gateway_url, gateway_handle) = start_server(gateway).await;

    let response = reqwest::Client::new()
        .get(format!(
            "{gateway_url}/api/admin/routing/managed-instruction-profiles"
        ))
        .header(GATEWAY_HEADER, "rust-phase3b")
        .header(TRUSTED_ADMIN_USER_ID_HEADER, "admin-user-123")
        .header(TRUSTED_ADMIN_USER_ROLE_HEADER, "admin")
        .header(TRUSTED_ADMIN_SESSION_ID_HEADER, "session-123")
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value = response.json().await.expect("json body should parse");
    let profiles = payload["profiles"].as_array().expect("profiles array");
    assert_eq!(profiles.len(), 2);
    assert_eq!(profiles[0]["profile_id"], json!("security_research_v1"));
    assert_eq!(profiles[1]["profile_id"], json!("adult_fiction_v1"));
    assert!(profiles.iter().all(|profile| {
        profile["profile_sha256"]
            .as_str()
            .is_some_and(|value| value.len() == 64)
            && profile.get("embedded_text").is_none()
    }));
    assert_eq!(payload["merge_modes"], json!(["prepend", "if_missing"]));
    assert_eq!(
        payload["supported_provider_api_formats"],
        json!(["openai:responses", "openai:chat", "claude:messages"])
    );

    gateway_handle.abort();
}
