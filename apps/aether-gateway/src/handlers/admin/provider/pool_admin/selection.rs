use crate::handlers::admin::request::AdminAppState;
use crate::provider_key_auth::provider_key_is_oauth_managed;
use aether_admin::provider::pool as admin_provider_pool_pure;
use aether_data_contracts::repository::provider_catalog::StoredProviderCatalogKey;
use serde_json::Value;
use std::collections::BTreeSet;

fn admin_pool_status_snapshot_bool(key: &StoredProviderCatalogKey, path: &[&str]) -> bool {
    let mut current = key.status_snapshot.as_ref();
    for segment in path {
        current = current.and_then(|value| value.get(*segment));
    }
    current.and_then(Value::as_bool).unwrap_or(false)
}

pub(super) fn admin_pool_normalize_text(value: impl AsRef<str>) -> String {
    admin_provider_pool_pure::admin_pool_normalize_text(value)
}

fn admin_pool_parse_auth_config_json(
    state: &AdminAppState<'_>,
    key: &StoredProviderCatalogKey,
) -> Option<serde_json::Map<String, serde_json::Value>> {
    let ciphertext = key.encrypted_auth_config.as_deref()?.trim();
    if ciphertext.is_empty() {
        return None;
    }
    let plaintext = state.decrypt_catalog_secret_with_fallbacks(ciphertext)?;
    serde_json::from_str::<serde_json::Value>(&plaintext)
        .ok()?
        .as_object()
        .cloned()
}

pub(super) fn admin_pool_derive_plan_tier(
    state: &AdminAppState<'_>,
    key: &StoredProviderCatalogKey,
    provider_type: &str,
) -> Option<String> {
    if !provider_key_is_oauth_managed(key, provider_type) {
        return None;
    }

    let auth_config = admin_pool_parse_auth_config_json(state, key);
    aether_provider_pool::derive_plan_tier(provider_type, key, auth_config.as_ref())
}

pub(super) fn admin_pool_matches_quick_selector(
    state: &AdminAppState<'_>,
    key: &StoredProviderCatalogKey,
    provider_type: &str,
    selector: &str,
) -> bool {
    let oauth_plan_type = admin_pool_derive_plan_tier(state, key, provider_type);
    admin_provider_pool_pure::admin_pool_matches_quick_selector(
        key,
        selector,
        oauth_plan_type.as_deref(),
        admin_provider_pool_pure::admin_pool_now_unix_secs(),
    )
}

pub(super) fn admin_pool_matches_search(
    state: &AdminAppState<'_>,
    key: &StoredProviderCatalogKey,
    provider_type: &str,
    search: Option<&str>,
) -> bool {
    let oauth_plan_type = admin_pool_derive_plan_tier(state, key, provider_type);
    admin_provider_pool_pure::admin_pool_matches_search(key, search, oauth_plan_type.as_deref())
}

pub(super) fn admin_pool_key_status_bucket(
    key: &StoredProviderCatalogKey,
    provider_type: &str,
    cooldown_key_ids: &BTreeSet<String>,
    now_unix_secs: u64,
) -> &'static str {
    let account_blocked = admin_pool_status_snapshot_bool(key, &["account", "blocked"]);
    let account_quota_exhausted =
        admin_provider_pool_pure::admin_pool_key_account_quota_exhausted(key, provider_type);
    let cooldown_reason = cooldown_key_ids
        .contains(&key.id)
        .then_some("pool_cooldown");
    let scheduling = admin_provider_pool_pure::admin_pool_resolve_scheduling_state(
        admin_provider_pool_pure::AdminPoolSchedulingStateInput {
            key,
            now_unix_secs,
            cooldown_reason,
            cooldown_ttl_seconds: None,
            account_blocked,
            account_status_code: None,
            account_status_label: None,
            account_status_reason: None,
            account_status_source: None,
            account_quota_exhausted,
        },
    );
    scheduling.state.code()
}

pub(super) fn admin_pool_status_filter_matches(
    status_filter: &str,
    status_bucket: &str,
    key: &StoredProviderCatalogKey,
) -> bool {
    match status_filter {
        "all" => true,
        "active" => key.is_active,
        "available" => status_bucket == "available",
        "invalid" => status_bucket == "invalid",
        "inactive" | "disabled" => status_bucket == "disabled",
        "quota_exhausted" => status_bucket == "quota_exhausted",
        "cooldown" | "temporary_unavailable" => status_bucket == "temporary_unavailable",
        "blocked" => status_bucket == "blocked",
        _ => true,
    }
}

pub(super) fn admin_pool_key_plan_bucket(
    state: &AdminAppState<'_>,
    key: &StoredProviderCatalogKey,
    provider_type: &str,
) -> String {
    admin_pool_derive_plan_tier(state, key, provider_type)
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

pub(super) fn admin_pool_plan_filter_matches(plan_filter: &str, plan_bucket: &str) -> bool {
    plan_filter == "all" || plan_filter == plan_bucket
}

pub(super) fn admin_pool_key_is_known_banned(key: &StoredProviderCatalogKey) -> bool {
    admin_provider_pool_pure::admin_pool_key_is_known_banned(key)
}

pub(super) fn admin_pool_sort_keys(keys: &mut [StoredProviderCatalogKey]) {
    admin_provider_pool_pure::admin_pool_sort_keys(keys);
}
