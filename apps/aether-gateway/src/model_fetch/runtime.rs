use std::collections::HashMap;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use aether_data_contracts::repository::provider_catalog::{
    StoredProviderCatalogEndpoint, StoredProviderCatalogKey, StoredProviderCatalogProvider,
};
use aether_model_fetch::{
    apply_model_filters, codex_model_fetch_client_version_override, fetch_models_from_transports,
    fetch_models_from_transports_with_codex_client_version, json_string_list,
    merge_upstream_metadata, model_fetch_interval_minutes, model_fetch_startup_delay_seconds,
    model_fetch_startup_enabled, preset_models_for_provider, selected_models_fetch_endpoints,
    sync_provider_model_whitelist_associations, ModelFetchAssociationStore, ModelFetchRunSummary,
};
use serde_json::{json, Value};
use tracing::{debug, info, warn};

use crate::{AppState, GatewayError};

use super::codex_version::{
    candidate_catalog_preserves_current_models, client_version_check_is_due,
    client_version_next_check_at, discovered_version_is_newer, effective_version_after_probe,
    effective_version_from_state_and_memory, fetch_official_codex_stable_version,
    process_last_known_version, read_codex_client_version_state, write_codex_client_version_state,
    CodexClientVersionState, CODEX_CLIENT_VERSION_RETRY_INTERVAL_SECS,
};

pub(crate) mod state;

use self::state::ModelFetchRuntimeState;

#[derive(Debug, Clone)]
struct SelectedFetchTarget {
    provider: StoredProviderCatalogProvider,
    key: StoredProviderCatalogKey,
    endpoints: Vec<StoredProviderCatalogEndpoint>,
}

pub(crate) fn spawn_model_fetch_worker(state: AppState) -> Option<tokio::task::JoinHandle<()>> {
    if !state.has_provider_catalog_data_reader() || !state.has_provider_catalog_data_writer() {
        return None;
    }

    Some(tokio::spawn(async move {
        if model_fetch_startup_enabled() {
            let startup_delay = model_fetch_startup_delay_seconds();
            if startup_delay > 0 {
                tokio::time::sleep(Duration::from_secs(startup_delay)).await;
            }
            if let Err(err) = run_model_fetch_cycle(&state, "startup").await {
                warn!(error = ?err, "gateway model fetch startup failed");
            }
        } else {
            info!("gateway model fetch startup disabled");
        }

        let mut model_fetch_interval = tokio::time::interval(Duration::from_secs(
            model_fetch_interval_minutes().saturating_mul(60),
        ));
        model_fetch_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        model_fetch_interval.tick().await;
        let mut codex_version_interval = tokio::time::interval(Duration::from_secs(
            CODEX_CLIENT_VERSION_RETRY_INTERVAL_SECS,
        ));
        codex_version_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        codex_version_interval.tick().await;
        loop {
            tokio::select! {
                _ = model_fetch_interval.tick() => {
                    if let Err(err) = run_model_fetch_cycle(&state, "tick").await {
                        warn!(error = ?err, "gateway model fetch tick failed");
                    }
                }
                _ = codex_version_interval.tick() => {
                    refresh_codex_client_version_if_due(&state).await;
                }
            }
        }
    }))
}

pub(crate) async fn perform_model_fetch_once(
    state: &AppState,
) -> Result<ModelFetchRunSummary, GatewayError> {
    perform_model_fetch_once_with_state(state).await
}

pub(crate) async fn perform_model_fetch_for_key(
    state: &AppState,
    provider_id: &str,
    key_id: &str,
) -> Result<ModelFetchRunSummary, GatewayError> {
    perform_model_fetch_for_key_with_state(state, provider_id, key_id).await
}

async fn perform_model_fetch_once_with_state<S>(
    state: &S,
) -> Result<ModelFetchRunSummary, GatewayError>
where
    S: ModelFetchRuntimeState + ?Sized,
{
    let targets = collect_fetch_targets(state, None, None).await?;
    execute_fetch_targets(state, targets).await
}

async fn perform_model_fetch_for_key_with_state<S>(
    state: &S,
    provider_id: &str,
    key_id: &str,
) -> Result<ModelFetchRunSummary, GatewayError>
where
    S: ModelFetchRuntimeState + ?Sized,
{
    let targets = collect_fetch_targets(state, Some(provider_id), Some(key_id)).await?;
    execute_fetch_targets(state, targets).await
}

async fn collect_fetch_targets<S>(
    state: &S,
    provider_id_filter: Option<&str>,
    key_id_filter: Option<&str>,
) -> Result<Vec<SelectedFetchTarget>, GatewayError>
where
    S: ModelFetchRuntimeState + ?Sized,
{
    collect_fetch_targets_with_auto_fetch_requirement(
        state,
        provider_id_filter,
        key_id_filter,
        true,
    )
    .await
}

