use crate::handlers::admin::niffler_legacy_projection::project_provider_key_with_niffler_account;
use crate::handlers::admin::request::AdminAppState;
use crate::provider_key_auth::provider_key_effective_api_formats;
use crate::provider_pool_demand::provider_pool_live_in_flight_by_key;
use aether_data_contracts::repository::niffler_core::NifflerUpstreamAccountListQuery;
use aether_data_contracts::repository::provider_catalog::{
    ProviderCatalogKeyListOrder, ProviderCatalogKeyListQuery,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

async fn build_admin_provider_key_items_payload(
    state: &AdminAppState<'_>,
    provider_id: &str,
    skip: usize,
    limit: usize,
) -> Option<(Vec<Value>, usize)> {
    if !state.has_provider_catalog_data_reader() {
        return None;
    }
    let provider = state
        .read_provider_catalog_providers_by_ids(&[provider_id.to_string()])
        .await
        .ok()
        .and_then(|mut providers| providers.drain(..).next())?;
    let key_page = state
        .list_provider_catalog_key_page(&ProviderCatalogKeyListQuery {
            provider_id: provider.id.clone(),
            search: None,
            is_active: None,
            offset: skip,
            limit,
            order: ProviderCatalogKeyListOrder::CreatedAt,
        })
        .await
        .ok()?;
    let endpoints = state
        .list_provider_catalog_endpoints_by_provider_ids(std::slice::from_ref(&provider.id))
        .await
        .ok()
        .unwrap_or_default();
    let now_unix_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let keys = key_page.items;
    let key_ids = keys.iter().map(|key| key.id.clone()).collect::<Vec<_>>();
    let current_concurrency_by_key =
        provider_pool_live_in_flight_by_key(state.runtime_state(), &provider.id, &key_ids).await;
    let niffler_accounts_by_id = state
        .list_niffler_upstream_accounts(&NifflerUpstreamAccountListQuery {
            upstream_service_id: Some(provider.id.clone()),
            status: None,
            search: None,
            offset: 0,
            limit: 1000,
        })
        .await
        .ok()
        .map(|page| {
            page.items
                .into_iter()
                .map(|account| (account.id.clone(), account))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut items = Vec::with_capacity(keys.len());
    for key in keys {
        let api_formats =
            provider_key_effective_api_formats(&key, &provider.provider_type, &endpoints);
        let mut payload = state.build_admin_provider_key_response(
            &key,
            &provider.provider_type,
            &api_formats,
            now_unix_secs,
        );
        if let Some(object) = payload.as_object_mut() {
            object.insert(
                "current_concurrency".to_string(),
                json!(current_concurrency_by_key
                    .get(&key.id)
                    .copied()
                    .unwrap_or(0)),
            );
        }
        if let Some(account) = niffler_accounts_by_id.get(&key.id) {
            project_provider_key_with_niffler_account(&mut payload, account, now_unix_secs);
        }
        items.push(payload);
    }
    Some((items, key_page.total))
}

pub(crate) async fn build_admin_provider_keys_payload(
    state: &AdminAppState<'_>,
    provider_id: &str,
    skip: usize,
    limit: usize,
) -> Option<Value> {
    let (items, _) =
        build_admin_provider_key_items_payload(state, provider_id, skip, limit).await?;
    Some(Value::Array(items))
}

pub(crate) async fn build_admin_provider_keys_page_payload(
    state: &AdminAppState<'_>,
    provider_id: &str,
    page: usize,
    page_size: usize,
) -> Option<Value> {
    let skip = page.saturating_sub(1).saturating_mul(page_size);
    let (items, total) =
        build_admin_provider_key_items_payload(state, provider_id, skip, page_size).await?;
    Some(json!({
        "total": total,
        "page": page,
        "page_size": page_size,
        "keys": items,
    }))
}
