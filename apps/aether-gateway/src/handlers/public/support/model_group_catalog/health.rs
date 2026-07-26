use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

use aether_data_contracts::repository::global_models::{
    StoredAdminProviderModel, StoredPublicGlobalModel,
};
use aether_data_contracts::repository::provider_catalog::{
    StoredProviderCatalogEndpoint, StoredProviderCatalogKey, StoredProviderCatalogProvider,
};
use aether_scheduler_core::{
    is_provider_key_circuit_open, matches_model_mapping, provider_key_health_score,
};
use serde::Serialize;
use serde_json::json;

use crate::handlers::shared::{
    json_string_list, provider_catalog_key_supports_format, provider_key_scheduling_state_payload,
    provider_key_status_snapshot_payload,
};
use crate::AppState;

#[derive(Clone, Serialize)]
pub(super) struct PublicModelHealthSummary {
    status: &'static str,
    score: Option<f64>,
    active_providers: usize,
    active_endpoints: usize,
    providers: BTreeSet<String>,
}

impl Default for PublicModelHealthSummary {
    fn default() -> Self {
        Self {
            status: "unavailable",
            score: None,
            active_providers: 0,
            active_endpoints: 0,
            providers: BTreeSet::new(),
        }
    }
}

pub(super) async fn load_public_model_health_snapshot(
    state: &AppState,
    models: &[StoredPublicGlobalModel],
) -> BTreeMap<String, PublicModelHealthSummary> {
    let mut health_by_model = models
        .iter()
        .map(|model| (model.id.clone(), PublicModelHealthSummary::default()))
        .collect::<BTreeMap<_, _>>();
    if models.is_empty() || !state.has_global_model_data_reader() {
        return health_by_model;
    }

    let model_ids = models
        .iter()
        .map(|model| model.id.clone())
        .collect::<Vec<_>>();
    let provider_models = match state
        .list_admin_provider_models_by_global_model_ids(&model_ids)
        .await
    {
        Ok(items) => items,
        Err(_) => return health_by_model,
    };
    let provider_ids = provider_models
        .iter()
        .map(|model| model.provider_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if provider_ids.is_empty() || !state.has_provider_catalog_data_reader() {
        return health_by_model;
    }

    let (providers, endpoints, keys) = tokio::join!(
        state.read_provider_catalog_providers_by_ids(&provider_ids),
        state.list_provider_catalog_endpoints_by_provider_ids(&provider_ids),
        state.list_provider_catalog_keys_by_provider_ids(&provider_ids),
    );
    let Ok(providers) = providers else {
        return health_by_model;
    };
    let endpoints = endpoints.unwrap_or_default();
    let keys = keys.unwrap_or_default();

    let providers_by_id = providers
        .into_iter()
        .map(|provider| (provider.id.clone(), provider))
        .collect::<BTreeMap<_, _>>();
    let mut endpoints_by_provider = BTreeMap::<String, Vec<StoredProviderCatalogEndpoint>>::new();
    for endpoint in endpoints {
        endpoints_by_provider
            .entry(endpoint.provider_id.clone())
            .or_default()
            .push(endpoint);
    }
    let mut keys_by_provider = BTreeMap::<String, Vec<StoredProviderCatalogKey>>::new();
    for key in keys {
        keys_by_provider
            .entry(key.provider_id.clone())
            .or_default()
            .push(key);
    }
    let mut provider_models_by_global_model =
        BTreeMap::<String, Vec<StoredAdminProviderModel>>::new();
    for provider_model in provider_models {
        provider_models_by_global_model
            .entry(provider_model.global_model_id.clone())
            .or_default()
            .push(provider_model);
    }

    let now_unix_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    for model in models {
        let summary = summarize_model_health(
            model,
            provider_models_by_global_model
                .get(&model.id)
                .map(Vec::as_slice)
                .unwrap_or_default(),
            &providers_by_id,
            &endpoints_by_provider,
            &keys_by_provider,
            now_unix_secs,
        );
        health_by_model.insert(model.id.clone(), summary);
    }

    health_by_model
}

fn summarize_model_health(
    model: &StoredPublicGlobalModel,
    provider_models: &[StoredAdminProviderModel],
    providers_by_id: &BTreeMap<String, StoredProviderCatalogProvider>,
    endpoints_by_provider: &BTreeMap<String, Vec<StoredProviderCatalogEndpoint>>,
    keys_by_provider: &BTreeMap<String, Vec<StoredProviderCatalogKey>>,
    now_unix_secs: u64,
) -> PublicModelHealthSummary {
    let mut active_providers = 0usize;
    let mut active_endpoints = 0usize;
    let mut scores = Vec::new();
    let mut provider_names = BTreeSet::new();
    let global_model_mappings = model
        .config
        .as_ref()
        .and_then(|value| value.get("model_mappings"))
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    for provider_model in provider_models.iter().filter(|item| item.is_active) {
        let Some(provider) = providers_by_id
            .get(&provider_model.provider_id)
            .filter(|provider| provider.is_active)
        else {
            continue;
        };
        let provider_model_mapping_names =
            provider_model_mapping_names(provider_model.provider_model_mappings.as_ref());
        let key_match_model_names = key_match_model_names(
            &model.name,
            &provider_model.provider_model_name,
            &provider_model_mapping_names,
        );
        let mut provider_available = false;

        for endpoint in endpoints_by_provider
            .get(&provider.id)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .filter(|endpoint| endpoint.is_active)
        {
            let endpoint_keys = keys_by_provider
                .get(&provider.id)
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .filter(|key| {
                    provider_catalog_key_supports_format(
                        key,
                        &provider.provider_type,
                        &endpoint.api_format,
                    )
                })
                .filter(|key| {
                    key_allowed_models_match_global_model(
                        key.allowed_models.as_ref(),
                        &key_match_model_names,
                        &global_model_mappings,
                    )
                });
            let mut endpoint_available = false;
            for key in endpoint_keys {
                let status_snapshot =
                    provider_key_status_snapshot_payload(key, &provider.provider_type);
                let scheduling = provider_key_scheduling_state_payload(
                    key,
                    &provider.provider_type,
                    &status_snapshot,
                    now_unix_secs,
                );
                let available = key.is_active
                    && !is_provider_key_circuit_open(key, &endpoint.api_format)
                    && !scheduling.blocking;
                if !available {
                    continue;
                }
                endpoint_available = true;
                if let Some(score) = provider_key_health_score(key, &endpoint.api_format) {
                    scores.push(score);
                }
            }
            if endpoint_available {
                active_endpoints += 1;
                provider_available = true;
            }
        }

        if provider_available {
            active_providers += 1;
            provider_names.insert(provider.name.clone());
        }
    }

    summary_from_parts(active_providers, active_endpoints, scores, provider_names)
}

fn summary_from_parts(
    active_providers: usize,
    active_endpoints: usize,
    scores: Vec<f64>,
    provider_names: BTreeSet<String>,
) -> PublicModelHealthSummary {
    let score = (!scores.is_empty()).then(|| scores.iter().sum::<f64>() / scores.len() as f64);
    let status = if active_providers == 0 {
        "unavailable"
    } else if score.is_some_and(|value| value < 0.5) {
        "degraded"
    } else {
        "healthy"
    };
    PublicModelHealthSummary {
        status,
        score,
        active_providers,
        active_endpoints,
        providers: provider_names,
    }
}

fn summarize_routing_health(routing: Option<&serde_json::Value>) -> PublicModelHealthSummary {
    let mut active_providers = 0usize;
    let mut active_endpoints = 0usize;
    let mut scores = Vec::new();
    let mut provider_names = BTreeSet::new();

    let providers = routing
        .and_then(|payload| payload.get("providers"))
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    for provider in providers {
        if provider["is_active"] != json!(true) || provider["model_is_active"] != json!(true) {
            continue;
        }

        let mut provider_available = false;
        let endpoints = provider
            .get("endpoints")
            .and_then(serde_json::Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        for endpoint in endpoints
            .iter()
            .filter(|endpoint| endpoint["is_active"] == json!(true))
        {
            let keys = endpoint
                .get("keys")
                .and_then(serde_json::Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default();
            let endpoint_available = keys.iter().any(key_is_available);
            if endpoint_available {
                active_endpoints += 1;
                provider_available = true;
            }
            scores.extend(keys.iter().filter_map(active_key_health_score));
        }

        if provider_available {
            active_providers += 1;
            if let Some(name) = provider.get("name").and_then(serde_json::Value::as_str) {
                provider_names.insert(name.to_string());
            }
        }
    }

    summary_from_parts(active_providers, active_endpoints, scores, provider_names)
}

fn key_is_available(key: &serde_json::Value) -> bool {
    key["is_active"] == json!(true)
        && key["circuit_breaker_open"] != json!(true)
        && key["scheduling_blocking"] != json!(true)
}

fn active_key_health_score(key: &serde_json::Value) -> Option<f64> {
    (key["is_active"] == json!(true)
        && key["circuit_breaker_open"] != json!(true)
        && key["scheduling_blocking"] != json!(true))
    .then(|| key["health_score"].as_f64())
    .flatten()
}

fn key_allowed_models_match_global_model(
    raw_allowed_models: Option<&serde_json::Value>,
    model_names: &[String],
    global_model_mappings: &[String],
) -> bool {
    let allowed_models = json_string_list(raw_allowed_models);
    if raw_allowed_models.is_none() || allowed_models.is_empty() {
        return true;
    }

    allowed_models
        .iter()
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .any(|allowed_model| {
            model_names
                .iter()
                .any(|model_name| model_name.eq_ignore_ascii_case(allowed_model))
                || global_model_mappings
                    .iter()
                    .any(|pattern| matches_model_mapping(pattern, allowed_model))
        })
}

fn provider_model_mapping_names(raw_mappings: Option<&serde_json::Value>) -> Vec<String> {
    raw_mappings
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    item.as_str()
                        .or_else(|| item.get("name").and_then(serde_json::Value::as_str))
                })
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn key_match_model_names(
    global_model_name: &str,
    provider_model_name: &str,
    provider_model_mapping_names: &[String],
) -> Vec<String> {
    let mut names = Vec::new();
    push_unique_model_name(&mut names, global_model_name);
    push_unique_model_name(&mut names, provider_model_name);
    for mapping_name in provider_model_mapping_names {
        push_unique_model_name(&mut names, mapping_name);
    }
    names
}

fn push_unique_model_name(names: &mut Vec<String>, value: &str) {
    let value = value.trim();
    if value.is_empty()
        || names
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(value))
    {
        return;
    }
    names.push(value.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_when_no_routing_exists() {
        let summary = summarize_routing_health(None);

        assert_eq!(summary.status, "unavailable");
        assert_eq!(summary.active_providers, 0);
        assert_eq!(summary.active_endpoints, 0);
        assert!(summary.score.is_none());
        assert!(summary.providers.is_empty());
    }

    #[test]
    fn summarizes_only_available_providers_and_endpoints() {
        let routing = json!({
            "providers": [
                {
                    "name": "openai",
                    "is_active": true,
                    "model_is_active": true,
                    "endpoints": [{
                        "is_active": true,
                        "keys": [{
                            "is_active": true,
                            "circuit_breaker_open": false,
                            "scheduling_blocking": false,
                            "health_score": 0.8
                        }]
                    }]
                },
                {
                    "name": "blocked",
                    "is_active": true,
                    "model_is_active": true,
                    "endpoints": [{
                        "is_active": true,
                        "keys": [{
                            "is_active": true,
                            "circuit_breaker_open": false,
                            "scheduling_blocking": true,
                            "health_score": 0.2
                        }]
                    }]
                }
            ]
        });

        let summary = summarize_routing_health(Some(&routing));

        assert_eq!(summary.status, "healthy");
        assert_eq!(summary.active_providers, 1);
        assert_eq!(summary.active_endpoints, 1);
        assert_eq!(summary.score, Some(0.8));
        assert_eq!(
            summary.providers.into_iter().collect::<Vec<_>>(),
            ["openai"]
        );
    }

    #[test]
    fn unavailable_when_all_keys_are_blocked_from_scheduling() {
        let routing = json!({
            "providers": [{
                "name": "blocked",
                "is_active": true,
                "model_is_active": true,
                "endpoints": [{
                    "is_active": true,
                    "keys": [{
                        "is_active": true,
                        "circuit_breaker_open": false,
                        "scheduling_blocking": true,
                        "health_score": 0.9
                    }]
                }]
            }]
        });

        let summary = summarize_routing_health(Some(&routing));

        assert_eq!(summary.status, "unavailable");
        assert_eq!(summary.active_providers, 0);
        assert_eq!(summary.active_endpoints, 0);
        assert!(summary.score.is_none());
    }
}
