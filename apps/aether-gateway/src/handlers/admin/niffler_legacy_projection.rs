use crate::handlers::admin::shared::unix_secs_to_rfc3339;
use aether_data_contracts::repository::niffler_core::{
    NifflerAccountStatus, StoredNifflerAccountModelCapability, StoredNifflerProductPlan,
    StoredNifflerProductPlanModel, StoredNifflerUpstreamAccount, StoredNifflerUpstreamService,
};
use serde_json::{json, Value};

pub(crate) const NIFFLER_CORE_PAGE: &str = "/admin/niffler-core";

const PROVIDER_READ_ONLY_REASON: &str =
    "这个旧 Provider 已迁移到 Niffler Core 上游服务，旧入口只读。";
const PROVIDER_KEY_READ_ONLY_REASON: &str =
    "这个旧 Provider Key 已迁移到 Niffler Core 上游账号，旧入口只读。";
const PROVIDER_MODEL_READ_ONLY_REASON: &str =
    "这个旧 Provider 模型列表已迁移到 Niffler Core 账号模型能力，旧入口只读。";
const USER_GROUP_READ_ONLY_REASON: &str =
    "这个旧用户分组已迁移到 Niffler Core 产品策略，旧入口只读。";
const USER_KEY_GROUP_BINDING_READ_ONLY_REASON: &str =
    "这把用户 Key 已绑定 Niffler Core 产品策略，旧分组绑定只读。";

fn unix_ms_to_rfc3339(value: u64) -> Option<String> {
    unix_secs_to_rfc3339(value / 1000)
}

fn put_common_read_only_projection(
    object: &mut serde_json::Map<String, Value>,
    source_type: &'static str,
    source_id: &str,
    reason: &'static str,
) {
    object.insert("niffler_core_projected".to_string(), json!(true));
    object.insert("legacy_read_only".to_string(), json!(true));
    object.insert("niffler_core_page".to_string(), json!(NIFFLER_CORE_PAGE));
    object.insert("legacy_read_only_reason".to_string(), json!(reason));
    object.insert("niffler_core_source_type".to_string(), json!(source_type));
    object.insert("niffler_core_source_id".to_string(), json!(source_id));
}

pub(crate) fn project_provider_summary_with_niffler_service(
    payload: &mut Value,
    service: &StoredNifflerUpstreamService,
) {
    let Some(object) = payload.as_object_mut() else {
        return;
    };
    put_common_read_only_projection(
        object,
        "upstream_service",
        &service.id,
        PROVIDER_READ_ONLY_REASON,
    );
    object.insert("name".to_string(), json!(service.display_name));
    object.insert("provider_type".to_string(), json!(service.service_kind));
    object.insert("is_active".to_string(), json!(service.is_active));
    object.insert(
        "cost_multiplier".to_string(),
        json!(service.cost_multiplier),
    );
    object.insert(
        "niffler_core_upstream_service_id".to_string(),
        json!(service.id),
    );
    object.insert(
        "niffler_core_upstream_service_name".to_string(),
        json!(service.display_name),
    );
    object.insert(
        "default_api_format".to_string(),
        json!(service.default_api_format),
    );
    if let Some(api_format) = service.default_api_format.as_deref() {
        object.insert("api_format".to_string(), json!(api_format));
        object.insert("api_formats".to_string(), json!([api_format]));
    }
    object.insert("base_url".to_string(), json!(service.base_url));
    object.insert(
        "created_at".to_string(),
        json!(unix_ms_to_rfc3339(service.created_at_unix_ms)),
    );
    object.insert(
        "updated_at".to_string(),
        json!(unix_ms_to_rfc3339(service.updated_at_unix_ms)),
    );
}

fn account_status_label(status: NifflerAccountStatus) -> &'static str {
    match status {
        NifflerAccountStatus::Available => "可用",
        NifflerAccountStatus::Disabled => "停用",
        NifflerAccountStatus::Invalid => "失效",
        NifflerAccountStatus::QuotaExhausted => "额度耗尽",
        NifflerAccountStatus::CoolingDown => "冷却中",
    }
}

