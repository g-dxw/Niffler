use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aether_billing::quantize_cost;
use aether_contracts::ExecutionPlan;
use aether_data::repository::content_moderation_evidence::InsertContentModerationEvidenceRecord;
use aether_usage_runtime::{
    build_usage_event_data_seed, UsageEvent, UsageEventData, UsageEventType,
};
use axum::body::Body;
use axum::http::{header::CONTENT_TYPE, Response, StatusCode};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use tracing::warn;

use crate::handlers::shared::decrypt_catalog_secret_with_fallbacks;
use crate::{AppState, GatewayError};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "omni-moderation-latest";
const DEFAULT_TIMEOUT_MS: u64 = 3_000;
const DEFAULT_EVIDENCE_RETENTION_DAYS: u64 = 30;
const MAX_CONTENT_MODERATION_API_KEY_ATTEMPTS: usize = 3;
pub(crate) const CONTENT_MODERATION_CONFIG_KEY: &str = "content_moderation_account_protection";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContentModerationLevel {
    Off,
    LatestUserInput,
    AllUserInputs,
    FullRequest,
}

impl ContentModerationLevel {
    fn from_value(value: Option<&Value>) -> Self {
        let Some(value) = value.and_then(Value::as_str).map(str::trim) else {
            return Self::AllUserInputs;
        };
        match value {
            value
                if value.eq_ignore_ascii_case("off") || value.eq_ignore_ascii_case("disabled") =>
            {
                Self::Off
            }
            value
                if value.eq_ignore_ascii_case("latest_user_input")
                    || value.eq_ignore_ascii_case("latest") =>
            {
                Self::LatestUserInput
            }
            value
                if value.eq_ignore_ascii_case("all_user_inputs")
                    || value.eq_ignore_ascii_case("user_inputs") =>
            {
                Self::AllUserInputs
            }
            value
                if value.eq_ignore_ascii_case("full_request")
                    || value.eq_ignore_ascii_case("full") =>
            {
                Self::FullRequest
            }
            _ => Self::AllUserInputs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ContentModerationTarget {
    Provider(String),
    UpstreamService(String),
    UpstreamAccount(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContentModerationTargetRef<'a> {
    pub(crate) provider_id: &'a str,
    pub(crate) upstream_service_id: &'a str,
    pub(crate) upstream_account_id: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContentModerationSettings {
    pub(crate) enabled: bool,
    targets: Vec<ContentModerationTarget>,
    pub(crate) level: ContentModerationLevel,
    pub(crate) api_keys: Vec<String>,
    pub(crate) base_url: String,
    pub(crate) model: String,
    pub(crate) timeout_ms: u64,
    pub(crate) evidence_retention_days: u64,
    pub(crate) input_price_per_1m: f64,
    pub(crate) output_price_per_1m: f64,
}

#[derive(Debug, Clone)]
pub(crate) enum ContentModerationCacheEntry {
    Passed(ContentModerationUsageRecord),
    Blocked(ContentModerationUsageRecord),
    FailedOpen,
}

#[derive(Debug, Clone)]
pub(crate) struct ContentModerationUsageRecord {
    pub(crate) model: String,
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) cost_usd: f64,
    pub(crate) actual_cost_usd: f64,
    pub(crate) input_sha256: String,
    pub(crate) flagged: bool,
    pub(crate) categories: Value,
    pub(crate) category_scores: Value,
    pub(crate) evidence_id: Option<String>,
    pub(crate) evidence_retention_days: u64,
    pub(crate) evidence_expires_at_unix_secs: u64,
    pub(crate) protected_provider_id: String,
    pub(crate) protected_upstream_service_id: String,
    pub(crate) protected_upstream_account_id: String,
}

pub(crate) enum ContentModerationPrecheckOutcome {
    Continue {
        report_context: Option<Value>,
    },
    Blocked {
        response: Response<Body>,
        report_context: Option<Value>,
    },
}

impl ContentModerationSettings {
    pub(crate) fn from_system_config_value(value: Option<&Value>) -> Result<Self, String> {
        let Some(object) = value.and_then(Value::as_object) else {
            return Ok(Self::default());
        };

        let level = ContentModerationLevel::from_value(object.get("level"));
        let targets = object
            .get("targets")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(parse_target).collect::<Vec<_>>())
            .unwrap_or_default();
        let timeout_ms = object
            .get("timeout_ms")
            .or_else(|| object.get("timeout"))
            .and_then(json_u64)
            .unwrap_or(DEFAULT_TIMEOUT_MS)
            .clamp(500, 60_000);
        let evidence_retention_days = object
            .get("evidence_retention_days")
            .and_then(json_u64)
            .unwrap_or(DEFAULT_EVIDENCE_RETENTION_DAYS)
            .clamp(1, 365);

        Ok(Self {
            enabled: object
                .get("enabled")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            targets,
            level,
            api_keys: object
                .get("api_keys")
                .and_then(Value::as_array)
                .map(|items| {
                    normalize_moderation_api_keys(
                        items.iter().filter_map(Value::as_str).map(str::to_owned),
                    )
                })
                .unwrap_or_default(),
            base_url: object
                .get("base_url")
                .and_then(Value::as_str)
                .and_then(trimmed_string)
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            model: object
                .get("model")
                .and_then(Value::as_str)
                .and_then(trimmed_string)
                .unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            timeout_ms,
            evidence_retention_days,
            input_price_per_1m: object
                .get("input_price_per_1m")
                .and_then(Value::as_f64)
                .filter(|value| value.is_finite() && *value >= 0.0)
                .unwrap_or(0.0),
            output_price_per_1m: object
                .get("output_price_per_1m")
                .and_then(Value::as_f64)
                .filter(|value| value.is_finite() && *value >= 0.0)
                .unwrap_or(0.0),
        })
    }

    pub(crate) fn matches_target(&self, target: &ContentModerationTargetRef<'_>) -> bool {
        if !self.enabled || self.level == ContentModerationLevel::Off {
            return false;
        }
        if self.targets.is_empty() {
            return true;
        }
        self.targets.iter().any(|configured| match configured {
            ContentModerationTarget::Provider(id) => id == target.provider_id,
            ContentModerationTarget::UpstreamService(id) => id == target.upstream_service_id,
            ContentModerationTarget::UpstreamAccount(id) => id == target.upstream_account_id,
        })
    }
}

impl Default for ContentModerationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            targets: Vec::new(),
            level: ContentModerationLevel::AllUserInputs,
            api_keys: Vec::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            timeout_ms: DEFAULT_TIMEOUT_MS,
            evidence_retention_days: DEFAULT_EVIDENCE_RETENTION_DAYS,
            input_price_per_1m: 0.0,
            output_price_per_1m: 0.0,
        }
    }
}

