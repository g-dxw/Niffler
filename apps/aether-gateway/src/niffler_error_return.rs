use std::collections::BTreeMap;

use aether_contracts::ExecutionPlan;
use aether_data_contracts::repository::niffler_core::{
    CreateNifflerAccountRiskEventRecord, NifflerAccountProtectionAction, NifflerErrorResponseScope,
    NifflerErrorReturnSettingListQuery, NifflerUpstreamErrorHandlingStep, NifflerUserResponseMode,
    StoredNifflerErrorReturnSetting,
};
use aether_scheduler_core::parse_request_candidate_report_context;
use axum::body::Body;
use axum::http::{Response, StatusCode};
use serde_json::{json, Value};
use tracing::warn;
use uuid::Uuid;

use crate::api::response::build_client_response_from_parts;
use crate::clock::current_unix_ms;
use crate::control::{GatewayControlDecision, GatewayLocalAuthRejection};
use crate::niffler_runtime::resolve_niffler_runtime_rollout_decision;
use crate::{AppState, GatewayError};

const ERROR_RETURN_RULE_LIMIT: usize = 100;

#[derive(Debug, Clone)]
pub(crate) struct NifflerUpstreamErrorContext {
    pub(crate) request_id: Option<String>,
    pub(crate) user_id: Option<String>,
    pub(crate) api_key_id: Option<String>,
    pub(crate) upstream_service_id: Option<String>,
    pub(crate) upstream_account_id: Option<String>,
    pub(crate) model_name: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct NifflerErrorRewrite {
    pub(crate) body_json: Value,
    pub(crate) body_bytes: Vec<u8>,
}

pub(crate) fn build_niffler_upstream_error_context(
    plan: &ExecutionPlan,
    report_context: Option<&Value>,
) -> NifflerUpstreamErrorContext {
    let metadata = parse_request_candidate_report_context(report_context);
    NifflerUpstreamErrorContext {
        request_id: metadata
            .as_ref()
            .and_then(|metadata| metadata.request_id.clone())
            .or_else(|| Some(plan.request_id.clone())),
        user_id: metadata
            .as_ref()
            .and_then(|metadata| metadata.user_id.clone()),
        api_key_id: metadata
            .as_ref()
            .and_then(|metadata| metadata.api_key_id.clone()),
        upstream_service_id: metadata
            .as_ref()
            .and_then(|metadata| metadata.provider_id.clone())
            .or_else(|| Some(plan.provider_id.clone())),
        upstream_account_id: metadata
            .as_ref()
            .and_then(|metadata| metadata.key_id.clone())
            .or_else(|| Some(plan.key_id.clone())),
        model_name: metadata
            .and_then(|metadata| metadata.mapped_model)
            .or_else(|| plan.model_name.clone()),
    }
}

pub(crate) async fn build_niffler_platform_auth_rejection_response(
    state: &AppState,
    trace_id: &str,
    control_decision: Option<&GatewayControlDecision>,
    rejection: &GatewayLocalAuthRejection,
) -> Result<Response<Body>, GatewayError> {
    let platform_error = PlatformError::from_auth_rejection(rejection);
    let message = resolve_platform_error_message(
        state,
        control_decision,
        platform_error.status_code,
        platform_error.code,
        platform_error.default_message.as_str(),
    )
    .await
    .unwrap_or(platform_error.default_message);
    let mut error = serde_json::Map::from_iter([
        (
            "type".to_string(),
            Value::String(platform_error.error_type.to_string()),
        ),
        ("message".to_string(), Value::String(message)),
    ]);
    if !platform_error.details.is_null() {
        error.insert("details".to_string(), platform_error.details);
    }
    let payload = json!({ "error": error });
    build_json_error_response(
        platform_error.status_code,
        trace_id,
        control_decision,
        payload,
        None,
    )
}

pub(crate) async fn build_niffler_platform_rate_limited_response(
    state: &AppState,
    trace_id: &str,
    control_decision: Option<&GatewayControlDecision>,
    default_message: &str,
    retry_after: u64,
    limit: u64,
    scope: &str,
) -> Result<Response<Body>, GatewayError> {
    let status_code = StatusCode::TOO_MANY_REQUESTS.as_u16();
    let message = resolve_platform_error_message(
        state,
        control_decision,
        status_code,
        "rate_limit_exceeded",
        default_message,
    )
    .await
    .unwrap_or_else(|| default_message.to_string());
    let payload = json!({
        "error": {
            "type": "rate_limit_exceeded",
            "message": message,
        }
    });
    let headers = BTreeMap::from([
        ("Retry-After".to_string(), retry_after.to_string()),
        ("X-RateLimit-Limit".to_string(), limit.to_string()),
        ("X-RateLimit-Remaining".to_string(), "0".to_string()),
        ("X-RateLimit-Scope".to_string(), scope.to_string()),
    ]);
    build_json_error_response(
        status_code,
        trace_id,
        control_decision,
        payload,
        Some(headers),
    )
}

pub(crate) async fn build_niffler_platform_http_error_response(
    state: &AppState,
    trace_id: &str,
    control_decision: Option<&GatewayControlDecision>,
    status_code: StatusCode,
    code: &'static str,
    default_message: &str,
) -> Result<Response<Body>, GatewayError> {
    let message = resolve_platform_error_message(
        state,
        control_decision,
        status_code.as_u16(),
        code,
        default_message,
    )
    .await
    .unwrap_or_else(|| default_message.to_string());
    let payload = json!({
        "error": {
            "type": "http_error",
            "message": message,
        }
    });
    build_json_error_response(
        status_code.as_u16(),
        trace_id,
        control_decision,
        payload,
        None,
    )
}

pub(crate) async fn rewrite_niffler_upstream_error_response(
    state: &AppState,
    context: NifflerUpstreamErrorContext,
    upstream_status_code: u16,
    body_json: Option<&Value>,
    body_bytes: &[u8],
) -> Option<NifflerErrorRewrite> {
    let api_key_id = context.api_key_id.as_deref()?.trim();
    if api_key_id.is_empty() {
        return None;
    }
    let decision = match resolve_niffler_runtime_rollout_decision(state, api_key_id).await {
        Ok(decision) if decision.enable_error_return_rules => decision,
        Ok(_) => return None,
        Err(error) => {
            warn!(
                event_name = "niffler_error_return_rollout_lookup_failed",
                log_type = "ops",
                api_key_id,
                error = ?error,
                "gateway skipped niffler upstream error rewrite because rollout lookup failed"
            );
            return None;
        }
    };
    if !decision.enable_error_return_rules {
        return None;
    }

    let settings =
        match load_error_return_settings(state, NifflerErrorResponseScope::Upstream, None).await {
            Some(settings) => settings,
            None => return None,
        };
    let text = error_text_for_matching(body_json, body_bytes);
    let setting = select_upstream_setting(
        &settings,
        context.upstream_service_id.as_deref(),
        upstream_status_code,
        text.as_deref(),
    )?;
    let rewritten = rewrite_error_body(body_json, upstream_status_code, text.as_deref(), setting)?;

    if setting.handling_step == Some(NifflerUpstreamErrorHandlingStep::RiskKeyword) {
        record_account_risk_event(
            state,
            &context,
            upstream_status_code,
            setting,
            matched_text_for_setting(setting, text.as_deref()),
        )
        .await;
    }

    Some(rewritten)
}

async fn resolve_platform_error_message(
    state: &AppState,
    control_decision: Option<&GatewayControlDecision>,
    status_code: u16,
    code: &str,
    default_message: &str,
) -> Option<String> {
    let api_key_id = control_decision
        .and_then(|decision| decision.auth_context.as_ref())
        .map(|auth| auth.api_key_id.as_str())?
        .trim();
    if api_key_id.is_empty() {
        return None;
    }
    let decision = match resolve_niffler_runtime_rollout_decision(state, api_key_id).await {
        Ok(decision) if decision.enable_error_return_rules => decision,
        Ok(_) => return None,
        Err(error) => {
            warn!(
                event_name = "niffler_error_return_rollout_lookup_failed",
                log_type = "ops",
                api_key_id,
                error = ?error,
                "gateway skipped niffler platform error rewrite because rollout lookup failed"
            );
            return None;
        }
    };
    if !decision.enable_error_return_rules {
        return None;
    }

    let settings =
        load_error_return_settings(state, NifflerErrorResponseScope::Platform, None).await?;
    let match_text = format!("{code}\n{default_message}");
    let setting = select_platform_setting(&settings, status_code, match_text.as_str())?;
    Some(apply_response_mode(
        setting.response_mode,
        default_message,
        setting.match_text.as_deref(),
        setting.user_message.as_str(),
    ))
}

async fn load_error_return_settings(
    state: &AppState,
    scope: NifflerErrorResponseScope,
    upstream_service_id: Option<&str>,
) -> Option<Vec<StoredNifflerErrorReturnSetting>> {
    let query = NifflerErrorReturnSettingListQuery {
        scope: Some(scope),
        upstream_service_id: upstream_service_id.map(str::to_string),
        include_inactive: false,
        offset: 0,
        limit: ERROR_RETURN_RULE_LIMIT,
    };
    match state.list_niffler_error_return_settings(&query).await {
        Ok(page) => Some(page.items),
        Err(error) => {
            warn!(
                event_name = "niffler_error_return_settings_lookup_failed",
                log_type = "ops",
                scope = scope.as_str(),
                upstream_service_id = upstream_service_id.unwrap_or("-"),
                error = ?error,
                "gateway skipped niffler error rewrite because settings lookup failed"
            );
            None
        }
    }
}

fn build_json_error_response(
    status_code: u16,
    trace_id: &str,
    control_decision: Option<&GatewayControlDecision>,
    payload: Value,
    extra_headers: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>, GatewayError> {
    let body =
        serde_json::to_vec(&payload).map_err(|err| GatewayError::Internal(err.to_string()))?;
    let mut headers =
        BTreeMap::from([("content-type".to_string(), "application/json".to_string())]);
    if let Some(extra_headers) = extra_headers {
        headers.extend(extra_headers);
    }
    build_client_response_from_parts(
        status_code,
        &headers,
        Body::from(body),
        trace_id,
        control_decision,
    )
}

fn select_platform_setting<'a>(
    settings: &'a [StoredNifflerErrorReturnSetting],
    status_code: u16,
    text: &str,
) -> Option<&'a StoredNifflerErrorReturnSetting> {
    settings
        .iter()
        .filter(|setting| setting.is_active && setting.scope == NifflerErrorResponseScope::Platform)
        .filter(|setting| setting_matches(setting, status_code, Some(text)))
        .max_by_key(|setting| (match_specificity(setting), setting.created_at_unix_ms))
}