async fn collect_fetch_targets_with_auto_fetch_requirement<S>(
    state: &S,
    provider_id_filter: Option<&str>,
    key_id_filter: Option<&str>,
    require_auto_fetch_models: bool,
) -> Result<Vec<SelectedFetchTarget>, GatewayError>
where
    S: ModelFetchRuntimeState + ?Sized,
{
    if !state.has_provider_catalog_data_reader() || !state.has_provider_catalog_data_writer() {
        return Ok(Vec::new());
    }

    let providers = state
        .list_provider_catalog_providers(true)
        .await?
        .into_iter()
        .filter(|provider| provider_id_filter.is_none_or(|provider_id| provider.id == provider_id))
        .collect::<Vec<_>>();
    if providers.is_empty() {
        return Ok(Vec::new());
    }

    let provider_ids = providers
        .iter()
        .map(|provider| provider.id.clone())
        .collect::<Vec<_>>();
    let mut endpoints_by_provider = HashMap::<String, Vec<StoredProviderCatalogEndpoint>>::new();
    for endpoint in state
        .list_provider_catalog_endpoints_by_provider_ids(&provider_ids)
        .await?
    {
        endpoints_by_provider
            .entry(endpoint.provider_id.clone())
            .or_default()
            .push(endpoint);
    }
    let mut keys_by_provider = HashMap::<String, Vec<StoredProviderCatalogKey>>::new();
    for key in <S as ModelFetchAssociationStore>::list_provider_catalog_keys_by_provider_ids(
        state,
        &provider_ids,
    )
    .await
    .map_err(GatewayError::Internal)?
    {
        keys_by_provider
            .entry(key.provider_id.clone())
            .or_default()
            .push(key);
    }

    let mut targets = Vec::new();
    for provider in providers {
        let endpoints = endpoints_by_provider
            .remove(&provider.id)
            .unwrap_or_default();
        let keys = keys_by_provider.remove(&provider.id).unwrap_or_default();
        for key in keys {
            if key_id_filter.is_some_and(|key_id| key.id != key_id) {
                continue;
            }
            if !key.is_active || (require_auto_fetch_models && !key.auto_fetch_models) {
                continue;
            }
            let selected_endpoints = selected_models_fetch_endpoints(&endpoints, &key);
            targets.push(SelectedFetchTarget {
                provider: provider.clone(),
                key,
                endpoints: selected_endpoints,
            });
        }
    }
    Ok(targets)
}

async fn execute_fetch_targets<S>(
    state: &S,
    targets: Vec<SelectedFetchTarget>,
) -> Result<ModelFetchRunSummary, GatewayError>
where
    S: ModelFetchRuntimeState + ?Sized,
{
    let mut summary = ModelFetchRunSummary {
        attempted: targets.len(),
        succeeded: 0,
        failed: 0,
        skipped: 0,
    };
    for target in targets {
        match fetch_and_persist_key_models(state, &target).await? {
            KeyFetchDisposition::Succeeded => summary.succeeded += 1,
            KeyFetchDisposition::Failed => summary.failed += 1,
            KeyFetchDisposition::Skipped => summary.skipped += 1,
        }
    }
    Ok(summary)
}

async fn run_model_fetch_cycle(state: &AppState, phase: &'static str) -> Result<(), GatewayError> {
    refresh_codex_client_version_if_due(state).await;
    let summary = perform_model_fetch_once_with_state(state).await?;
    if summary.attempted == 0 {
        debug!(phase, "gateway model fetch found no eligible keys");
        return Ok(());
    }

    info!(
        phase,
        attempted = summary.attempted,
        succeeded = summary.succeeded,
        failed = summary.failed,
        skipped = summary.skipped,
        "gateway model fetch cycle completed"
    );
    Ok(())
}

async fn refresh_codex_client_version_if_due(state: &AppState) {
    if let Some(version) = codex_model_fetch_client_version_override() {
        debug!(
            version,
            "Codex model fetch client version pinned by environment"
        );
        return;
    }

    let now_unix_secs = now_unix_secs();
    let process_last_known = process_last_known_version(state);
    let existing = match read_codex_client_version_state(state).await {
        Ok(existing) => existing,
        Err(err) => {
            warn!(error = %err, "Codex client version state read failed");
            None
        }
    };
    if !client_version_check_is_due(existing.as_ref(), now_unix_secs) {
        return;
    }

    let current_version =
        effective_version_from_state_and_memory(existing.as_ref(), process_last_known.as_deref());
    let discovered_version = match fetch_official_codex_stable_version(state).await {
        Ok(version) => version,
        Err(err) => {
            warn!(error = %err, version = %current_version, "Codex client version discovery failed");
            persist_codex_client_version_check(state, current_version, now_unix_secs, false).await;
            return;
        }
    };

    if !discovered_version_is_newer(&current_version, &discovered_version) {
        persist_codex_client_version_check(state, current_version, now_unix_secs, true).await;
        return;
    }

    let accepted = match probe_codex_client_version(state, &current_version, &discovered_version)
        .await
    {
        Ok(accepted) => accepted,
        Err(err) => {
            warn!(error = ?err, version = %discovered_version, "Codex client version validation failed");
            false
        }
    };
    let effective_version =
        effective_version_after_probe(&current_version, &discovered_version, accepted);
    let persisted =
        persist_codex_client_version_check(state, effective_version, now_unix_secs, accepted).await;
    if accepted && persisted {
        info!(
            previous_version = %current_version,
            version = %discovered_version,
            "Codex model fetch client version updated"
        );
    }
}