pub(crate) async fn run_content_moderation_precheck(
    state: &AppState,
    plan: &ExecutionPlan,
    report_context: Option<Value>,
) -> Result<ContentModerationPrecheckOutcome, GatewayError> {
    let Some(settings) = read_content_moderation_settings(state).await else {
        return Ok(ContentModerationPrecheckOutcome::Continue { report_context });
    };
    let target = ContentModerationTargetRef {
        provider_id: plan.provider_id.as_str(),
        upstream_service_id: plan.endpoint_id.as_str(),
        upstream_account_id: plan.key_id.as_str(),
    };
    if !settings.matches_target(&target) {
        return Ok(ContentModerationPrecheckOutcome::Continue { report_context });
    }

    let Some(body) = plan.body.json_body.as_ref() else {
        warn!(
            event_name = "content_moderation_skipped_non_json_body",
            log_type = "event",
            request_id = %plan.request_id,
            provider_id = %plan.provider_id,
            endpoint_id = %plan.endpoint_id,
            key_id = %plan.key_id,
            "content moderation skipped because execution plan has no inline JSON body"
        );
        return Ok(ContentModerationPrecheckOutcome::Continue { report_context });
    };
    let Some(input) = extract_moderation_input(body, settings.level) else {
        return Ok(ContentModerationPrecheckOutcome::Continue { report_context });
    };

    let input_sha256 = sha256_hex(input.as_bytes());
    let cache_key = moderation_cache_key(plan.request_id.as_str(), &settings, &input_sha256);
    if let Some(cached) = cached_moderation_result(state, cache_key.as_str()) {
        return Ok(match cached {
            ContentModerationCacheEntry::Passed(record) => {
                ContentModerationPrecheckOutcome::Continue {
                    report_context: attach_content_moderation_report_context(
                        report_context,
                        &record,
                        "passed",
                    ),
                }
            }
            ContentModerationCacheEntry::Blocked(record) => {
                ContentModerationPrecheckOutcome::Blocked {
                    response: build_content_moderation_blocked_response(&record),
                    report_context: attach_content_moderation_report_context(
                        report_context,
                        &record,
                        "flagged",
                    ),
                }
            }
            ContentModerationCacheEntry::FailedOpen => {
                ContentModerationPrecheckOutcome::Continue { report_context }
            }
        });
    }

    let review = match execute_openai_moderation_request(state, &settings, input.as_str()).await {
        Ok(review) => review,
        Err(err) => {
            remember_moderation_result(state, cache_key, ContentModerationCacheEntry::FailedOpen);
            warn!(
                event_name = "content_moderation_failed_open",
                log_type = "event",
                request_id = %plan.request_id,
                provider_id = %plan.provider_id,
                endpoint_id = %plan.endpoint_id,
                key_id = %plan.key_id,
                error = %err,
                "content moderation failed; request will continue without moderation charge"
            );
            return Ok(ContentModerationPrecheckOutcome::Continue { report_context });
        }
    };

    let mut record = build_content_moderation_usage_record(
        &settings,
        &target,
        input.as_str(),
        input_sha256,
        review,
    );
    record.evidence_id = store_content_moderation_evidence(
        state,
        plan,
        report_context.as_ref(),
        input.as_str(),
        &record,
    )
    .await;
    if record.flagged {
        remember_moderation_result(
            state,
            cache_key,
            ContentModerationCacheEntry::Blocked(record.clone()),
        );
        record_content_moderation_blocked_usage(state, plan, report_context.as_ref(), &record)
            .await;
        return Ok(ContentModerationPrecheckOutcome::Blocked {
            response: build_content_moderation_blocked_response(&record),
            report_context: attach_content_moderation_report_context(
                report_context,
                &record,
                "flagged",
            ),
        });
    }

    remember_moderation_result(
        state,
        cache_key,
        ContentModerationCacheEntry::Passed(record.clone()),
    );
    Ok(ContentModerationPrecheckOutcome::Continue {
        report_context: attach_content_moderation_report_context(report_context, &record, "passed"),
    })
}

