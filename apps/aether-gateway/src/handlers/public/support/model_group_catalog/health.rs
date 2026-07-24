use std::collections::BTreeSet;

use serde::Serialize;
use serde_json::json;

use crate::handlers::admin::{build_admin_global_model_routing_payload, request::AdminAppState};
use crate::AppState;

#[derive(Clone, Serialize)]
pub(super) struct PublicModelHealthSummary {
    status: &'static str,
    score: Option<f64>,
    active_providers: usize,
    active_endpoints: usize,
    providers: BTreeSet<String>,
}

pub(super) async fn load_public_model_health(
    state: &AppState,
    model_id: &str,
) -> PublicModelHealthSummary {
    let admin_state = AdminAppState::new(state);
    let routing = build_admin_global_model_routing_payload(&admin_state, model_id).await;
    summarize_routing_health(routing.as_ref())
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

fn key_is_available(key: &serde_json::Value) -> bool {
    key["is_active"] == json!(true)
        && key["circuit_breaker_open"] != json!(true)
        && key["scheduling_blocking"] != json!(true)
}

fn active_key_health_score(key: &serde_json::Value) -> Option<f64> {
    (key["is_active"] == json!(true) && key["circuit_breaker_open"] != json!(true))
        .then(|| key["health_score"].as_f64())
        .flatten()
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
        assert_eq!(summary.score, Some(0.5));
        assert_eq!(
            summary.providers.into_iter().collect::<Vec<_>>(),
            ["openai"]
        );
    }
}