fn select_upstream_setting<'a>(
    settings: &'a [StoredNifflerErrorReturnSetting],
    upstream_service_id: Option<&str>,
    status_code: u16,
    text: Option<&str>,
) -> Option<&'a StoredNifflerErrorReturnSetting> {
    settings
        .iter()
        .filter(|setting| setting.is_active && setting.scope == NifflerErrorResponseScope::Upstream)
        .filter(|setting| {
            setting
                .upstream_service_id
                .as_deref()
                .is_none_or(|expected| upstream_service_id == Some(expected))
        })
        .filter(|setting| upstream_setting_matches(setting, status_code, text))
        .min_by_key(|setting| {
            (
                setting
                    .handling_step
                    .map(NifflerUpstreamErrorHandlingStep::priority)
                    .unwrap_or(u8::MAX),
                std::cmp::Reverse(u8::from(setting.upstream_service_id.is_some())),
                std::cmp::Reverse(match_specificity(setting)),
                std::cmp::Reverse(setting.created_at_unix_ms),
            )
        })
}

fn upstream_setting_matches(
    setting: &StoredNifflerErrorReturnSetting,
    status_code: u16,
    text: Option<&str>,
) -> bool {
    match setting.handling_step {
        Some(NifflerUpstreamErrorHandlingStep::RiskKeyword)
        | Some(NifflerUpstreamErrorHandlingStep::ContactOrMarketingReplacement) => {
            setting.match_text.as_deref().is_some_and(|needle| {
                text.is_some_and(|text| text_contains(text, needle))
                    && status_matches(setting.match_status_code, status_code)
            })
        }
        Some(NifflerUpstreamErrorHandlingStep::StatusCodeMessage) => {
            setting.match_status_code == Some(status_code)
                && text_matches(setting.match_text.as_deref(), text)
        }
        Some(NifflerUpstreamErrorHandlingStep::DefaultUpstreamMessage) => {
            setting_matches(setting, status_code, text)
        }
        None => false,
    }
}