pub(crate) fn extract_moderation_input(
    body: &Value,
    level: ContentModerationLevel,
) -> Option<String> {
    match level {
        ContentModerationLevel::Off => None,
        ContentModerationLevel::FullRequest => serde_json::to_string(body)
            .ok()
            .and_then(|value| trimmed_string(value.as_str())),
        ContentModerationLevel::LatestUserInput => collect_user_inputs(body).pop(),
        ContentModerationLevel::AllUserInputs => join_non_empty(collect_user_inputs(body)),
    }
}

async fn read_content_moderation_settings(state: &AppState) -> Option<ContentModerationSettings> {
    let value = match state
        .read_system_config_json_value(CONTENT_MODERATION_CONFIG_KEY)
        .await
    {
        Ok(value) => value,
        Err(err) => {
            warn!(
                event_name = "content_moderation_config_read_failed",
                log_type = "event",
                error = ?err,
                "content moderation config read failed; request will continue"
            );
            return None;
        }
    };
    let runtime_value = decrypt_content_moderation_settings_value(state, value.as_ref());
    match ContentModerationSettings::from_system_config_value(runtime_value.as_ref()) {
        Ok(settings) if settings.enabled && settings.level != ContentModerationLevel::Off => {
            Some(settings)
        }
        Ok(_) => None,
        Err(err) => {
            warn!(
                event_name = "content_moderation_config_parse_failed",
                log_type = "event",
                error = %err,
                "content moderation config parse failed; request will continue"
            );
            None
        }
    }
}

fn decrypt_content_moderation_settings_value(
    state: &AppState,
    value: Option<&Value>,
) -> Option<Value> {
    let Some(Value::Object(object)) = value else {
        return value.cloned();
    };
    let mut object = object.clone();
    if let Some(encrypted_keys) = object.get("api_keys_encrypted").and_then(Value::as_array) {
        let api_keys = encrypted_keys
            .iter()
            .filter_map(Value::as_str)
            .filter_map(|ciphertext| {
                let trimmed = ciphertext.trim();
                if trimmed.is_empty() {
                    return None;
                }
                decrypt_catalog_secret_with_fallbacks(state.encryption_key(), trimmed)
            })
            .collect::<Vec<_>>();
        object.insert("api_keys".to_string(), json!(api_keys));
    }
    object.remove("api_keys_encrypted");
    Some(Value::Object(object))
}