async fn persist_codex_client_version_check(
    state: &AppState,
    version: String,
    checked_at_unix_secs: u64,
    check_succeeded: bool,
) -> bool {
    if let Err(err) = write_codex_client_version_state(
        state,
        &CodexClientVersionState {
            version,
            checked_at_unix_secs,
            next_check_at_unix_secs: client_version_next_check_at(
                checked_at_unix_secs,
                check_succeeded,
            ),
        },
    )
    .await
    {
        warn!(error = %err, "Codex client version state write failed");
        return false;
    }
    true
}

async fn probe_codex_client_version<S>(
    state: &S,
    current_version: &str,
    candidate_version: &str,
) -> Result<bool, GatewayError>
where
    S: ModelFetchRuntimeState + ?Sized,
{
    let targets =
        collect_fetch_targets_with_auto_fetch_requirement(state, None, None, false).await?;
    let mut attempted = false;
    for target in targets.into_iter().filter(|target| {
        target
            .provider
            .provider_type
            .trim()
            .eq_ignore_ascii_case("codex")
    }) {
        let mut transports = Vec::new();
        for endpoint in &target.endpoints {
            if let Some(transport) = state
                .read_provider_transport_snapshot(&target.provider.id, &endpoint.id, &target.key.id)
                .await?
            {
                if !codex_endpoint_pins_client_version(&transport.endpoint.base_url) {
                    transports.push(transport);
                }
            }
        }
        if transports.is_empty() {
            continue;
        }
        attempted = true;
        let current = match fetch_models_from_transports_with_codex_client_version(
            state,
            &transports,
            Some(current_version),
        )
        .await
        {
            Ok(outcome) if outcome.has_success && !outcome.fetched_model_ids.is_empty() => outcome,
            Ok(_) => continue,
            Err(err) => {
                debug!(
                    provider_id = %target.provider.id,
                    key_id = %target.key.id,
                    version = %current_version,
                    error = %err,
                    "Current Codex client version probe failed for key"
                );
                continue;
            }
        };
        let candidate = match fetch_models_from_transports_with_codex_client_version(
            state,
            &transports,
            Some(candidate_version),
        )
        .await
        {
            Ok(outcome) if outcome.has_success && !outcome.fetched_model_ids.is_empty() => outcome,
            Ok(_) => continue,
            Err(err) => {
                debug!(
                    provider_id = %target.provider.id,
                    key_id = %target.key.id,
                    version = %candidate_version,
                    error = %err,
                    "Candidate Codex client version probe failed for key"
                );
                continue;
            }
        };
        if candidate_catalog_preserves_current_models(
            &current.fetched_model_ids,
            &candidate.fetched_model_ids,
        ) {
            return Ok(true);
        }
        warn!(
            provider_id = %target.provider.id,
            key_id = %target.key.id,
            current_version = %current_version,
            candidate_version = %candidate_version,
            current_model_count = current.fetched_model_ids.len(),
            candidate_model_count = candidate.fetched_model_ids.len(),
            "Candidate Codex client version returned a reduced model catalog"
        );
    }
    if !attempted {
        debug!(version = %candidate_version, "Codex client version probe found no eligible key");
    }
    Ok(false)
}