fn account_scheduling_state(status: NifflerAccountStatus) -> &'static str {
    match status {
        NifflerAccountStatus::Available => "available",
        NifflerAccountStatus::Disabled => "disabled",
        NifflerAccountStatus::Invalid => "invalid",
        NifflerAccountStatus::QuotaExhausted => "quota_exhausted",
        NifflerAccountStatus::CoolingDown => "temporary_unavailable",
    }
}

fn account_scheduling_reason(status: NifflerAccountStatus) -> &'static str {
    match status {
        NifflerAccountStatus::Available => "niffler_core_available",
        NifflerAccountStatus::Disabled => "niffler_core_disabled",
        NifflerAccountStatus::Invalid => "niffler_core_invalid",
        NifflerAccountStatus::QuotaExhausted => "niffler_core_quota_exhausted",
        NifflerAccountStatus::CoolingDown => "niffler_core_cooling_down",
    }
}

pub(crate) fn project_provider_key_with_niffler_account(
    payload: &mut Value,
    account: &StoredNifflerUpstreamAccount,
    now_unix_secs: u64,
) {
    let Some(object) = payload.as_object_mut() else {
        return;
    };
    let label = account_status_label(account.status);
    let blocking = !account.status.allows_scheduling();
    let ttl_seconds = account
        .cooldown_until_unix_ms
        .map(|value| value / 1000)
        .and_then(|until_secs| until_secs.checked_sub(now_unix_secs));

    put_common_read_only_projection(
        object,
        "upstream_account",
        &account.id,
        PROVIDER_KEY_READ_ONLY_REASON,
    );
    object.insert("name".to_string(), json!(account.display_name));
    object.insert("oauth_email".to_string(), json!(account.email));
    object.insert("oauth_phone".to_string(), json!(account.phone));
    object.insert("auth_type".to_string(), json!(account.auth_kind));
    object.insert("internal_priority".to_string(), json!(account.priority));
    object.insert(
        "niffler_core_upstream_account_id".to_string(),
        json!(account.id),
    );
    object.insert(
        "niffler_core_upstream_account_name".to_string(),
        json!(account.display_name),
    );
    object.insert(
        "niffler_core_account_status".to_string(),
        json!(account.status.as_str()),
    );
    object.insert(
        "niffler_core_cost_multiplier".to_string(),
        json!(account.cost_multiplier),
    );
    object.insert(
        "cooldown_until_unix_ms".to_string(),
        json!(account.cooldown_until_unix_ms),
    );
    object.insert(
        "last_tested_at_unix_ms".to_string(),
        json!(account.last_tested_at_unix_ms),
    );
    object.insert(
        "last_test_error".to_string(),
        json!(account.last_test_error),
    );
    object.insert(
        "is_active".to_string(),
        json!(account.status != NifflerAccountStatus::Disabled),
    );
    object.insert(
        "scheduling_state".to_string(),
        json!(account_scheduling_state(account.status)),
    );
    object.insert(
        "scheduling_status".to_string(),
        json!(if blocking { "blocked" } else { "available" }),
    );
    object.insert(
        "scheduling_reason".to_string(),
        json!(account_scheduling_reason(account.status)),
    );
    object.insert("scheduling_reason_label".to_string(), json!(label));
    object.insert("scheduling_label".to_string(), json!(label));
    object.insert("scheduling_blocking".to_string(), json!(blocking));
    object.insert("scheduling_ttl_seconds".to_string(), json!(ttl_seconds));
    object.insert(
        "created_at".to_string(),
        json!(unix_ms_to_rfc3339(account.created_at_unix_ms)),
    );
    object.insert(
        "updated_at".to_string(),
        json!(unix_ms_to_rfc3339(account.updated_at_unix_ms)),
    );
}