#[derive(Debug)]
struct OpenAiModerationReview {
    model: String,
    flagged: bool,
    categories: Value,
    category_scores: Value,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
}

async fn execute_openai_moderation_request(
    state: &AppState,
    settings: &ContentModerationSettings,
    input: &str,
) -> Result<OpenAiModerationReview, String> {
    let api_keys = settings.api_keys.as_slice();
    if api_keys.is_empty() {
        return Err("content moderation api keys are empty".to_string());
    }
    let start_index = next_moderation_api_key_start_index(state, api_keys.len());
    let mut last_err = None;
    for offset in 0..api_keys.len().min(MAX_CONTENT_MODERATION_API_KEY_ATTEMPTS) {
        let api_key = api_keys[(start_index + offset) % api_keys.len()].as_str();
        match execute_openai_moderation_request_with_key(state, settings, api_key, input).await {
            Ok(review) => return Ok(review),
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.unwrap_or_else(|| "content moderation api keys are empty".to_string()))
}

async fn execute_openai_moderation_request_with_key(
    state: &AppState,
    settings: &ContentModerationSettings,
    api_key: &str,
    input: &str,
) -> Result<OpenAiModerationReview, String> {
    let url = format!("{}/moderations", settings.base_url.trim_end_matches('/'));
    let response = state
        .client
        .post(url)
        .bearer_auth(api_key)
        .timeout(Duration::from_millis(settings.timeout_ms))
        .json(&json!({
            "model": settings.model,
            "input": input,
        }))
        .send()
        .await
        .map_err(|err| err.to_string())?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("moderation api returned {status}"));
    }
    let value = response
        .json::<Value>()
        .await
        .map_err(|err| format!("moderation api returned invalid json: {err}"))?;
    parse_openai_moderation_response(&value, settings.model.as_str())
}

fn next_moderation_api_key_start_index(state: &AppState, key_count: usize) -> usize {
    if key_count == 0 {
        return 0;
    }
    state
        .content_moderation_key_cursor
        .fetch_add(1, Ordering::Relaxed)
        .try_into()
        .unwrap_or(0usize)
        % key_count
}

fn parse_openai_moderation_response(
    value: &Value,
    fallback_model: &str,
) -> Result<OpenAiModerationReview, String> {
    let results = value
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| "moderation api response missing results".to_string())?;
    let flagged = results.iter().any(|item| {
        item.get("flagged")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    });
    let mut categories = Map::new();
    let mut category_scores = Map::new();
    for result in results {
        if let Some(object) = result.get("categories").and_then(Value::as_object) {
            merge_json_object(&mut categories, object);
        }
        if let Some(object) = result.get("category_scores").and_then(Value::as_object) {
            merge_json_object(&mut category_scores, object);
        }
    }
    let usage = value.get("usage").and_then(Value::as_object);
    Ok(OpenAiModerationReview {
        model: value
            .get("model")
            .and_then(Value::as_str)
            .and_then(trimmed_string)
            .unwrap_or_else(|| fallback_model.to_string()),
        flagged,
        categories: Value::Object(categories),
        category_scores: Value::Object(category_scores),
        input_tokens: usage.and_then(|usage| {
            usage
                .get("input_tokens")
                .or_else(|| usage.get("prompt_tokens"))
                .and_then(json_u64)
        }),
        output_tokens: usage.and_then(|usage| {
            usage
                .get("output_tokens")
                .or_else(|| usage.get("completion_tokens"))
                .and_then(json_u64)
        }),
    })
}

fn build_content_moderation_usage_record(
    settings: &ContentModerationSettings,
    target: &ContentModerationTargetRef<'_>,
    input: &str,
    input_sha256: String,
    review: OpenAiModerationReview,
) -> ContentModerationUsageRecord {
    let input_tokens = review
        .input_tokens
        .unwrap_or_else(|| estimate_moderation_input_tokens(input));
    let output_tokens = review.output_tokens.unwrap_or(0);
    let cost_usd = quantize_cost(
        (input_tokens as f64 * settings.input_price_per_1m
            + output_tokens as f64 * settings.output_price_per_1m)
            / 1_000_000.0,
    );
    let now = current_unix_secs();
    ContentModerationUsageRecord {
        model: review.model,
        input_tokens,
        output_tokens,
        cost_usd,
        actual_cost_usd: cost_usd,
        input_sha256,
        flagged: review.flagged,
        categories: review.categories,
        category_scores: review.category_scores,
        evidence_id: None,
        evidence_retention_days: settings.evidence_retention_days,
        evidence_expires_at_unix_secs: now.saturating_add(
            settings
                .evidence_retention_days
                .saturating_mul(24 * 60 * 60),
        ),
        protected_provider_id: target.provider_id.to_string(),
        protected_upstream_service_id: target.upstream_service_id.to_string(),
        protected_upstream_account_id: target.upstream_account_id.to_string(),
    }
}

