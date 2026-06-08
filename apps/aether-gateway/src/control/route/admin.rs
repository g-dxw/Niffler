use axum::http;

use super::{classified, ClassifiedRoute};

#[path = "admin/basic_families.rs"]
mod basic_families;
#[path = "admin/endpoints_families.rs"]
mod endpoints_families;
#[path = "admin/model_provider_families.rs"]
mod model_provider_families;
#[path = "admin/observability_families.rs"]
mod observability_families;
#[path = "admin/operations_families.rs"]
mod operations_families;
#[path = "admin/provider_ops_routes.rs"]
mod provider_ops_routes;
#[path = "admin/routing_families.rs"]
mod routing_families;
#[path = "admin/system_families.rs"]
mod system_families;

use basic_families::classify_admin_basic_family_route;
use endpoints_families::classify_admin_endpoints_family_route;
use model_provider_families::classify_admin_model_provider_family_route;
use observability_families::classify_admin_observability_family_route;
use operations_families::classify_admin_operations_family_route;
use provider_ops_routes::classify_admin_provider_ops_routes;
use routing_families::classify_admin_routing_family_route;
use system_families::classify_admin_system_family_route;

pub(super) fn classify_admin_route(
    method: &http::Method,
    normalized_path: &str,
) -> Option<ClassifiedRoute> {
    let normalized_path_no_trailing = normalized_path.trim_end_matches('/');
    let normalized_path_no_trailing = if normalized_path_no_trailing.is_empty() {
        "/"
    } else {
        normalized_path_no_trailing
    };

    if method == http::Method::GET
        && matches!(
            normalized_path,
            "/api/admin/providers" | "/api/admin/providers/"
        )
    {
        Some(classified(
            "admin_proxy",
            "providers_manage",
            "list_providers",
            "admin:providers",
            false,
        ))
    } else if method == http::Method::GET
        && normalized_path_no_trailing == "/api/admin/niffler-core/readiness"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            "readiness",
            "admin:system",
            false,
        ))
    } else if method == http::Method::GET
        && normalized_path_no_trailing == "/api/admin/niffler-core/legacy-dependency-audit"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            "legacy_dependency_audit",
            "admin:system",
            false,
        ))
    } else if matches!(method, &http::Method::GET | &http::Method::POST)
        && normalized_path_no_trailing == "/api/admin/niffler-core/upstream-services"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            if method == http::Method::GET {
                "list_upstream_services"
            } else {
                "create_upstream_service"
            },
            "admin:providers",
            false,
        ))
    } else if matches!(method, &http::Method::GET | &http::Method::POST)
        && normalized_path_no_trailing
            .strip_prefix("/api/admin/niffler-core/upstream-services/")
            .and_then(|rest| rest.strip_suffix("/accounts"))
            .is_some_and(|id| !id.is_empty() && !id.contains('/'))
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            if method == http::Method::GET {
                "list_upstream_accounts"
            } else {
                "create_upstream_account"
            },
            "admin:providers",
            false,
        ))
    } else if matches!(method, &http::Method::GET | &http::Method::PUT)
        && normalized_path_no_trailing
            .strip_prefix("/api/admin/niffler-core/upstream-services/")
            .and_then(|rest| rest.strip_suffix("/capabilities"))
            .is_some_and(|id| !id.is_empty() && !id.contains('/'))
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            if method == http::Method::GET {
                "list_upstream_service_capabilities"
            } else {
                "update_upstream_service_capabilities"
            },
            "admin:providers",
            false,
        ))
    } else if matches!(method, &http::Method::GET | &http::Method::POST)
        && normalized_path_no_trailing == "/api/admin/niffler-core/product-plans"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            if method == http::Method::GET {
                "list_product_plans"
            } else {
                "create_product_plan"
            },
            "admin:routing_profiles",
            false,
        ))
    } else if matches!(method, &http::Method::GET | &http::Method::POST)
        && normalized_path_no_trailing
            .strip_prefix("/api/admin/niffler-core/product-plans/")
            .and_then(|rest| rest.strip_suffix("/models"))
            .is_some_and(|id| !id.is_empty() && !id.contains('/'))
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            if method == http::Method::GET {
                "list_product_plan_models"
            } else {
                "upsert_product_plan_model"
            },
            "admin:routing_profiles",
            false,
        ))
    } else if matches!(method, &http::Method::GET | &http::Method::POST)
        && normalized_path_no_trailing
            .strip_prefix("/api/admin/niffler-core/product-plans/")
            .and_then(|rest| rest.strip_suffix("/api-key-bindings"))
            .is_some_and(|id| !id.is_empty() && !id.contains('/'))
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            if method == http::Method::GET {
                "list_product_plan_api_key_bindings"
            } else {
                "upsert_product_plan_api_key_binding"
            },
            "admin:routing_profiles",
            false,
        ))
    } else if method == http::Method::GET
        && normalized_path_no_trailing == "/api/admin/niffler-core/api-key-product-plan-bindings"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            "list_api_key_product_plan_bindings",
            "admin:routing_profiles",
            false,
        ))
    } else if matches!(method, &http::Method::GET | &http::Method::POST)
        && normalized_path_no_trailing == "/api/admin/niffler-core/runtime-rollout-settings"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            if method == http::Method::GET {
                "list_runtime_rollout_settings"
            } else {
                "upsert_runtime_rollout_setting"
            },
            "admin:routing_profiles",
            false,
        ))
    } else if method == http::Method::GET
        && normalized_path_no_trailing == "/api/admin/niffler-core/runtime-rollout-preview"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            "preview_runtime_rollout",
            "admin:routing_profiles",
            false,
        ))
    } else if matches!(method, &http::Method::GET | &http::Method::POST)
        && normalized_path_no_trailing == "/api/admin/niffler-core/error-return-settings"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            if method == http::Method::GET {
                "list_error_return_settings"
            } else {
                "create_error_return_setting"
            },
            "admin:routing_profiles",
            false,
        ))
    } else if method == http::Method::GET
        && normalized_path_no_trailing == "/api/admin/niffler-core/billing-reservations"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            "list_billing_reservations",
            "admin:wallets",
            false,
        ))
    } else if method == http::Method::GET
        && normalized_path_no_trailing == "/api/admin/niffler-core/billing-reservation-dry-runs"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            "list_billing_reservation_dry_runs",
            "admin:wallets",
            false,
        ))
    } else if method == http::Method::GET
        && normalized_path_no_trailing == "/api/admin/niffler-core/settlement-snapshots"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            "list_settlement_snapshots",
            "admin:wallets",
            false,
        ))
    } else if method == http::Method::GET
        && normalized_path_no_trailing == "/api/admin/niffler-core/referral-reward-ledger"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            "list_referral_reward_ledger",
            "admin:wallets",
            false,
        ))
    } else if method == http::Method::POST
        && normalized_path_no_trailing
            .strip_prefix("/api/admin/niffler-core/referral-reward-ledger/")
            .is_some_and(|rest| {
                !rest.is_empty()
                    && (rest.ends_with("/retry") || rest.ends_with("/cancel"))
                    && rest.split('/').count() == 2
            })
    {
        let route_kind = if normalized_path_no_trailing.ends_with("/retry") {
            "retry_referral_reward_ledger"
        } else {
            "cancel_referral_reward_ledger"
        };
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            route_kind,
            "admin:wallets",
            false,
        ))
    } else if method == http::Method::GET
        && normalized_path_no_trailing == "/api/admin/niffler-core/route-attempts"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            "list_route_attempts",
            "admin:routing_profiles",
            false,
        ))
    } else if method == http::Method::GET
        && normalized_path_no_trailing == "/api/admin/niffler-core/consistency-checks"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            "list_consistency_checks",
            "admin:wallets",
            false,
        ))
    } else if method == http::Method::GET
        && normalized_path_no_trailing == "/api/admin/niffler-core/stability-observations"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            "list_stability_observations",
            "admin:system",
            false,
        ))
    } else if (method == http::Method::GET || method == http::Method::PUT)
        && normalized_path_no_trailing == "/api/admin/niffler-core/rollback-drill-evidence"
    {
        Some(classified(
            "admin_proxy",
            "niffler_core_manage",
            "rollback_drill_evidence",
            "admin:system",
            false,
        ))
    } else if let Some(route) =
        classify_admin_basic_family_route(method, normalized_path, normalized_path_no_trailing)
    {
        Some(route)
    } else if let Some(route) = classify_admin_observability_family_route(
        method,
        normalized_path,
        normalized_path_no_trailing,
    ) {
        Some(route)
    } else if let Some(route) =
        classify_admin_operations_family_route(method, normalized_path, normalized_path_no_trailing)
    {
        Some(route)
    } else if let Some(route) =
        classify_admin_system_family_route(method, normalized_path, normalized_path_no_trailing)
    {
        Some(route)
    } else if let Some(route) =
        classify_admin_routing_family_route(method, normalized_path_no_trailing)
    {
        Some(route)
    } else if let Some(route) = classify_admin_provider_ops_routes(method, normalized_path) {
        Some(route)
    } else if let Some(route) = classify_admin_model_provider_family_route(method, normalized_path)
    {
        Some(route)
    } else {
        classify_admin_endpoints_family_route(method, normalized_path)
    }
}