pub(crate) fn niffler_account_model_capability_projection(
    service: &StoredNifflerUpstreamService,
    capability: &StoredNifflerAccountModelCapability,
) -> Value {
    let mut payload = json!({
        "id": capability.id,
        "provider_id": service.id,
        "global_model_id": capability.model_name,
        "provider_model_name": capability.model_name,
        "provider_model_mappings": [],
        "config": null,
        "price_per_request": null,
        "cost_multiplier": service.cost_multiplier,
        "tiered_pricing": null,
        "supports_vision": null,
        "supports_function_calling": null,
        "supports_streaming": null,
        "supports_extended_thinking": null,
        "supports_image_generation": null,
        "supports_embedding": null,
        "effective_tiered_pricing": null,
        "effective_input_price": null,
        "effective_output_price": null,
        "effective_price_per_request": null,
        "effective_supports_vision": null,
        "effective_supports_function_calling": null,
        "effective_supports_streaming": null,
        "effective_supports_extended_thinking": null,
        "effective_supports_image_generation": null,
        "effective_supports_embedding": null,
        "is_active": service.is_active && capability.is_enabled,
        "is_available": service.is_active && capability.is_enabled,
        "created_at": unix_ms_to_rfc3339(capability.created_at_unix_ms),
        "updated_at": unix_ms_to_rfc3339(capability.updated_at_unix_ms),
        "global_model_name": capability.model_name,
        "global_model_display_name": capability.model_name,
        "effective_config": {
            "niffler_core_source": "account_model_capability",
            "upstream_account_id": capability.upstream_account_id,
            "source": capability.source,
            "last_checked_at_unix_ms": capability.last_checked_at_unix_ms,
            "last_error": capability.last_error,
        },
        "model_test_capabilities": {},
        "niffler_core_account_model_capability_id": capability.id,
    });
    if let Some(object) = payload.as_object_mut() {
        put_common_read_only_projection(
            object,
            "account_model_capability",
            &capability.id,
            PROVIDER_MODEL_READ_ONLY_REASON,
        );
    }
    payload
}

pub(crate) fn product_plan_user_group_projection(
    plan: &StoredNifflerProductPlan,
    models: &[StoredNifflerProductPlanModel],
    default_group_id: Option<&str>,
) -> Value {
    let allowed_models = models
        .iter()
        .filter(|model| model.is_enabled)
        .map(|model| model.model_name.clone())
        .collect::<Vec<_>>();
    let model_sales_multipliers = models
        .iter()
        .filter_map(|model| {
            model
                .sales_multiplier_override
                .map(|value| (model.model_name.clone(), json!(value)))
        })
        .collect::<serde_json::Map<_, _>>();
    let mut payload = json!({
        "id": plan.id,
        "name": plan.display_name,
        "normalized_name": plan.display_name.to_ascii_lowercase(),
        "description": plan.description,
        "visibility": if plan.is_public { "public" } else { "internal" },
        "sales_multiplier": plan.sales_multiplier,
        "model_sales_multipliers": Value::Object(model_sales_multipliers),
        "allowed_providers": null,
        "allowed_providers_mode": "unrestricted",
        "allowed_api_formats": null,
        "allowed_api_formats_mode": "unrestricted",
        "allowed_models": allowed_models,
        "allowed_models_mode": "specific",
        "rate_limit": null,
        "rate_limit_mode": "system",
        "concurrent_limit": null,
        "concurrent_limit_mode": "system",
        "is_default": default_group_id == Some(plan.id.as_str()),
        "is_active": plan.is_active,
        "created_at": unix_ms_to_rfc3339(plan.created_at_unix_ms),
        "updated_at": unix_ms_to_rfc3339(plan.updated_at_unix_ms),
        "niffler_core_product_plan_id": plan.id,
        "niffler_core_product_plan_name": plan.display_name,
    });
    if let Some(object) = payload.as_object_mut() {
        put_common_read_only_projection(
            object,
            "product_plan",
            &plan.id,
            USER_GROUP_READ_ONLY_REASON,
        );
    }
    payload
}