async fn store_content_moderation_evidence(
    state: &AppState,
    plan: &ExecutionPlan,
    report_context: Option<&Value>,
    input: &str,
    record: &ContentModerationUsageRecord,
) -> Option<String> {
    if !state.data.has_content_moderation_evidence_writer() {
        return None;
    }

    let evidence_id = format!("cme-{}", uuid::Uuid::new_v4());
    let evidence = InsertContentModerationEvidenceRecord {
        id: evidence_id,
        request_id: plan.request_id.clone(),
        user_id: report_context_string(report_context, "user_id"),
        api_key_id: report_context_string(report_context, "api_key_id"),
        provider_id: trimmed_string(plan.provider_id.as_str()),
        upstream_service_id: trimmed_string(plan.endpoint_id.as_str()),
        upstream_account_id: trimmed_string(plan.key_id.as_str()),
        moderation_model: record.model.clone(),
        input_sha256: record.input_sha256.clone(),
        input_text: Some(input.to_string()),
        categories: record.categories.clone(),
        category_scores: record.category_scores.clone(),
        flagged: record.flagged,
        created_at_unix_secs: current_unix_secs(),
        expires_at_unix_secs: record.evidence_expires_at_unix_secs,
    };

    match state
        .data
        .insert_content_moderation_evidence(evidence)
        .await
    {
        Ok(Some(stored)) => Some(stored.id),
        Ok(None) => None,
        Err(err) => {
            warn!(
                event_name = "content_moderation_evidence_write_failed",
                log_type = "event",
                request_id = %plan.request_id,
                provider_id = %plan.provider_id,
                endpoint_id = %plan.endpoint_id,
                key_id = %plan.key_id,
                error = %err,
                "content moderation evidence write failed; request will continue"
            );
            None
        }
    }
}

fn attach_content_moderation_report_context(
    report_context: Option<Value>,
    record: &ContentModerationUsageRecord,
    result: &str,
) -> Option<Value> {
    let mut object = match report_context {
        Some(Value::Object(object)) => object,
        Some(other) => Map::from_iter([("original_report_context".to_string(), other)]),
        None => Map::new(),
    };
    object.insert(
        "content_moderation".to_string(),
        content_moderation_metadata(record, result),
    );
    object.insert(
        "content_moderation_cost_usd".to_string(),
        Value::from(record.cost_usd),
    );
    object.insert(
        "content_moderation_actual_cost_usd".to_string(),
        Value::from(record.actual_cost_usd),
    );
    Some(Value::Object(object))
}

async fn record_content_moderation_blocked_usage(
    state: &AppState,
    plan: &ExecutionPlan,
    report_context: Option<&Value>,
    record: &ContentModerationUsageRecord,
) {
    if !state.usage_runtime.is_enabled() {
        return;
    }
    let mut data = build_usage_event_data_seed(plan, report_context);
    apply_content_moderation_blocked_usage_data(&mut data, record);
    state.usage_runtime.submit_terminal_event(
        state.data.as_ref(),
        UsageEvent::new(UsageEventType::Failed, plan.request_id.clone(), data),
    );
}