fn codex_endpoint_pins_client_version(base_url: &str) -> bool {
    base_url
        .split_once('?')
        .map(|(_, query)| {
            query.split('&').any(|part| {
                part.split_once('=')
                    .map(|(key, _)| key.trim().eq_ignore_ascii_case("client_version"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyFetchDisposition {
    Succeeded,
    Failed,
    Skipped,
}

async fn fetch_and_persist_key_models(
    state: &(impl ModelFetchRuntimeState + ?Sized),
    target: &SelectedFetchTarget,
) -> Result<KeyFetchDisposition, GatewayError> {
    let now_unix_secs = now_unix_secs();
    if target.endpoints.is_empty() {
        if let Some(models) = preset_models_for_provider(&target.provider.provider_type) {
            let fetched_model_ids = models
                .iter()
                .filter_map(|model| model.get("id"))
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            let filtered_models = apply_model_filters(
                &fetched_model_ids,
                json_string_list(target.key.locked_models.as_ref()),
                json_string_list(target.key.model_include_patterns.as_ref()),
                json_string_list(target.key.model_exclude_patterns.as_ref()),
            );
            persist_key_fetch_success(state, &target.key, now_unix_secs, &filtered_models, None)
                .await?;
            state
                .write_upstream_models_cache(&target.provider.id, &target.key.id, &models)
                .await;
            sync_provider_model_whitelist_associations(
                state,
                &target.provider.id,
                &filtered_models,
            )
            .await
            .map_err(GatewayError::Internal)?;
            return Ok(KeyFetchDisposition::Succeeded);
        }
        persist_key_fetch_failure(
            state,
            &target.key,
            now_unix_secs,
            "No supported endpoint for Rust models fetch".to_string(),
        )
        .await?;
        return Ok(KeyFetchDisposition::Skipped);
    }

    let mut transports = Vec::new();
    for endpoint in &target.endpoints {
        match state
            .read_provider_transport_snapshot(&target.provider.id, &endpoint.id, &target.key.id)
            .await?
        {
            Some(transport) => transports.push(transport),
            None => {
                warn!(
                    provider_id = %target.provider.id,
                    endpoint_id = %endpoint.id,
                    key_id = %target.key.id,
                    "gateway model fetch transport snapshot unavailable"
                );
            }
        }
    }

    if transports.is_empty() {
        persist_key_fetch_failure(
            state,
            &target.key,
            now_unix_secs,
            "Provider transport snapshot unavailable".to_string(),
        )
        .await?;
        return Ok(KeyFetchDisposition::Skipped);
    }

    let result = match fetch_models_from_transports(state, &transports).await {
        Ok(result) => result,
        Err(err) => {
            persist_key_fetch_failure(state, &target.key, now_unix_secs, err.clone()).await?;
            warn!(
                provider_id = %target.provider.id,
                key_id = %target.key.id,
                message = %err,
                "gateway model fetch failed"
            );
            return Ok(KeyFetchDisposition::Failed);
        }
    };

    if !result.has_success {
        let error = if result.errors.is_empty() {
            "Upstream models fetch failed".to_string()
        } else {
            result.errors.join("; ")
        };
        persist_key_fetch_failure(state, &target.key, now_unix_secs, error.clone()).await?;
        warn!(
            provider_id = %target.provider.id,
            key_id = %target.key.id,
            message = %error,
            "gateway model fetch failed"
        );
        return Ok(KeyFetchDisposition::Failed);
    }

    if result.fetched_model_ids.is_empty() {
        let error = "Upstream models fetch returned an empty catalog".to_string();
        persist_key_fetch_failure(state, &target.key, now_unix_secs, error.clone()).await?;
        warn!(
            provider_id = %target.provider.id,
            key_id = %target.key.id,
            message = %error,
            "gateway model fetch failed"
        );
        return Ok(KeyFetchDisposition::Failed);
    }

    let filtered_models = apply_model_filters(
        &result.fetched_model_ids,
        json_string_list(target.key.locked_models.as_ref()),
        json_string_list(target.key.model_include_patterns.as_ref()),
        json_string_list(target.key.model_exclude_patterns.as_ref()),
    );

    persist_key_fetch_success(
        state,
        &target.key,
        now_unix_secs,
        &filtered_models,
        result.upstream_metadata.as_ref(),
    )
    .await?;
    state
        .write_upstream_models_cache(&target.provider.id, &target.key.id, &result.cached_models)
        .await;
    sync_provider_model_whitelist_associations(state, &target.provider.id, &filtered_models)
        .await
        .map_err(GatewayError::Internal)?;
    Ok(KeyFetchDisposition::Succeeded)
}

async fn persist_key_fetch_failure(
    state: &(impl ModelFetchRuntimeState + ?Sized),
    key: &StoredProviderCatalogKey,
    now_unix_secs: u64,
    error: String,
) -> Result<(), GatewayError> {
    let mut updated = key.clone();
    updated.last_models_fetch_at_unix_secs = Some(now_unix_secs);
    updated.last_models_fetch_error = Some(error);
    updated.updated_at_unix_secs = Some(now_unix_secs);
    state.update_provider_catalog_key(&updated).await?;
    Ok(())
}

async fn persist_key_fetch_success(
    state: &(impl ModelFetchRuntimeState + ?Sized),
    key: &StoredProviderCatalogKey,
    now_unix_secs: u64,
    allowed_models: &[String],
    upstream_metadata: Option<&Value>,
) -> Result<(), GatewayError> {
    let mut updated = key.clone();
    updated.allowed_models = if allowed_models.is_empty() {
        None
    } else {
        Some(json!(allowed_models))
    };
    if let Some(upstream_metadata) = upstream_metadata {
        updated.upstream_metadata = Some(merge_upstream_metadata(
            updated.upstream_metadata.as_ref(),
            upstream_metadata,
        ));
    }
    updated.last_models_fetch_at_unix_secs = Some(now_unix_secs);
    updated.last_models_fetch_error = None;
    updated.updated_at_unix_secs = Some(now_unix_secs);
    state.update_provider_catalog_key(&updated).await?;
    Ok(())
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::{
        codex_endpoint_pins_client_version, collect_fetch_targets_with_auto_fetch_requirement,
        perform_model_fetch_once_with_state, probe_codex_client_version,
        state::ModelFetchRuntimeState,
    };
    use aether_contracts::{ExecutionPlan, ExecutionResult, ProxySnapshot};
    use aether_data_contracts::repository::global_models::{
        AdminGlobalModelListQuery, AdminProviderModelListQuery, StoredAdminGlobalModelPage,
        StoredAdminProviderModel, UpsertAdminProviderModelRecord,
    };
    use aether_data_contracts::repository::provider_catalog::{
        StoredProviderCatalogEndpoint, StoredProviderCatalogKey, StoredProviderCatalogProvider,
    };
    use aether_model_fetch::{
        build_models_fetch_execution_plan, ModelFetchAssociationStore, ModelFetchTransportRuntime,
    };
    use async_trait::async_trait;
    use serde_json::{json, Value};
    use std::collections::{HashMap, VecDeque};
    use std::sync::{Arc, Mutex};

    use crate::provider_transport::LocalResolvedOAuthRequestAuth;
    use crate::GatewayError;
    use aether_provider_transport::snapshot::{
        GatewayProviderTransportEndpoint, GatewayProviderTransportKey,
        GatewayProviderTransportProvider, GatewayProviderTransportSnapshot,
    };

    #[test]
    fn codex_version_probe_skips_endpoints_with_explicit_version() {
        assert!(codex_endpoint_pins_client_version(
            "https://chatgpt.com/backend-api/codex?client_version=0.143.0"
        ));
        assert!(codex_endpoint_pins_client_version(
            "https://chatgpt.com/backend-api/codex?foo=1&CLIENT_VERSION=0.143.0"
        ));
        assert!(!codex_endpoint_pins_client_version(
            "https://chatgpt.com/backend-api/codex"
        ));
    }

    #[derive(Clone, Default)]
    struct TestState {
        providers: Arc<Vec<StoredProviderCatalogProvider>>,
        endpoints: Arc<Vec<StoredProviderCatalogEndpoint>>,
        keys: Arc<Mutex<Vec<StoredProviderCatalogKey>>>,
        transports: Arc<HashMap<(String, String, String), GatewayProviderTransportSnapshot>>,
        execution_results: Arc<Mutex<VecDeque<ExecutionResult>>>,
        cached_models: Arc<Mutex<HashMap<(String, String), Vec<Value>>>>,
    }

    impl TestState {
        fn new(
            providers: Vec<StoredProviderCatalogProvider>,
            endpoints: Vec<StoredProviderCatalogEndpoint>,
            keys: Vec<StoredProviderCatalogKey>,
            transports: HashMap<(String, String, String), GatewayProviderTransportSnapshot>,
            execution_results: Vec<ExecutionResult>,
        ) -> Self {
            Self {
                providers: Arc::new(providers),
                endpoints: Arc::new(endpoints),
                keys: Arc::new(Mutex::new(keys)),
                transports: Arc::new(transports),
                execution_results: Arc::new(Mutex::new(VecDeque::from(execution_results))),
                cached_models: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn key(&self, key_id: &str) -> StoredProviderCatalogKey {
            self.keys
                .lock()
                .expect("keys mutex")
                .iter()
                .find(|key| key.id == key_id)
                .cloned()
                .expect("key should exist")
        }
    }

    #[async_trait]
    impl ModelFetchTransportRuntime for TestState {
        async fn resolve_local_oauth_request_auth(
            &self,
            transport: &GatewayProviderTransportSnapshot,
        ) -> Result<Option<LocalResolvedOAuthRequestAuth>, String> {
            if transport.key.auth_type.trim().eq_ignore_ascii_case("oauth") {
                return Ok(Some(LocalResolvedOAuthRequestAuth::Header {
                    name: "authorization".to_string(),
                    value: "Bearer oauth-token".to_string(),
                }));
            }
            Ok(None)
        }

        async fn resolve_model_fetch_proxy(
            &self,
            _transport: &GatewayProviderTransportSnapshot,
        ) -> Option<ProxySnapshot> {
            None
        }

        async fn execute_model_fetch_execution_plan(
            &self,
            _plan: &ExecutionPlan,
        ) -> Result<ExecutionResult, String> {
            self.execution_results
                .lock()
                .expect("execution result mutex")
                .pop_front()
                .ok_or_else(|| "missing execution result".to_string())
        }
    }

    #[async_trait]
    impl ModelFetchAssociationStore for TestState {
        type Error = String;

        fn has_global_model_reader(&self) -> bool {
            false
        }

        fn has_global_model_writer(&self) -> bool {
            false
        }

        fn model_fetch_internal_error(&self, message: String) -> Self::Error {
            message
        }

        async fn list_admin_provider_models(
            &self,
            _query: &AdminProviderModelListQuery,
        ) -> Result<Vec<StoredAdminProviderModel>, Self::Error> {
            Ok(Vec::new())
        }

        async fn list_admin_global_models(
            &self,
            _query: &AdminGlobalModelListQuery,
        ) -> Result<StoredAdminGlobalModelPage, Self::Error> {
            Ok(StoredAdminGlobalModelPage {
                items: Vec::new(),
                total: 0,
            })
        }

        async fn create_admin_provider_model(
            &self,
            _record: &UpsertAdminProviderModelRecord,
        ) -> Result<Option<StoredAdminProviderModel>, Self::Error> {
            Ok(None)
        }

        async fn list_provider_catalog_keys_by_provider_ids(
            &self,
            provider_ids: &[String],
        ) -> Result<Vec<StoredProviderCatalogKey>, Self::Error> {
            Ok(self
                .keys
                .lock()
                .expect("keys mutex")
                .iter()
                .filter(|key| {
                    provider_ids
                        .iter()
                        .any(|provider_id| provider_id == &key.provider_id)
                })
                .cloned()
                .collect())
        }
    }

    #[async_trait]
    impl ModelFetchRuntimeState for TestState {
        fn has_provider_catalog_data_reader(&self) -> bool {
            true
        }

        fn has_provider_catalog_data_writer(&self) -> bool {
            true
        }

        async fn list_provider_catalog_providers(
            &self,
            _active_only: bool,
        ) -> Result<Vec<StoredProviderCatalogProvider>, GatewayError> {
            Ok(self.providers.as_ref().clone())
        }

        async fn list_provider_catalog_endpoints_by_provider_ids(
            &self,
            provider_ids: &[String],
        ) -> Result<Vec<StoredProviderCatalogEndpoint>, GatewayError> {
            Ok(self
                .endpoints
                .iter()
                .filter(|endpoint| {
                    provider_ids
                        .iter()
                        .any(|provider_id| provider_id == &endpoint.provider_id)
                })
                .cloned()
                .collect())
        }

        async fn read_provider_transport_snapshot(
            &self,
            provider_id: &str,
            endpoint_id: &str,
            key_id: &str,
        ) -> Result<Option<GatewayProviderTransportSnapshot>, GatewayError> {
            Ok(self
                .transports
                .get(&(
                    provider_id.to_string(),
                    endpoint_id.to_string(),
                    key_id.to_string(),
                ))
                .cloned())
        }

        async fn execute_execution_runtime_sync_plan(
            &self,
            _plan: &ExecutionPlan,
        ) -> Result<ExecutionResult, GatewayError> {
            Err(GatewayError::Internal(
                "execute_execution_runtime_sync_plan should not be called".to_string(),
            ))
        }

        async fn update_provider_catalog_key(
            &self,
            key: &StoredProviderCatalogKey,
        ) -> Result<(), GatewayError> {
            let mut keys = self.keys.lock().expect("keys mutex");
            let Some(slot) = keys.iter_mut().find(|item| item.id == key.id) else {
                return Err(GatewayError::Internal("key not found".to_string()));
            };
            *slot = key.clone();
            Ok(())
        }

        async fn write_upstream_models_cache(
            &self,
            provider_id: &str,
            key_id: &str,
            cached_models: &[Value],
        ) {
            self.cached_models.lock().expect("cache mutex").insert(
                (provider_id.to_string(), key_id.to_string()),
                cached_models.to_vec(),
            );
        }
    }

    fn sample_provider(provider_id: &str, provider_type: &str) -> StoredProviderCatalogProvider {
        StoredProviderCatalogProvider::new(
            provider_id.to_string(),
            provider_id.to_string(),
            None,
            provider_type.to_string(),
        )
        .expect("provider should build")
        .with_transport_fields(true, false, false, None, None, None, None, None, None)
    }

    fn sample_endpoint(
        endpoint_id: &str,
        provider_id: &str,
        api_format: &str,
    ) -> StoredProviderCatalogEndpoint {
        StoredProviderCatalogEndpoint::new(
            endpoint_id.to_string(),
            provider_id.to_string(),
            api_format.to_string(),
            None,
            None,
            true,
        )
        .expect("endpoint should build")
        .with_transport_fields(
            "https://cloudcode-pa.googleapis.com".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("endpoint transport should build")
    }

    fn sample_key(
        key_id: &str,
        provider_id: &str,
        auth_type: &str,
        api_formats: &[&str],
    ) -> StoredProviderCatalogKey {
        let mut key = StoredProviderCatalogKey::new(
            key_id.to_string(),
            provider_id.to_string(),
            "primary".to_string(),
            auth_type.to_string(),
            None,
            true,
        )
        .expect("key should build")
        .with_transport_fields(
            Some(json!(api_formats)),
            "encrypted".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("key transport should build");
        key.auto_fetch_models = true;
        key
    }

    #[tokio::test]
    async fn codex_version_probe_can_use_active_key_without_auto_fetch() {
        let provider = sample_provider("provider-codex", "codex");
        let endpoint = sample_endpoint("endpoint-codex", "provider-codex", "openai:responses");
        let mut key = sample_key(
            "key-codex",
            "provider-codex",
            "oauth",
            &["openai:responses"],
        );
        key.auto_fetch_models = false;
        let state = TestState::new(
            vec![provider],
            vec![endpoint],
            vec![key],
            HashMap::new(),
            Vec::new(),
        );

        let probe_targets =
            collect_fetch_targets_with_auto_fetch_requirement(&state, None, None, false)
                .await
                .expect("probe targets");
        let automatic_targets =
            collect_fetch_targets_with_auto_fetch_requirement(&state, None, None, true)
                .await
                .expect("automatic targets");

        assert_eq!(probe_targets.len(), 1);
        assert!(automatic_targets.is_empty());
    }

    fn sample_transport(
        provider_type: &str,
        provider_id: &str,
        endpoint_id: &str,
        key_id: &str,
        api_format: &str,
        auth_type: &str,
        decrypted_auth_config: Option<&str>,
    ) -> GatewayProviderTransportSnapshot {
        GatewayProviderTransportSnapshot {
            provider: GatewayProviderTransportProvider {
                id: provider_id.to_string(),
                name: provider_id.to_string(),
                provider_type: provider_type.to_string(),
                website: None,
                is_active: true,
                keep_priority_on_conversion: false,
                enable_format_conversion: false,
                concurrent_limit: None,
                max_retries: None,
                proxy: None,
                request_timeout_secs: None,
                stream_first_byte_timeout_secs: None,
                config: None,
            },
            endpoint: GatewayProviderTransportEndpoint {
                id: endpoint_id.to_string(),
                provider_id: provider_id.to_string(),
                api_format: api_format.to_string(),
                api_family: None,
                endpoint_kind: None,
                is_active: true,
                base_url: "https://cloudcode-pa.googleapis.com".to_string(),
                header_rules: None,
                body_rules: None,
                max_retries: None,
                custom_path: None,
                config: None,
                format_acceptance_config: None,
                proxy: None,
            },
            key: GatewayProviderTransportKey {
                id: key_id.to_string(),
                provider_id: provider_id.to_string(),
                name: "primary".to_string(),
                auth_type: auth_type.to_string(),
                is_active: true,
                api_formats: Some(vec![api_format.to_string()]),
                auth_type_by_format: None,
                allow_auth_channel_mismatch_formats: None,
                allowed_models: None,
                capabilities: None,
                rate_multipliers: None,
                global_priority_by_format: None,
                expires_at_unix_secs: None,
                proxy: None,
                fingerprint: None,
                decrypted_api_key: "secret".to_string(),
                decrypted_auth_config: decrypted_auth_config.map(ToOwned::to_owned),
            },
        }
    }

    fn execution_result(body: Value) -> ExecutionResult {
        ExecutionResult {
            request_id: "req-1".to_string(),
            candidate_id: None,
            status_code: 200,
            headers: Default::default(),
            body: Some(aether_contracts::ResponseBody {
                json_body: Some(body),
                body_bytes_b64: None,
            }),
            telemetry: None,
            error: None,
        }
    }

    #[tokio::test]
    async fn codex_version_probe_rejects_reduced_candidate_catalog() {
        let provider = sample_provider("provider-codex", "codex");
        let mut endpoint = sample_endpoint("endpoint-codex", "provider-codex", "openai:responses");
        endpoint.base_url = "https://chatgpt.com/backend-api/codex".to_string();
        let mut key = sample_key(
            "key-codex",
            "provider-codex",
            "oauth",
            &["openai:responses"],
        );
        key.auto_fetch_models = false;
        let mut transport = sample_transport(
            "codex",
            "provider-codex",
            "endpoint-codex",
            "key-codex",
            "openai:responses",
            "oauth",
            None,
        );
        transport.endpoint.base_url = "https://chatgpt.com/backend-api/codex".to_string();
        let state = TestState::new(
            vec![provider],
            vec![endpoint],
            vec![key],
            HashMap::from([(
                (
                    "provider-codex".to_string(),
                    "endpoint-codex".to_string(),
                    "key-codex".to_string(),
                ),
                transport,
            )]),
            vec![
                execution_result(json!({
                    "models": [{"id":"gpt-5.5"}, {"id":"gpt-5.6-sol"}]
                })),
                execution_result(json!({
                    "models": [{"id":"gpt-5.5"}]
                })),
            ],
        );

        assert!(!probe_codex_client_version(&state, "0.144.1", "0.144.3")
            .await
            .expect("probe result"));
    }

    #[tokio::test]
    async fn gateway_runtime_state_supports_shared_models_fetch_plan_builder() {
        let state = TestState::default();
        let transport = sample_transport(
            "openai",
            "provider-openai",
            "endpoint-openai-chat",
            "key-openai-chat",
            "openai:chat",
            "api_key",
            None,
        );

        let plan = build_models_fetch_execution_plan(&state, &transport)
            .await
            .expect("shared models fetch plan should build");

        assert_eq!(plan.method, "GET");
        assert_eq!(plan.provider_id, "provider-openai");
        assert_eq!(plan.endpoint_id, "endpoint-openai-chat");
        assert_eq!(plan.key_id, "key-openai-chat");
        assert_eq!(plan.model_name.as_deref(), Some("models"));
    }

    #[tokio::test]
    async fn model_fetch_uses_preset_models_without_endpoint() {
        let provider = sample_provider("provider-codex", "codex");
        let key = sample_key(
            "key-codex",
            "provider-codex",
            "api_key",
            &["openai:responses"],
        );
        let state = TestState::new(vec![provider], vec![], vec![key], HashMap::new(), vec![]);

        let summary = perform_model_fetch_once_with_state(&state)
            .await
            .expect("fetch should succeed");

        assert_eq!(summary.attempted, 1);
        assert_eq!(summary.succeeded, 1);
        let updated = state.key("key-codex");
        let allowed_models = updated
            .allowed_models
            .and_then(|value| value.as_array().cloned())
            .expect("allowed_models should be set");
        assert!(allowed_models.iter().any(|model| model == "gpt-5.4"));
        assert!(state
            .cached_models
            .lock()
            .expect("cache mutex")
            .contains_key(&("provider-codex".to_string(), "key-codex".to_string())));
    }

    #[tokio::test]
    async fn model_fetch_merges_antigravity_metadata_and_preserves_reset_time() {
        let provider = sample_provider("provider-antigravity", "antigravity");
        let endpoint = sample_endpoint(
            "endpoint-antigravity",
            "provider-antigravity",
            "gemini:generate_content",
        );
        let mut key = sample_key(
            "key-antigravity",
            "provider-antigravity",
            "oauth",
            &["gemini:generate_content"],
        );
        key.upstream_metadata = Some(json!({
            "antigravity": {
                "quota_by_model": {
                    "gemini-2.5-pro": {
                        "reset_time": "2026-04-12T00:00:00Z"
                    }
                }
            }
        }));
        let transport = sample_transport(
            "antigravity",
            "provider-antigravity",
            "endpoint-antigravity",
            "key-antigravity",
            "gemini:generate_content",
            "oauth",
            Some(r#"{"project_id":"project-1","client_version":"1.2.3","session_id":"sess-1"}"#),
        );
        let state = TestState::new(
            vec![provider],
            vec![endpoint],
            vec![key],
            HashMap::from([(
                (
                    "provider-antigravity".to_string(),
                    "endpoint-antigravity".to_string(),
                    "key-antigravity".to_string(),
                ),
                transport,
            )]),
            vec![execution_result(json!({
                "models": {
                    "gemini-2.5-pro": {
                        "displayName": "Gemini 2.5 Pro",
                        "quotaInfo": {
                            "remainingFraction": 0.25
                        }
                    }
                }
            }))],
        );

        let summary = perform_model_fetch_once_with_state(&state)
            .await
            .expect("fetch should succeed");

        assert_eq!(summary.succeeded, 1);
        let updated = state.key("key-antigravity");
        assert_eq!(updated.allowed_models, Some(json!(["gemini-2.5-pro"])));
        assert_eq!(
            updated
                .upstream_metadata
                .as_ref()
                .and_then(|value| value.get("antigravity"))
                .and_then(|value| value.get("quota_by_model"))
                .and_then(|value| value.get("gemini-2.5-pro"))
                .and_then(|value| value.get("reset_time")),
            Some(&json!("2026-04-12T00:00:00Z"))
        );
    }

    #[tokio::test]
    async fn model_fetch_failure_keeps_existing_allowed_models() {
        let provider = sample_provider("provider-openai", "openai");
        let endpoint = sample_endpoint(
            "endpoint-openai-responses",
            "provider-openai",
            "openai:responses",
        );
        let mut key = sample_key(
            "key-openai-responses",
            "provider-openai",
            "api_key",
            &["openai:responses"],
        );
        key.allowed_models = Some(json!(["gpt-old"]));
        let state = TestState::new(
            vec![provider],
            vec![endpoint],
            vec![key],
            HashMap::new(),
            vec![],
        );

        let summary = perform_model_fetch_once_with_state(&state)
            .await
            .expect("fetch should finish");

        assert_eq!(summary.succeeded, 0);
        assert_eq!(summary.skipped, 1);
        let updated = state.key("key-openai-responses");
        assert_eq!(updated.allowed_models, Some(json!(["gpt-old"])));
        assert_eq!(
            updated.last_models_fetch_error.as_deref(),
            Some("Provider transport snapshot unavailable")
        );
    }

    #[tokio::test]
    async fn model_fetch_empty_catalog_keeps_existing_allowed_models() {
        let provider = sample_provider("provider-codex", "codex");
        let mut endpoint = sample_endpoint(
            "endpoint-codex-responses",
            "provider-codex",
            "openai:responses",
        );
        endpoint.base_url = "https://chatgpt.com/backend-api/codex".to_string();
        let mut key = sample_key(
            "key-codex-responses",
            "provider-codex",
            "oauth",
            &["openai:responses"],
        );
        key.allowed_models = Some(json!(["gpt-5.5"]));
        let mut transport = sample_transport(
            "codex",
            "provider-codex",
            "endpoint-codex-responses",
            "key-codex-responses",
            "openai:responses",
            "oauth",
            None,
        );
        transport.endpoint.base_url = "https://chatgpt.com/backend-api/codex".to_string();
        let state = TestState::new(
            vec![provider],
            vec![endpoint],
            vec![key],
            HashMap::from([(
                (
                    "provider-codex".to_string(),
                    "endpoint-codex-responses".to_string(),
                    "key-codex-responses".to_string(),
                ),
                transport,
            )]),
            vec![execution_result(json!({"models": []}))],
        );

        let summary = perform_model_fetch_once_with_state(&state)
            .await
            .expect("fetch should finish");

        assert_eq!(summary.succeeded, 0);
        assert_eq!(summary.failed, 1);
        let updated = state.key("key-codex-responses");
        assert_eq!(updated.allowed_models, Some(json!(["gpt-5.5"])));
        assert_eq!(
            updated.last_models_fetch_error.as_deref(),
            Some("Upstream models fetch returned an empty catalog")
        );
        assert!(!state
            .cached_models
            .lock()
            .expect("cache mutex")
            .contains_key(&(
                "provider-codex".to_string(),
                "key-codex-responses".to_string()
            )));
    }
}
