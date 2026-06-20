use crate::content_moderation::{
    extract_moderation_input, run_content_moderation_precheck, ContentModerationLevel,
    ContentModerationPrecheckOutcome, ContentModerationSettings, ContentModerationTargetRef,
    CONTENT_MODERATION_CONFIG_KEY,
};
use aether_contracts::{ExecutionPlan, RequestBody};
use aether_crypto::{encrypt_python_fernet_plaintext, DEVELOPMENT_ENCRYPTION_KEY};
use aether_data::repository::usage::InMemoryUsageReadRepository;
use aether_data_contracts::repository::usage::{StoredRequestUsageAudit, UsageReadRepository};
use axum::response::IntoResponse;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::data::GatewayDataState;

use super::{any, json, start_server, Arc, Json, Mutex, Request, Router, StatusCode};

#[test]
fn moderation_settings_default_to_disabled() {
    let settings = ContentModerationSettings::from_system_config_value(None)
        .expect("missing config should parse");

    assert!(!settings.enabled);
    assert_eq!(settings.level, ContentModerationLevel::AllUserInputs);
    assert!(settings.api_keys.is_empty());
    assert_eq!(settings.model, "omni-moderation-latest");
    assert_eq!(settings.timeout_ms, 3_000);
    assert_eq!(settings.evidence_retention_days, 30);
}

#[test]
fn moderation_settings_read_multiple_api_keys() {
    let settings = ContentModerationSettings::from_system_config_value(Some(&json!({
        "enabled": true,
        "api_keys": [" sk-moderation-a ", "", "sk-moderation-b", "sk-moderation-a"]
    })))
    .expect("config should parse");

    assert_eq!(
        settings.api_keys,
        vec!["sk-moderation-a".to_string(), "sk-moderation-b".to_string()]
    );
}

#[test]
fn moderation_settings_match_provider_endpoint_or_key_scope() {
    let settings = ContentModerationSettings::from_system_config_value(Some(&json!({
        "enabled": true,
        "targets": [
            {"kind": "provider", "id": "provider-1"},
            {"kind": "upstream_service", "id": "endpoint-1"},
            {"kind": "upstream_account", "id": "key-1"}
        ]
    })))
    .expect("config should parse");

    assert!(settings.matches_target(&ContentModerationTargetRef {
        provider_id: "provider-1",
        upstream_service_id: "other-endpoint",
        upstream_account_id: "other-key",
    }));
    assert!(settings.matches_target(&ContentModerationTargetRef {
        provider_id: "other-provider",
        upstream_service_id: "endpoint-1",
        upstream_account_id: "other-key",
    }));
    assert!(settings.matches_target(&ContentModerationTargetRef {
        provider_id: "other-provider",
        upstream_service_id: "other-endpoint",
        upstream_account_id: "key-1",
    }));
    assert!(!settings.matches_target(&ContentModerationTargetRef {
        provider_id: "other-provider",
        upstream_service_id: "other-endpoint",
        upstream_account_id: "other-key",
    }));
}

#[test]
fn moderation_input_extracts_latest_user_input_without_system_content() {
    let body = json!({
        "model": "gpt-5",
        "messages": [
            {"role": "system", "content": "system secret"},
            {"role": "user", "content": "first user message"},
            {"role": "assistant", "content": "assistant output"},
            {"role": "user", "content": [
                {"type": "text", "text": "latest"},
                {"type": "input_text", "text": "user text"}
            ]}
        ]
    });

    let input = extract_moderation_input(&body, ContentModerationLevel::LatestUserInput)
        .expect("input should be extracted");

    assert_eq!(input, "latest\nuser text");
    assert!(!input.contains("system secret"));
    assert!(!input.contains("assistant output"));
}