fn apply_content_moderation_blocked_usage_data(
    data: &mut UsageEventData,
    record: &ContentModerationUsageRecord,
) {
    data.provider_name = "Niffler 内容审查".to_string();
    data.model = record.model.clone();
    data.input_tokens = Some(record.input_tokens);
    data.output_tokens = Some(record.output_tokens);
    data.total_tokens = Some(record.input_tokens.saturating_add(record.output_tokens));
    data.total_cost_usd = Some(record.cost_usd);
    data.actual_total_cost_usd = Some(record.actual_cost_usd);
    data.status_code = Some(StatusCode::FORBIDDEN.as_u16());
    data.error_message = Some("请求内容未通过内容审查".to_string());
    data.error_category = Some("client_error".to_string());
    data.response_headers = Some(json!({"content-type": "application/json"}));
    data.response_body = Some(content_moderation_error_body());
    data.client_response_headers = Some(json!({"content-type": "application/json"}));
    data.client_response_body = Some(content_moderation_error_body());
    data.billing_status_override = Some("pending".to_string());
    let mut metadata = match data.request_metadata.take() {
        Some(Value::Object(object)) => object,
        Some(other) => Map::from_iter([("original_request_metadata".to_string(), other)]),
        None => Map::new(),
    };
    metadata.insert(
        "source".to_string(),
        Value::String("content_moderation".to_string()),
    );
    metadata.insert(
        "platform_rejection_reason".to_string(),
        Value::String("content_moderation_flagged".to_string()),
    );
    metadata.insert(
        "platform_reason".to_string(),
        Value::String("content_moderation_flagged".to_string()),
    );
    metadata.insert(
        "content_moderation".to_string(),
        content_moderation_metadata(record, "flagged"),
    );
    metadata.insert(
        "content_moderation_cost_usd".to_string(),
        Value::from(record.cost_usd),
    );
    metadata.insert(
        "content_moderation_actual_cost_usd".to_string(),
        Value::from(record.actual_cost_usd),
    );
    data.request_metadata = Some(Value::Object(metadata));
}

fn content_moderation_metadata(record: &ContentModerationUsageRecord, result: &str) -> Value {
    json!({
        "result": result,
        "flagged": record.flagged,
        "model": record.model,
        "input_tokens": record.input_tokens,
        "output_tokens": record.output_tokens,
        "cost_usd": record.cost_usd,
        "actual_cost_usd": record.actual_cost_usd,
        "input_sha256": record.input_sha256,
        "evidence_id": record.evidence_id,
        "categories": record.categories,
        "category_scores": record.category_scores,
        "evidence_retention_days": record.evidence_retention_days,
        "evidence_expires_at_unix_secs": record.evidence_expires_at_unix_secs,
        "protected_target": {
            "provider_id": record.protected_provider_id,
            "upstream_service_id": record.protected_upstream_service_id,
            "upstream_account_id": record.protected_upstream_account_id,
        }
    })
}

fn build_content_moderation_blocked_response(
    record: &ContentModerationUsageRecord,
) -> Response<Body> {
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header(CONTENT_TYPE, "application/json")
        .header("x-aether-content-moderation", "flagged")
        .body(Body::from(content_moderation_error_body().to_string()))
        .expect("content moderation response should build")
        .with_header_cost(record.cost_usd)
}

trait ContentModerationResponseExt {
    fn with_header_cost(self, cost_usd: f64) -> Self;
}

impl ContentModerationResponseExt for Response<Body> {
    fn with_header_cost(mut self, cost_usd: f64) -> Self {
        if let Ok(value) = axum::http::HeaderValue::from_str(&cost_usd.to_string()) {
            self.headers_mut()
                .insert("x-aether-content-moderation-cost-usd", value);
        }
        self
    }
}

fn content_moderation_error_body() -> Value {
    json!({
        "error": {
            "type": "content_moderation_flagged",
            "message": "请求内容未通过内容审查，已拒绝转发到上游服务。",
            "code": StatusCode::FORBIDDEN.as_u16()
        }
    })
}

fn cached_moderation_result(
    state: &AppState,
    cache_key: &str,
) -> Option<ContentModerationCacheEntry> {
    state
        .content_moderation_cache
        .lock()
        .expect("content moderation cache should lock")
        .get(cache_key)
        .cloned()
}

fn remember_moderation_result(
    state: &AppState,
    cache_key: String,
    result: ContentModerationCacheEntry,
) {
    prune_moderation_cache(state);
    state
        .content_moderation_cache
        .lock()
        .expect("content moderation cache should lock")
        .insert(cache_key, result);
}

fn prune_moderation_cache(state: &AppState) {
    const MAX_CACHE_ENTRIES: usize = 1024;
    let mut cache = state
        .content_moderation_cache
        .lock()
        .expect("content moderation cache should lock");
    if cache.len() < MAX_CACHE_ENTRIES {
        return;
    }
    cache.clear();
}

fn moderation_cache_key(
    request_id: &str,
    settings: &ContentModerationSettings,
    input_sha256: &str,
) -> String {
    format!(
        "{}:{}:{}:{}",
        request_id,
        settings.level.cache_key_part(),
        settings.model,
        input_sha256
    )
}