fn setting_matches(
    setting: &StoredNifflerErrorReturnSetting,
    status_code: u16,
    text: Option<&str>,
) -> bool {
    status_matches(setting.match_status_code, status_code)
        && text_matches(setting.match_text.as_deref(), text)
}

fn status_matches(expected: Option<u16>, actual: u16) -> bool {
    expected.is_none_or(|expected| expected == actual)
}

fn text_matches(expected: Option<&str>, actual: Option<&str>) -> bool {
    expected.is_none_or(|needle| actual.is_some_and(|text| text_contains(text, needle)))
}

fn text_contains(text: &str, needle: &str) -> bool {
    let needle = needle.trim();
    !needle.is_empty() && text.to_lowercase().contains(&needle.to_lowercase())
}

fn match_specificity(setting: &StoredNifflerErrorReturnSetting) -> u8 {
    u8::from(setting.match_status_code.is_some()) + u8::from(setting.match_text.is_some()) * 2
}

fn rewrite_error_body(
    body_json: Option<&Value>,
    status_code: u16,
    original_text: Option<&str>,
    setting: &StoredNifflerErrorReturnSetting,
) -> Option<NifflerErrorRewrite> {
    let original_message =
        extract_error_message(body_json).or_else(|| original_text.map(str::to_string));
    let original_message =
        original_message.unwrap_or_else(|| format!("上游返回错误 {status_code}"));
    let message = apply_response_mode(
        setting.response_mode,
        original_message.as_str(),
        setting.match_text.as_deref(),
        setting.user_message.as_str(),
    );
    let mut next = body_json.cloned().unwrap_or_else(|| {
        json!({
            "error": {
                "type": "upstream_error",
                "code": status_code,
            }
        })
    });
    set_error_message(&mut next, message);
    let body_bytes = serde_json::to_vec(&next).ok()?;
    Some(NifflerErrorRewrite {
        body_json: next,
        body_bytes,
    })
}