#[test]
fn moderation_input_extracts_all_user_inputs_for_responses_body() {
    let body = json!({
        "model": "gpt-5",
        "input": [
            {"role": "user", "content": [{"type": "input_text", "text": "first"}]},
            {"role": "assistant", "content": [{"type": "output_text", "text": "skip"}]},
            {"role": "user", "content": "second"}
        ]
    });

    let input = extract_moderation_input(&body, ContentModerationLevel::AllUserInputs)
        .expect("input should be extracted");

    assert_eq!(input, "first\nsecond");
}

#[test]
fn moderation_input_can_review_full_request() {
    let body = json!({
        "model": "gpt-5",
        "messages": [{"role": "system", "content": "system content"}]
    });

    let input = extract_moderation_input(&body, ContentModerationLevel::FullRequest)
        .expect("input should be extracted");

    assert!(input.contains("system content"));
    assert!(input.contains("messages"));
}

#[tokio::test]
async fn moderation_precheck_retries_next_api_key_after_key_failure() {
    let seen_authorization = Arc::new(Mutex::new(Vec::new()));
    let moderation_api = Router::new().route(
        "/moderations",
        any({
            let seen_authorization = Arc::clone(&seen_authorization);
            move |request: Request| {
                let seen_authorization = Arc::clone(&seen_authorization);
                let authorization = request
                    .headers()
                    .get(http::header::AUTHORIZATION)
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default()
                    .to_string();
                async move {
                    seen_authorization
                        .lock()
                        .expect("authorization log should lock")
                        .push(authorization.clone());
                    if authorization == "Bearer sk-moderation-bad" {
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(json!({"error": "bad moderation key"})),
                        )
                            .into_response();
                    }
                    Json(json!({
                        "model": "omni-moderation-latest",
                        "results": [{"flagged": false}],
                        "usage": {"input_tokens": 100, "output_tokens": 0}
                    }))
                    .into_response()
                }
            }
        }),
    );
    let (moderation_url, handle) = start_server(moderation_api).await;
    let state = state_with_moderation_config_keys(
        moderation_url.as_str(),
        ["sk-moderation-bad", "sk-moderation-good"],
        false,
    );

    let outcome =
        run_content_moderation_precheck(&state, &test_plan("req-moderation-key-retry"), None)
            .await
            .expect("precheck should retry another key");

    assert!(matches!(
        outcome,
        ContentModerationPrecheckOutcome::Continue { .. }
    ));
    assert_eq!(
        seen_authorization
            .lock()
            .expect("authorization log should lock")
            .as_slice(),
        ["Bearer sk-moderation-bad", "Bearer sk-moderation-good"]
    );

    handle.abort();
}

#[tokio::test]
async fn moderation_precheck_uses_encrypted_api_keys_from_system_config() {
    let seen_authorization = Arc::new(Mutex::new(Vec::new()));
    let moderation_api = Router::new().route(
        "/moderations",
        any({
            let seen_authorization = Arc::clone(&seen_authorization);
            move |request: Request| {
                let seen_authorization = Arc::clone(&seen_authorization);
                let authorization = request
                    .headers()
                    .get(http::header::AUTHORIZATION)
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default()
                    .to_string();
                async move {
                    seen_authorization
                        .lock()
                        .expect("authorization log should lock")
                        .push(authorization);
                    Json(json!({
                        "model": "omni-moderation-latest",
                        "results": [{"flagged": false}],
                        "usage": {"input_tokens": 100, "output_tokens": 0}
                    }))
                    .into_response()
                }
            }
        }),
    );
    let (moderation_url, handle) = start_server(moderation_api).await;
    let encrypted_api_key =
        encrypt_python_fernet_plaintext(DEVELOPMENT_ENCRYPTION_KEY, "sk-moderation-encrypted")
            .expect("api key should encrypt");
    let config = json!({
        "enabled": true,
        "targets": [{"kind": "provider", "id": "provider-openai-usage-local-1"}],
        "level": "all_user_inputs",
        "api_keys_encrypted": [encrypted_api_key],
        "base_url": moderation_url,
        "model": "omni-moderation-latest",
        "timeout_ms": 1_000,
        "input_price_per_1m": 1.0,
        "output_price_per_1m": 0.0,
        "evidence_retention_days": 30
    });
    let state = crate::AppState::new()
        .expect("gateway should build")
        .with_data_state_for_tests(
            GatewayDataState::disabled().with_system_config_values_for_tests([(
                CONTENT_MODERATION_CONFIG_KEY.to_string(),
                config,
            )]),
        );

    let outcome =
        run_content_moderation_precheck(&state, &test_plan("req-moderation-encrypted-key"), None)
            .await
            .expect("precheck should use encrypted key");

    assert!(matches!(
        outcome,
        ContentModerationPrecheckOutcome::Continue { .. }
    ));
    assert_eq!(
        seen_authorization
            .lock()
            .expect("authorization log should lock")
            .as_slice(),
        ["Bearer sk-moderation-encrypted"]
    );

    handle.abort();
}