impl ContentModerationLevel {
    fn cache_key_part(self) -> &'static str {
        match self {
            ContentModerationLevel::Off => "off",
            ContentModerationLevel::LatestUserInput => "latest_user_input",
            ContentModerationLevel::AllUserInputs => "all_user_inputs",
            ContentModerationLevel::FullRequest => "full_request",
        }
    }
}

fn merge_json_object(target: &mut Map<String, Value>, source: &Map<String, Value>) {
    for (key, value) in source {
        target.insert(key.clone(), value.clone());
    }
}

fn estimate_moderation_input_tokens(input: &str) -> u64 {
    let char_count = input.chars().count() as u64;
    char_count.saturating_add(3).saturating_div(4).max(1)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn report_context_string(report_context: Option<&Value>, key: &str) -> Option<String> {
    report_context
        .and_then(|value| value.get(key))
        .and_then(Value::as_str)
        .and_then(trimmed_string)
}

fn parse_target(value: &Value) -> Option<ContentModerationTarget> {
    let object = value.as_object()?;
    let kind = object.get("kind").and_then(Value::as_str)?.trim();
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .and_then(trimmed_string)?;
    if kind.eq_ignore_ascii_case("provider") {
        Some(ContentModerationTarget::Provider(id))
    } else if kind.eq_ignore_ascii_case("upstream_service")
        || kind.eq_ignore_ascii_case("service")
        || kind.eq_ignore_ascii_case("endpoint")
    {
        Some(ContentModerationTarget::UpstreamService(id))
    } else if kind.eq_ignore_ascii_case("upstream_account")
        || kind.eq_ignore_ascii_case("account")
        || kind.eq_ignore_ascii_case("key")
    {
        Some(ContentModerationTarget::UpstreamAccount(id))
    } else {
        None
    }
}

fn collect_user_inputs(body: &Value) -> Vec<String> {
    let mut values = Vec::new();
    collect_role_items(body.get("messages"), &mut values);
    collect_role_items(body.get("input"), &mut values);
    collect_role_items(body.get("contents"), &mut values);
    if let Some(text) = body
        .get("prompt")
        .and_then(Value::as_str)
        .and_then(trimmed_string)
    {
        values.push(text);
    }
    values
}

fn collect_role_items(value: Option<&Value>, out: &mut Vec<String>) {
    match value {
        Some(Value::String(text)) => {
            if let Some(text) = trimmed_string(text) {
                out.push(text);
            }
        }
        Some(Value::Array(items)) => {
            for item in items {
                if item
                    .get("role")
                    .and_then(Value::as_str)
                    .is_some_and(|role| role.eq_ignore_ascii_case("user"))
                {
                    if let Some(text) = extract_text_from_content(
                        item.get("content")
                            .or_else(|| item.get("parts"))
                            .unwrap_or(item),
                    ) {
                        out.push(text);
                    }
                }
            }
        }
        Some(Value::Object(object)) => {
            if object
                .get("role")
                .and_then(Value::as_str)
                .is_some_and(|role| role.eq_ignore_ascii_case("user"))
            {
                if let Some(text) = extract_text_from_content(value.unwrap()) {
                    out.push(text);
                }
            }
        }
        _ => {}
    }
}

fn extract_text_from_content(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => trimmed_string(text),
        Value::Array(items) => join_non_empty(items.iter().filter_map(extract_text_from_content)),
        Value::Object(object) => {
            if let Some(text) = object
                .get("text")
                .and_then(Value::as_str)
                .and_then(trimmed_string)
            {
                return Some(text);
            }
            if let Some(text) = object
                .get("input_text")
                .and_then(Value::as_str)
                .and_then(trimmed_string)
            {
                return Some(text);
            }
            object
                .get("content")
                .or_else(|| object.get("parts"))
                .and_then(extract_text_from_content)
        }
        _ => None,
    }
}

fn join_non_empty(values: impl IntoIterator<Item = String>) -> Option<String> {
    let values = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| values.join("\n"))
}

fn normalize_moderation_api_keys(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut keys = Vec::new();
    for value in values {
        let Some(key) = trimmed_string(&value) else {
            continue;
        };
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    keys
}

fn trimmed_string(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn json_u64(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_i64().and_then(|value| u64::try_from(value).ok()))
}