pub(crate) fn project_user_api_key_with_product_plan(
    payload: &mut Value,
    binding_id: &str,
    plan: &StoredNifflerProductPlan,
) {
    let Some(object) = payload.as_object_mut() else {
        return;
    };
    object.insert("niffler_core_projected".to_string(), json!(true));
    object.insert(
        "niffler_core_product_plan_projected".to_string(),
        json!(true),
    );
    object.insert("legacy_group_binding_read_only".to_string(), json!(true));
    object.insert(
        "legacy_group_binding_read_only_reason".to_string(),
        json!(USER_KEY_GROUP_BINDING_READ_ONLY_REASON),
    );
    object.insert("niffler_core_page".to_string(), json!(NIFFLER_CORE_PAGE));
    object.insert("niffler_core_binding_id".to_string(), json!(binding_id));
    object.insert("group_id".to_string(), json!(plan.id));
    object.insert("group_name".to_string(), json!(plan.display_name));
    object.insert("niffler_core_product_plan_id".to_string(), json!(plan.id));
    object.insert(
        "niffler_core_product_plan_name".to_string(),
        json!(plan.display_name),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> StoredNifflerUpstreamService {
        StoredNifflerUpstreamService {
            id: "provider-1".to_string(),
            display_name: "Codex 服务".to_string(),
            service_kind: "codex".to_string(),
            default_api_format: Some("openai:responses".to_string()),
            base_url: Some("https://example.test".to_string()),
            cost_multiplier: 0.6,
            is_active: false,
            config: None,
            created_at_unix_ms: 1_700_000_000_000,
            updated_at_unix_ms: 1_700_000_001_000,
        }
    }

    fn account(status: NifflerAccountStatus) -> StoredNifflerUpstreamAccount {
        StoredNifflerUpstreamAccount {
            id: "key-1".to_string(),
            upstream_service_id: "provider-1".to_string(),
            display_name: "账号 1".to_string(),
            email: Some("user@example.test".to_string()),
            phone: Some("15000000000".to_string()),
            auth_kind: "oauth".to_string(),
            status,
            cost_multiplier: 0.7,
            priority: 12,
            cooldown_until_unix_ms: Some(1_700_000_600_000),
            last_tested_at_unix_ms: Some(1_700_000_010_000),
            last_test_error: Some("需要重新登录".to_string()),
            config: None,
            created_at_unix_ms: 1_700_000_000_000,
            updated_at_unix_ms: 1_700_000_001_000,
        }
    }

    fn plan() -> StoredNifflerProductPlan {
        StoredNifflerProductPlan {
            id: "group-1".to_string(),
            display_name: "Plus 策略".to_string(),
            is_public: true,
            is_active: true,
            sales_multiplier: 1.8,
            description: Some("产品策略".to_string()),
            created_at_unix_ms: 1_700_000_000_000,
            updated_at_unix_ms: 1_700_000_001_000,
        }
    }

    #[test]
    fn provider_projection_uses_upstream_service_fields() {
        let mut payload = json!({
            "id": "provider-1",
            "name": "旧 Provider",
            "is_active": true,
            "cost_multiplier": 1.0,
        });

        project_provider_summary_with_niffler_service(&mut payload, &service());

        assert_eq!(payload["name"], "Codex 服务");
        assert_eq!(payload["provider_type"], "codex");
        assert_eq!(payload["api_format"], "openai:responses");
        assert_eq!(payload["is_active"], false);
        assert_eq!(payload["cost_multiplier"], 0.6);
        assert_eq!(payload["niffler_core_projected"], true);
        assert_eq!(payload["legacy_read_only"], true);
        assert_eq!(payload["niffler_core_page"], NIFFLER_CORE_PAGE);
    }

    #[test]
    fn account_projection_uses_upstream_account_status() {
        let mut payload = json!({
            "id": "key-1",
            "name": "旧 Key",
            "is_active": true,
            "scheduling_state": "available",
        });

        project_provider_key_with_niffler_account(
            &mut payload,
            &account(NifflerAccountStatus::QuotaExhausted),
            1_700_000_000,
        );

        assert_eq!(payload["name"], "账号 1");
        assert_eq!(payload["oauth_email"], "user@example.test");
        assert_eq!(payload["oauth_phone"], "15000000000");
        assert_eq!(payload["niffler_core_account_status"], "quota_exhausted");
        assert_eq!(payload["scheduling_state"], "quota_exhausted");
        assert_eq!(payload["scheduling_blocking"], true);
        assert_eq!(payload["internal_priority"], 12);
        assert_eq!(payload["legacy_read_only"], true);
    }

    #[test]
    fn model_projection_marks_account_model_capability_read_only() {
        let capability = StoredNifflerAccountModelCapability {
            id: "cap-1".to_string(),
            upstream_service_id: "provider-1".to_string(),
            upstream_account_id: "key-1".to_string(),
            model_name: "gpt-5.5".to_string(),
            is_enabled: true,
            source: Some("manual".to_string()),
            last_checked_at_unix_ms: None,
            last_error: None,
            created_at_unix_ms: 1_700_000_000_000,
            updated_at_unix_ms: 1_700_000_001_000,
        };

        let payload = niffler_account_model_capability_projection(&service(), &capability);

        assert_eq!(payload["provider_id"], "provider-1");
        assert_eq!(payload["provider_model_name"], "gpt-5.5");
        assert_eq!(payload["cost_multiplier"], 0.6);
        assert_eq!(payload["legacy_read_only"], true);
        assert_eq!(payload["niffler_core_account_model_capability_id"], "cap-1");
    }

    #[test]
    fn product_plan_projection_uses_plan_models() {
        let models = vec![
            StoredNifflerProductPlanModel {
                id: "model-1".to_string(),
                product_plan_id: "group-1".to_string(),
                model_name: "gpt-5.5".to_string(),
                is_enabled: true,
                sales_multiplier_override: Some(2.0),
                created_at_unix_ms: 1_700_000_000_000,
                updated_at_unix_ms: 1_700_000_001_000,
            },
            StoredNifflerProductPlanModel {
                id: "model-2".to_string(),
                product_plan_id: "group-1".to_string(),
                model_name: "disabled-model".to_string(),
                is_enabled: false,
                sales_multiplier_override: None,
                created_at_unix_ms: 1_700_000_000_000,
                updated_at_unix_ms: 1_700_000_001_000,
            },
        ];

        let payload = product_plan_user_group_projection(&plan(), &models, Some("group-1"));

        assert_eq!(payload["name"], "Plus 策略");
        assert_eq!(payload["sales_multiplier"], 1.8);
        assert_eq!(payload["allowed_models"], json!(["gpt-5.5"]));
        assert_eq!(payload["model_sales_multipliers"]["gpt-5.5"], 2.0);
        assert_eq!(payload["is_default"], true);
        assert_eq!(payload["legacy_read_only"], true);
    }

    #[test]
    fn user_key_projection_marks_group_binding_only_read_only() {
        let mut payload = json!({
            "id": "api-key-1",
            "group_id": "old-group",
            "group_name": "旧分组",
        });

        project_user_api_key_with_product_plan(&mut payload, "binding-1", &plan());

        assert_eq!(payload["group_id"], "group-1");
        assert_eq!(payload["group_name"], "Plus 策略");
        assert_eq!(payload["legacy_group_binding_read_only"], true);
        assert_eq!(payload["niffler_core_binding_id"], "binding-1");
        assert!(payload.get("legacy_read_only").is_none());
    }
}