#[tokio::test]
async fn moderation_precheck_limits_api_key_retry_attempts() {
    let calls = Arc::new(AtomicUsize::new(0));
    let moderation_api = Router::new().route(
        "/moderations",
        any({
            let calls = Arc::clone(&calls);
            move |_request: Request| {
                let calls = Arc::clone(&calls);
                async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"error": "bad moderation key"})),
                    )
                        .into_response()
                }
            }
        }),
    );
    let (moderation_url, handle) = start_server(moderation_api).await;
    let state = state_with_moderation_config_keys(
        moderation_url.as_str(),
        [
            "sk-moderation-bad-1",
            "sk-moderation-bad-2",
            "sk-moderation-bad-3",
            "sk-moderation-bad-4",
        ],
        false,
    );

    let outcome =
        run_content_moderation_precheck(&state, &test_plan("req-moderation-retry-limit"), None)
            .await
            .expect("precheck should fail open after limited retries");

    assert!(matches!(
        outcome,
        ContentModerationPrecheckOutcome::Continue { .. }
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 3);

    handle.abort();
}

#[tokio::test]
async fn moderation_precheck_passes_and_attaches_charge_metadata() {
    let calls = Arc::new(AtomicUsize::new(0));
    let moderation_api = Router::new().route(
        "/moderations",
        any({
            let calls = Arc::clone(&calls);
            move |_request: Request| {
                let calls = Arc::clone(&calls);
                async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Json(json!({
                        "model": "omni-moderation-latest",
                        "results": [{
                            "flagged": false,
                            "categories": {"violence": false},
                            "category_scores": {"violence": 0.01}
                        }],
                        "usage": {"input_tokens": 100, "output_tokens": 0}
                    }))
                }
            }
        }),
    );
    let (moderation_url, handle) = start_server(moderation_api).await;
    let state = state_with_moderation_config(moderation_url.as_str(), false);

    let outcome = run_content_moderation_precheck(&state, &test_plan("req-moderation-pass"), None)
        .await
        .expect("precheck should succeed");

    let ContentModerationPrecheckOutcome::Continue {
        report_context: Some(report_context),
    } = outcome
    else {
        panic!("precheck should continue with moderation metadata");
    };
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        report_context["content_moderation"]["result"],
        json!("passed")
    );
    assert_eq!(report_context["content_moderation_cost_usd"], json!(0.0001));

    handle.abort();
}

