use http::Uri;

use crate::control::management_token_required_permission;
use crate::handlers::shared::local_proxy_route_requires_buffered_body;

use super::{classify_control_route, headers, GatewayPublicRequestContext};

#[test]
fn classifies_admin_endpoint_health_api_formats_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/endpoints/health/api-formats?lookback_hours=12"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("endpoints_health"));
    assert_eq!(decision.route_kind.as_deref(), Some("health_api_formats"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:endpoints_health")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_modules_status_list_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/modules/status"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("modules_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("status_list"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:modules")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_modules_status_detail_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/modules/status/oauth"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("modules_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("status_detail"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:modules")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_modules_set_enabled_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/modules/status/management_tokens/enabled"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::PUT, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("modules_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("set_enabled"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:modules")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_readiness_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/readiness?recent_days=7"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(decision.route_kind.as_deref(), Some("readiness"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_upstream_service_writes_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/upstream-services"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::POST, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        decision.route_kind.as_deref(),
        Some("create_upstream_service")
    );
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:providers")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_upstream_account_writes_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/upstream-services/service-1/accounts"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::POST, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        decision.route_kind.as_deref(),
        Some("create_upstream_account")
    );
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:providers")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_upstream_service_capabilities_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/upstream-services/service-1/capabilities"
        .parse()
        .expect("uri should parse");
    let read_decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");
    let write_decision =
        classify_control_route(&http::Method::PUT, &uri, &headers).expect("route should classify");

    assert_eq!(read_decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        read_decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        read_decision.route_kind.as_deref(),
        Some("list_upstream_service_capabilities")
    );
    assert_eq!(
        read_decision.auth_endpoint_signature.as_deref(),
        Some("admin:providers")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::GET, &read_decision).as_deref(),
        Some("admin:providers:read")
    );
    assert!(!read_decision.is_execution_runtime_candidate());

    assert_eq!(write_decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        write_decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        write_decision.route_kind.as_deref(),
        Some("update_upstream_service_capabilities")
    );
    assert_eq!(
        write_decision.auth_endpoint_signature.as_deref(),
        Some("admin:providers")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::PUT, &write_decision).as_deref(),
        Some("admin:providers:write")
    );
    assert!(!write_decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_product_plan_writes_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/product-plans"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::POST, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(decision.route_kind.as_deref(), Some("create_product_plan"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:routing_profiles")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::POST, &decision).as_deref(),
        Some("admin:routing_profiles:write")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_product_plan_model_writes_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/product-plans/plan-1/models"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::POST, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        decision.route_kind.as_deref(),
        Some("upsert_product_plan_model")
    );
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:routing_profiles")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::POST, &decision).as_deref(),
        Some("admin:routing_profiles:write")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_product_plan_api_key_binding_writes_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/product-plans/plan-1/api-key-bindings"
        .parse()
        .expect("uri should parse");
    let read_decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");
    let write_decision =
        classify_control_route(&http::Method::POST, &uri, &headers).expect("route should classify");

    assert_eq!(read_decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        read_decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        read_decision.route_kind.as_deref(),
        Some("list_product_plan_api_key_bindings")
    );
    assert_eq!(
        read_decision.auth_endpoint_signature.as_deref(),
        Some("admin:routing_profiles")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::GET, &read_decision).as_deref(),
        Some("admin:routing_profiles:read")
    );
    assert!(!read_decision.is_execution_runtime_candidate());

    assert_eq!(write_decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        write_decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        write_decision.route_kind.as_deref(),
        Some("upsert_product_plan_api_key_binding")
    );
    assert_eq!(
        write_decision.auth_endpoint_signature.as_deref(),
        Some("admin:routing_profiles")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::POST, &write_decision).as_deref(),
        Some("admin:routing_profiles:write")
    );
    assert!(!write_decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_api_key_product_plan_bindings_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/api-key-product-plan-bindings"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        decision.route_kind.as_deref(),
        Some("list_api_key_product_plan_bindings")
    );
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:routing_profiles")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::GET, &decision).as_deref(),
        Some("admin:routing_profiles:read")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_runtime_rollout_setting_writes_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/runtime-rollout-settings"
        .parse()
        .expect("uri should parse");
    let read_decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");
    let write_decision =
        classify_control_route(&http::Method::POST, &uri, &headers).expect("route should classify");

    assert_eq!(read_decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        read_decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        read_decision.route_kind.as_deref(),
        Some("list_runtime_rollout_settings")
    );
    assert_eq!(
        read_decision.auth_endpoint_signature.as_deref(),
        Some("admin:routing_profiles")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::GET, &read_decision).as_deref(),
        Some("admin:routing_profiles:read")
    );
    assert!(!read_decision.is_execution_runtime_candidate());

    assert_eq!(write_decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        write_decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        write_decision.route_kind.as_deref(),
        Some("upsert_runtime_rollout_setting")
    );
    assert_eq!(
        write_decision.auth_endpoint_signature.as_deref(),
        Some("admin:routing_profiles")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::POST, &write_decision).as_deref(),
        Some("admin:routing_profiles:write")
    );
    assert!(!write_decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_runtime_rollout_preview_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/runtime-rollout-preview?api_key_id=key-1"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        decision.route_kind.as_deref(),
        Some("preview_runtime_rollout")
    );
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:routing_profiles")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::GET, &decision).as_deref(),
        Some("admin:routing_profiles:read")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_error_return_setting_writes_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/error-return-settings"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::POST, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        decision.route_kind.as_deref(),
        Some("create_error_return_setting")
    );
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:routing_profiles")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::POST, &decision).as_deref(),
        Some("admin:routing_profiles:write")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_billing_reservations_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/billing-reservations?status=active"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        decision.route_kind.as_deref(),
        Some("list_billing_reservations")
    );
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:wallets")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::GET, &decision).as_deref(),
        Some("admin:wallets:read")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_settlement_snapshots_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/settlement-snapshots?request_id=req-1"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        decision.route_kind.as_deref(),
        Some("list_settlement_snapshots")
    );
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:wallets")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::GET, &decision).as_deref(),
        Some("admin:wallets:read")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_referral_reward_ledger_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/referral-reward-ledger?status=pending"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(
        decision.route_kind.as_deref(),
        Some("list_referral_reward_ledger")
    );
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:wallets")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::GET, &decision).as_deref(),
        Some("admin:wallets:read")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_niffler_core_route_attempts_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/niffler-core/route-attempts?status=success"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("niffler_core_manage")
    );
    assert_eq!(decision.route_kind.as_deref(), Some("list_route_attempts"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:routing_profiles")
    );
    assert_eq!(
        management_token_required_permission(&http::Method::GET, &decision).as_deref(),
        Some("admin:routing_profiles:read")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn admin_niffler_core_write_routes_buffer_request_body() {
    let headers = headers(&[]);
    let routes = [
        (
            http::Method::POST,
            "/api/admin/niffler-core/upstream-services",
        ),
        (
            http::Method::POST,
            "/api/admin/niffler-core/upstream-services/service-1/accounts",
        ),
        (
            http::Method::PUT,
            "/api/admin/niffler-core/upstream-services/service-1/capabilities",
        ),
        (http::Method::POST, "/api/admin/niffler-core/product-plans"),
        (
            http::Method::POST,
            "/api/admin/niffler-core/product-plans/plan-1/models",
        ),
        (
            http::Method::POST,
            "/api/admin/niffler-core/product-plans/plan-1/api-key-bindings",
        ),
        (
            http::Method::POST,
            "/api/admin/niffler-core/runtime-rollout-settings",
        ),
        (
            http::Method::POST,
            "/api/admin/niffler-core/error-return-settings",
        ),
    ];

    for (method, path) in routes {
        let uri: Uri = path.parse().expect("uri should parse");
        let decision =
            classify_control_route(&method, &uri, &headers).expect("route should classify");
        let context = GatewayPublicRequestContext::from_request_parts(
            "trace-niffler-core-write",
            &method,
            &uri,
            &headers,
            Some(decision),
        );

        assert!(
            local_proxy_route_requires_buffered_body(&context),
            "{method} {path} should buffer request body"
        );
    }
}

#[test]
fn classifies_admin_system_version_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/version"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("version"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_settings_get_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/settings"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("settings_get"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_config_export_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/config/export"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("config_export"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_users_export_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/users/export"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("users_export"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_data_export_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/data/export"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("data_export"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_maintenance_write_routes_as_admin_proxy_route() {
    let headers = headers(&[]);
    let cases = [
        ("/api/admin/system/config/import", "config_import"),
        ("/api/admin/system/users/import", "users_import"),
        ("/api/admin/system/data/import", "data_import"),
        ("/api/admin/system/smtp/test", "smtp_test"),
        ("/api/admin/system/cleanup", "cleanup"),
        ("/api/admin/system/purge/config", "purge_config"),
        ("/api/admin/system/purge/users", "purge_users"),
        ("/api/admin/system/purge/usage", "purge_usage"),
        ("/api/admin/system/purge/audit-logs", "purge_audit_logs"),
        (
            "/api/admin/system/purge/request-bodies",
            "purge_request_bodies",
        ),
        (
            "/api/admin/system/purge/request-bodies/task",
            "purge_request_bodies_task",
        ),
        ("/api/admin/system/purge/stats", "purge_stats"),
    ];

    for (path, expected_kind) in cases {
        let uri: Uri = path.parse().expect("uri should parse");
        let decision = classify_control_route(&http::Method::POST, &uri, &headers)
            .expect("route should classify");

        assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
        assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
        assert_eq!(decision.route_kind.as_deref(), Some(expected_kind));
        assert_eq!(
            decision.auth_endpoint_signature.as_deref(),
            Some("admin:system")
        );
        assert!(!decision.is_execution_runtime_candidate());
    }
}

#[test]
fn classifies_admin_system_check_update_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/check-update"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("check_update"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_aws_regions_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/aws-regions"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("aws_regions"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_stats_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/stats".parse().expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("stats"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_cleanup_runs_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/cleanup/runs"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("cleanup_runs"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_settings_set_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/settings"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::PUT, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("settings_set"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_email_templates_list_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/email/templates"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("email_templates_list"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_email_template_get_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/email/templates/verification"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("email_template_get"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_email_template_set_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/email/templates/verification"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::PUT, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("email_template_set"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_email_template_preview_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/email/templates/verification/preview"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::POST, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(
        decision.route_kind.as_deref(),
        Some("email_template_preview")
    );
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_email_template_reset_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/email/templates/verification/reset"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::POST, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("email_template_reset"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_configs_list_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/configs?limit=20"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("configs_list"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_config_get_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/configs/smtp_password"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("config_get"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_config_set_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/configs/smtp_password"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::PUT, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("config_set"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_config_delete_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/configs/request_log_level"
        .parse()
        .expect("uri should parse");
    let decision = classify_control_route(&http::Method::DELETE, &uri, &headers)
        .expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("config_delete"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_system_api_formats_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/system/api-formats"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("system_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("api_formats"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:system")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_list_providers_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/providers/?skip=0&limit=50"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("providers_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("list_providers"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:providers")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_list_management_tokens_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/management-tokens?limit=20"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("management_tokens_manage")
    );
    assert_eq!(decision.route_kind.as_deref(), Some("list_tokens"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:management_tokens")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_management_token_write_routes_and_permission_catalog() {
    let headers = headers(&[]);
    let cases = [
        (
            http::Method::GET,
            "/api/admin/management-tokens/permissions/catalog",
            "permissions_catalog",
            "admin:management_tokens:read",
        ),
        (
            http::Method::POST,
            "/api/admin/management-tokens",
            "create_token",
            "admin:management_tokens:write",
        ),
        (
            http::Method::PUT,
            "/api/admin/management-tokens/token-123",
            "update_token",
            "admin:management_tokens:write",
        ),
        (
            http::Method::POST,
            "/api/admin/management-tokens/token-123/regenerate",
            "regenerate_token",
            "admin:management_tokens:write",
        ),
    ];

    for (method, path, expected_kind, expected_permission) in cases {
        let uri: Uri = path.parse().expect("uri should parse");
        let decision =
            classify_control_route(&method, &uri, &headers).expect("route should classify");

        assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
        assert_eq!(
            decision.route_family.as_deref(),
            Some("management_tokens_manage")
        );
        assert_eq!(decision.route_kind.as_deref(), Some(expected_kind));
        assert_eq!(
            decision.auth_endpoint_signature.as_deref(),
            Some("admin:management_tokens")
        );
        assert_eq!(
            management_token_required_permission(&method, &decision).as_deref(),
            Some(expected_permission)
        );
    }
}

#[test]
fn classifies_admin_ldap_config_get_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/ldap/config".parse().expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("ldap_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("get_config"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:ldap")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_ldap_config_set_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/ldap/config".parse().expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::PUT, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("ldap_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("set_config"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:ldap")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_ldap_test_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/ldap/test".parse().expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::POST, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("ldap_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("test_connection"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:ldap")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_list_gemini_file_mappings_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/gemini-files/mappings?page=1"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("gemini_files_manage")
    );
    assert_eq!(decision.route_kind.as_deref(), Some("list_mappings"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:gemini_files")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_gemini_file_stats_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/gemini-files/stats"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("gemini_files_manage")
    );
    assert_eq!(decision.route_kind.as_deref(), Some("stats"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:gemini_files")
    );
}

#[test]
fn classifies_admin_delete_gemini_file_mapping_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/gemini-files/mappings/mapping-123"
        .parse()
        .expect("uri should parse");
    let decision = classify_control_route(&http::Method::DELETE, &uri, &headers)
        .expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("gemini_files_manage")
    );
    assert_eq!(decision.route_kind.as_deref(), Some("delete_mapping"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:gemini_files")
    );
}

#[test]
fn classifies_admin_cleanup_gemini_file_mappings_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/gemini-files/mappings"
        .parse()
        .expect("uri should parse");
    let decision = classify_control_route(&http::Method::DELETE, &uri, &headers)
        .expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("gemini_files_manage")
    );
    assert_eq!(decision.route_kind.as_deref(), Some("cleanup_mappings"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:gemini_files")
    );
}

#[test]
fn classifies_admin_list_gemini_file_capable_keys_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/gemini-files/capable-keys"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("gemini_files_manage")
    );
    assert_eq!(decision.route_kind.as_deref(), Some("capable_keys"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:gemini_files")
    );
}

#[test]
fn classifies_admin_gemini_file_upload_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/gemini-files/upload"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::POST, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("gemini_files_manage")
    );
    assert_eq!(decision.route_kind.as_deref(), Some("upload"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:gemini_files")
    );
}

#[test]
fn classifies_admin_oauth_supported_types_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/oauth/supported-types"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("oauth_manage"));
    assert_eq!(decision.route_kind.as_deref(), Some("supported_types"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:oauth")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_provider_oauth_supported_types_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/provider-oauth/supported-types"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(
        decision.route_family.as_deref(),
        Some("provider_oauth_manage")
    );
    assert_eq!(decision.route_kind.as_deref(), Some("supported_types"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:provider_oauth")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_monitoring_trace_request_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/monitoring/trace/request-id-123"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("monitoring"));
    assert_eq!(decision.route_kind.as_deref(), Some("trace_request"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:monitoring")
    );
    assert!(!decision.is_execution_runtime_candidate());
}

#[test]
fn classifies_admin_monitoring_trace_provider_stats_as_admin_proxy_route() {
    let headers = headers(&[]);
    let uri: Uri = "/api/admin/monitoring/trace/stats/provider/provider-id"
        .parse()
        .expect("uri should parse");
    let decision =
        classify_control_route(&http::Method::GET, &uri, &headers).expect("route should classify");

    assert_eq!(decision.route_class.as_deref(), Some("admin_proxy"));
    assert_eq!(decision.route_family.as_deref(), Some("monitoring"));
    assert_eq!(decision.route_kind.as_deref(), Some("trace_provider_stats"));
    assert_eq!(
        decision.auth_endpoint_signature.as_deref(),
        Some("admin:monitoring")
    );
    assert!(!decision.is_execution_runtime_candidate());
}
