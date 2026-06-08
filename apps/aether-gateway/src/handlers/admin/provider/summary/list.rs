use crate::handlers::admin::niffler_legacy_projection::project_provider_summary_with_niffler_service;
use crate::handlers::admin::request::AdminAppState;
use crate::handlers::admin::shared::unix_secs_to_rfc3339;
use aether_data_contracts::repository::niffler_core::NifflerUpstreamServiceListQuery;
use aether_data_contracts::repository::provider_catalog::StoredProviderCatalogEndpoint;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) async fn build_admin_providers_payload(
    state: &AdminAppState<'_>,
    skip: usize,
    limit: usize,
    is_active: Option<bool>,
) -> Option<serde_json::Value> {
    let state = state.as_ref();
    if !state.has_provider_catalog_data_reader() {
        return None;
    }

    let active_only = is_active.unwrap_or(false);
    let mut providers = state
        .list_provider_catalog_providers(active_only)
        .await
        .ok()
        .unwrap_or_default();
    if matches!(is_active, Some(false)) {
        providers.retain(|provider| !provider.is_active);
    }
    providers.sort_by(|left, right| {
        left.provider_priority
            .cmp(&right.provider_priority)
            .then_with(|| left.name.cmp(&right.name))
    });

    let providers = providers
        .into_iter()
        .skip(skip)
        .take(limit)
        .collect::<Vec<_>>();
    let provider_ids = providers
        .iter()
        .map(|provider| provider.id.clone())
        .collect::<Vec<_>>();
    let endpoints = if provider_ids.is_empty() {
        Vec::new()
    } else {
        state
            .list_provider_catalog_endpoints_by_provider_ids(&provider_ids)
            .await
            .ok()
            .unwrap_or_default()
    };
    let key_stats = if provider_ids.is_empty() {
        Vec::new()
    } else {
        state
            .list_provider_catalog_key_stats_by_provider_ids(&provider_ids)
            .await
            .ok()
            .unwrap_or_default()
    };
    let first_endpoint_by_provider = endpoints
        .into_iter()
        .filter(|endpoint| endpoint.is_active)
        .fold(
            BTreeMap::<String, StoredProviderCatalogEndpoint>::new(),
            |mut acc, endpoint| {
                acc.entry(endpoint.provider_id.clone()).or_insert(endpoint);
                acc
            },
        );
    let has_any_key_by_provider =
        key_stats
            .into_iter()
            .fold(BTreeSet::<String>::new(), |mut acc, stats| {
                if stats.total_keys > 0 {
                    acc.insert(stats.provider_id);
                }
                acc
            });
    let niffler_services_by_id = state
        .list_niffler_upstream_services(&NifflerUpstreamServiceListQuery {
            include_inactive: true,
            search: None,
            offset: 0,
            limit: 1000,
        })
        .await
        .ok()
        .map(|page| {
            page.items
                .into_iter()
                .map(|service| (service.id.clone(), service))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut items = Vec::with_capacity(providers.len());
    for provider in providers {
        let provider_id = provider.id.clone();
        let endpoint = first_endpoint_by_provider.get(&provider_id);
        let mut payload = json!({
            "id": provider_id.clone(),
            "name": provider.name,
            "api_format": endpoint.map(|item| item.api_format.clone()),
            "base_url": endpoint.map(|item| item.base_url.clone()),
            "api_key": has_any_key_by_provider.contains(&provider_id).then_some("***"),
            "priority": provider.provider_priority,
            "is_active": provider.is_active,
            "created_at": provider.created_at_unix_ms.and_then(unix_secs_to_rfc3339),
            "updated_at": provider.updated_at_unix_secs.and_then(unix_secs_to_rfc3339),
        });
        if let Some(service) = niffler_services_by_id.get(&provider_id) {
            project_provider_summary_with_niffler_service(&mut payload, &service);
        }
        items.push(payload);
    }

    Some(serde_json::Value::Array(items))
}