#[tokio::test]
async fn moderation_precheck_blocks_and_records_charge() {
    let usage_repository = Arc::new(InMemoryUsageReadRepository::default());
    let moderation_api = Router::new().route(
        "/moderations",
        any(|_request: Request| async move {
            Json(json!({
                "model": "omni-moderation-latest",
                "results": [{
                    "flagged": true,
                    "categories": {"violence": true},
                    "category_scores": {"violence": 0.91}
                }],
                "usage": {"input_tokens": 200, "output_tokens": 0}
            }))
        }),
    );
    let (moderation_url, handle) = start_server(moderation_api).await;
    let state = state_with_moderation_config(moderation_url.as_str(), true)
        .with_data_state_for_tests(
            GatewayDataState::with_usage_repository_for_tests(Arc::clone(&usage_repository))
                .with_system_config_values_for_tests([(
                    CONTENT_MODERATION_CONFIG_KEY.to_string(),
                    moderation_config(moderation_url.as_str()),
                )]),
        );

    let outcome = run_content_moderation_precheck(&state, &test_plan("req-moderation-block"), None)
        .await
        .expect("precheck should succeed");

    let ContentModerationPrecheckOutcome::Blocked { response, .. } = outcome else {
        panic!("precheck should block flagged content");
    };
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let stored = wait_for_usage(&usage_repository, "req-moderation-block").await;
    assert_eq!(stored.status, "failed");
    assert_eq!(stored.billing_status, "pending");
    assert_eq!(stored.total_cost_usd, 0.0002);
    assert_eq!(
        stored
            .request_metadata
            .as_ref()
            .and_then(|value| value.get("content_moderation"))
            .and_then(|value| value.get("result")),
        Some(&json!("flagged"))
    );

    handle.abort();
}

#[tokio::test]
async fn moderation_precheck_failure_allows_without_charge() {
    let usage_repository = Arc::new(InMemoryUsageReadRepository::default());
    let moderation_api = Router::new().route(
        "/moderations",
        any(|_request: Request| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "moderation unavailable"})),
            )
        }),
    );
    let (moderation_url, handle) = start_server(moderation_api).await;
    let state = state_with_moderation_config(moderation_url.as_str(), true)
        .with_data_state_for_tests(
            GatewayDataState::with_usage_repository_for_tests(Arc::clone(&usage_repository))
                .with_system_config_values_for_tests([(
                    CONTENT_MODERATION_CONFIG_KEY.to_string(),
                    moderation_config(moderation_url.as_str()),
                )]),
        );

    let outcome =
        run_content_moderation_precheck(&state, &test_plan("req-moderation-fail-open"), None)
            .await
            .expect("precheck should fail open");

    assert!(matches!(
        outcome,
        ContentModerationPrecheckOutcome::Continue { .. }
    ));
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(usage_repository
        .find_by_request_id("req-moderation-fail-open")
        .await
        .expect("usage lookup should succeed")
        .is_none());

    handle.abort();
}

#[tokio::test]
async fn moderation_precheck_reuses_cached_result_for_same_request() {
    let calls = Arc::new(AtomicUsize::new(0));
    let moderation_api = Router::new().route(
        "/moderations",
        any({
            let calls = Arc::clone(&calls);
            move |_request: Request| {
                let calls = Arc::clone(&calls);
                async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Json(json!({
                        "results": [{"flagged": false}],
                        "usage": {"input_tokens": 100, "output_tokens": 0}
                    }))
                }
            }
        }),
    );
    let (moderation_url, handle) = start_server(moderation_api).await;
    let state = state_with_moderation_config(moderation_url.as_str(), false);
    let plan = test_plan("req-moderation-cache");

    run_content_moderation_precheck(&state, &plan, None)
        .await
        .expect("first precheck should succeed");
    run_content_moderation_precheck(&state, &plan, None)
        .await
        .expect("second precheck should succeed");

    assert_eq!(calls.load(Ordering::SeqCst), 1);

    handle.abort();
}