fn apply_response_mode(
    mode: NifflerUserResponseMode,
    original_message: &str,
    match_text: Option<&str>,
    user_message: &str,
) -> String {
    match mode {
        NifflerUserResponseMode::Replace => user_message.to_string(),
        NifflerUserResponseMode::Append => {
            if original_message.trim().is_empty() {
                user_message.to_string()
            } else {
                format!("{}\n\n{}", original_message.trim(), user_message)
            }
        }
        NifflerUserResponseMode::Redact => {
            match match_text.map(str::trim).filter(|value| !value.is_empty()) {
                Some(needle) if original_message.contains(needle) => {
                    original_message.replace(needle, user_message)
                }
                _ => user_message.to_string(),
            }
        }
    }
}

fn extract_error_message(body_json: Option<&Value>) -> Option<String> {
    let body = body_json?;
    body.get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            body.get("message")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

fn set_error_message(body_json: &mut Value, message: String) {
    if let Some(error) = body_json.get_mut("error").and_then(Value::as_object_mut) {
        error.insert("message".to_string(), Value::String(message));
        return;
    }
    if let Some(object) = body_json.as_object_mut() {
        object.insert("message".to_string(), Value::String(message));
        return;
    }
    *body_json = json!({
        "error": {
            "type": "upstream_error",
            "message": message,
        }
    });
}

fn error_text_for_matching(body_json: Option<&Value>, body_bytes: &[u8]) -> Option<String> {
    extract_error_message(body_json)
        .or_else(|| body_json.and_then(|value| serde_json::to_string(value).ok()))
        .or_else(|| std::str::from_utf8(body_bytes).ok().map(str::to_string))
}

fn matched_text_for_setting(
    setting: &StoredNifflerErrorReturnSetting,
    text: Option<&str>,
) -> Option<String> {
    let needle = setting.match_text.as_deref()?.trim();
    if needle.is_empty() {
        return None;
    }
    text.filter(|value| text_contains(value, needle))?;
    Some(needle.to_string())
}

async fn record_account_risk_event(
    state: &AppState,
    context: &NifflerUpstreamErrorContext,
    upstream_status_code: u16,
    setting: &StoredNifflerErrorReturnSetting,
    matched_text: Option<String>,
) {
    let Some(upstream_account_id) = context
        .upstream_account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    let now = current_unix_ms();
    let id = Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        format!(
            "niffler-account-risk:{}:{}:{}:{}",
            context.request_id.as_deref().unwrap_or("-"),
            upstream_account_id,
            setting.id,
            upstream_status_code
        )
        .as_bytes(),
    )
    .to_string();
    let record = CreateNifflerAccountRiskEventRecord {
        id,
        upstream_service_id: context.upstream_service_id.clone(),
        upstream_account_id: upstream_account_id.to_string(),
        request_id: context.request_id.clone(),
        user_id: context.user_id.clone(),
        api_key_id: context.api_key_id.clone(),
        model_name: context.model_name.clone(),
        rule_id: Some(setting.id.clone()),
        matched_text,
        upstream_status_code: Some(upstream_status_code),
        action: setting.account_protection_action,
        created_at_unix_ms: now,
    };
    if let Err(error) = state.create_niffler_account_risk_event(record).await {
        warn!(
            event_name = "niffler_account_risk_event_write_failed",
            log_type = "ops",
            request_id = context.request_id.as_deref().unwrap_or("-"),
            upstream_account_id,
            rule_id = setting.id.as_str(),
            error = ?error,
            "gateway failed to write niffler account risk event"
        );
    }
}

#[derive(Debug)]
struct PlatformError {
    status_code: u16,
    code: &'static str,
    error_type: &'static str,
    default_message: String,
    details: Value,
}

impl PlatformError {
    fn from_auth_rejection(rejection: &GatewayLocalAuthRejection) -> Self {
        match rejection {
            GatewayLocalAuthRejection::InvalidApiKey => Self::simple(
                StatusCode::UNAUTHORIZED,
                "invalid_api_key",
                "http_error",
                "无效的API密钥",
            ),
            GatewayLocalAuthRejection::LockedApiKey => Self::simple(
                StatusCode::FORBIDDEN,
                "locked_api_key",
                "http_error",
                "该密钥已被管理员锁定，请联系管理员",
            ),
            GatewayLocalAuthRejection::WalletUnavailable => Self::simple(
                StatusCode::FORBIDDEN,
                "wallet_unavailable",
                "http_error",
                "钱包不可用",
            ),
            GatewayLocalAuthRejection::BalanceDenied { remaining } => Self {
                status_code: StatusCode::TOO_MANY_REQUESTS.as_u16(),
                code: "balance_exceeded",
                error_type: "balance_exceeded",
                default_message: match remaining {
                    Some(value) => format!("余额不足（剩余: ${value:.2}）"),
                    None => "余额不足".to_string(),
                },
                details: json!({
                    "balance_type": "USD",
                    "remaining": remaining,
                }),
            },
            GatewayLocalAuthRejection::ProviderNotAllowed { provider } => Self::simple(
                StatusCode::FORBIDDEN,
                "provider_not_allowed",
                "http_error",
                format!("当前用户、用户组或密钥的访问控制策略不允许访问 {provider} 提供商"),
            ),
            GatewayLocalAuthRejection::ApiFormatNotAllowed { api_format } => Self::simple(
                StatusCode::FORBIDDEN,
                "api_format_not_allowed",
                "http_error",
                format!("当前用户、用户组或密钥的访问控制策略不允许访问 {api_format} 格式"),
            ),
            GatewayLocalAuthRejection::ModelNotAllowed { model } => Self::simple(
                StatusCode::FORBIDDEN,
                "model_not_allowed",
                "http_error",
                format!("当前用户、用户组或密钥的访问控制策略不允许访问模型 {model}"),
            ),
        }
    }