#[tokio::test]
async fn moderation_precheck_reuses_cache_across_protected_targets_for_same_request() {
    let calls = Arc::new(AtomicUsize::new(0));
    let moderation_api = Router::new().route(
        "/moderations",
        any({
            let calls = Arc::clone(&calls);
            move |_request: Request| {
                let calls = Arc::clone(&calls);
                async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Json(json!({
                        "results": [{"flagged": false}],
                        "usage": {"input_tokens": 100, "output_tokens": 0}
                    }))
                }
            }
        }),
    );
    let (moderation_url, handle) = start_server(moderation_api).await;
    let state = state_with_moderation_config(moderation_url.as_str(), false);
    let plan = test_plan("req-moderation-cache-target");
    let mut other_target_plan = test_plan("req-moderation-cache-target");
    other_target_plan.endpoint_id = "endpoint-openai-usage-local-2".to_string();
    other_target_plan.key_id = "key-openai-usage-local-2".to_string();

    run_content_moderation_precheck(&state, &plan, None)
        .await
        .expect("first precheck should succeed");
    let second_outcome = run_content_moderation_precheck(&state, &other_target_plan, None)
        .await
        .expect("different target precheck should reuse same request review");
    run_content_moderation_precheck(&state, &plan, None)
        .await
        .expect("same target precheck should reuse cache");

    let ContentModerationPrecheckOutcome::Continue {
        report_context: Some(report_context),
    } = second_outcome
    else {
        panic!("second target should continue with cached moderation metadata");
    };
    assert_eq!(
        report_context["content_moderation"]["protected_target"]["upstream_service_id"],
        json!("endpoint-openai-usage-local-1")
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    handle.abort();
}

fn state_with_moderation_config(base_url: &str, usage_enabled: bool) -> crate::AppState {
    state_with_moderation_config_keys(base_url, ["sk-moderation-test"], usage_enabled)
}

fn state_with_moderation_config_keys(
    base_url: &str,
    api_keys: impl IntoIterator<Item = impl AsRef<str>>,
    usage_enabled: bool,
) -> crate::AppState {
    let state = crate::AppState::new()
        .expect("gateway should build")
        .with_data_state_for_tests(
            GatewayDataState::disabled().with_system_config_values_for_tests([(
                CONTENT_MODERATION_CONFIG_KEY.to_string(),
                moderation_config_with_keys(base_url, api_keys),
            )]),
        );
    if usage_enabled {
        state.with_usage_runtime_for_tests(crate::UsageRuntimeConfig {
            enabled: true,
            ..crate::UsageRuntimeConfig::default()
        })
    } else {
        state
    }
}

fn moderation_config(base_url: &str) -> serde_json::Value {
    moderation_config_with_keys(base_url, ["sk-moderation-test"])
}

fn moderation_config_with_keys(
    base_url: &str,
    api_keys: impl IntoIterator<Item = impl AsRef<str>>,
) -> serde_json::Value {
    let api_keys = api_keys
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();
    json!({
        "enabled": true,
        "targets": [{"kind": "provider", "id": "provider-openai-usage-local-1"}],
        "level": "all_user_inputs",
        "api_keys": api_keys,
        "base_url": base_url,
        "model": "omni-moderation-latest",
        "timeout_ms": 1_000,
        "input_price_per_1m": 1.0,
        "output_price_per_1m": 0.0,
        "evidence_retention_days": 30
    })
}

fn test_plan(request_id: &str) -> ExecutionPlan {
    ExecutionPlan {
        request_id: request_id.to_string(),
        candidate_id: Some(format!("{request_id}-candidate")),
        provider_name: Some("OpenAI".to_string()),
        provider_id: "provider-openai-usage-local-1".to_string(),
        endpoint_id: "endpoint-openai-usage-local-1".to_string(),
        key_id: "key-openai-usage-local-1".to_string(),
        method: "POST".to_string(),
        url: "https://example.com/v1/chat/completions".to_string(),
        headers: Default::default(),
        content_type: Some("application/json".to_string()),
        content_encoding: None,
        body: RequestBody::from_json(json!({
            "model": "gpt-5",
            "messages": [{"role": "user", "content": "hello moderation"}]
        })),
        stream: false,
        client_api_format: "openai:chat".to_string(),
        provider_api_format: "openai:chat".to_string(),
        model_name: Some("gpt-5".to_string()),
        proxy: None,
        transport_profile: None,
        timeouts: None,
    }
}

async fn wait_for_usage(
    repository: &InMemoryUsageReadRepository,
    request_id: &str,
) -> StoredRequestUsageAudit {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        if let Some(stored) = repository
            .find_by_request_id(request_id)
            .await
            .expect("usage lookup should succeed")
        {
            return stored;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "usage should be recorded"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}