    fn simple(
        status_code: StatusCode,
        code: &'static str,
        error_type: &'static str,
        default_message: impl Into<String>,
    ) -> Self {
        Self {
            status_code: status_code.as_u16(),
            code,
            error_type,
            default_message: default_message.into(),
            details: Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setting(
        id: &str,
        scope: NifflerErrorResponseScope,
        status: Option<u16>,
        text: Option<&str>,
        step: Option<NifflerUpstreamErrorHandlingStep>,
        message: &str,
        created_at: u64,
    ) -> StoredNifflerErrorReturnSetting {
        StoredNifflerErrorReturnSetting {
            id: id.to_string(),
            scope,
            upstream_service_id: None,
            match_status_code: status,
            match_text: text.map(str::to_string),
            handling_step: step,
            response_mode: NifflerUserResponseMode::Replace,
            user_message: message.to_string(),
            account_protection_action: NifflerAccountProtectionAction::RecordOnly,
            pause_duration: None,
            is_active: true,
            created_at_unix_ms: created_at,
            updated_at_unix_ms: created_at,
        }
    }

    #[test]
    fn platform_setting_prefers_text_over_status_default() {
        let settings = vec![
            setting(
                "status",
                NifflerErrorResponseScope::Platform,
                Some(403),
                None,
                None,
                "status message",
                20,
            ),
            setting(
                "text",
                NifflerErrorResponseScope::Platform,
                None,
                Some("model_not_allowed"),
                None,
                "text message",
                10,
            ),
        ];
        let selected = select_platform_setting(&settings, 403, "model_not_allowed\n模型不可用")
            .expect("setting should match");
        assert_eq!(selected.id, "text");
    }

    #[test]
    fn upstream_setting_uses_handling_step_priority() {
        let settings = vec![
            setting(
                "default",
                NifflerErrorResponseScope::Upstream,
                None,
                None,
                Some(NifflerUpstreamErrorHandlingStep::DefaultUpstreamMessage),
                "default",
                30,
            ),
            setting(
                "risk",
                NifflerErrorResponseScope::Upstream,
                None,
                Some("abuse"),
                Some(NifflerUpstreamErrorHandlingStep::RiskKeyword),
                "risk",
                10,
            ),
        ];
        let selected = select_upstream_setting(&settings, None, 403, Some("network abuse"))
            .expect("setting should match");
        assert_eq!(selected.id, "risk");
    }

    #[test]
    fn upstream_setting_prefers_service_specific_rule_over_global_rule() {
        let mut global = setting(
            "global",
            NifflerErrorResponseScope::Upstream,
            Some(403),
            None,
            Some(NifflerUpstreamErrorHandlingStep::StatusCodeMessage),
            "global",
            30,
        );
        let mut service = setting(
            "service",
            NifflerErrorResponseScope::Upstream,
            Some(403),
            None,
            Some(NifflerUpstreamErrorHandlingStep::StatusCodeMessage),
            "service",
            10,
        );
        service.upstream_service_id = Some("svc-1".to_string());
        global.upstream_service_id = None;
        let settings = vec![global, service];
        let selected = select_upstream_setting(&settings, Some("svc-1"), 403, Some("error"))
            .expect("setting should match");
        assert_eq!(selected.id, "service");
    }

    #[test]
    fn rewrite_preserves_error_shape_and_replaces_message() {
        let setting = setting(
            "replace",
            NifflerErrorResponseScope::Upstream,
            Some(403),
            None,
            Some(NifflerUpstreamErrorHandlingStep::StatusCodeMessage),
            "请联系平台客服处理",
            10,
        );
        let body = json!({
            "error": {
                "type": "upstream",
                "message": "vendor ad",
            }
        });
        let rewrite = rewrite_error_body(Some(&body), 403, None, &setting)
            .expect("rewrite should be created");
        assert_eq!(rewrite.body_json["error"]["message"], "请联系平台客服处理");
        assert_eq!(rewrite.body_json["error"]["type"], "upstream");
    }
}
