use regex::Regex;
use serde_json::Value;

use super::{LocalFailoverPolicy, LocalFailoverRegexRule};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ParsedLocalErrorResponse {
    message: Option<String>,
    reason: Option<String>,
    raw: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LocalFailoverInput<'a> {
    pub(crate) status_code: u16,
    pub(crate) response_text: Option<&'a str>,
}

impl<'a> LocalFailoverInput<'a> {
    pub(crate) fn new(status_code: u16, response_text: Option<&'a str>) -> Self {
        Self {
            status_code,
            response_text: response_text
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalFailoverClassification {
    UseDefault,
    StopStatusCode,
    StopErrorPattern,
    StopExecutionError,
    RetrySuccessPattern,
    RetryStatusCode,
    RetryUpstreamFailure,
}

impl LocalFailoverClassification {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::UseDefault => "use_default",
            Self::StopStatusCode => "stop_status_code",
            Self::StopErrorPattern => "stop_error_pattern",
            Self::StopExecutionError => "stop_execution_error",
            Self::RetrySuccessPattern => "retry_success_pattern",
            Self::RetryStatusCode => "retry_status_code",
            Self::RetryUpstreamFailure => "retry_upstream_failure",
        }
    }
}

pub(crate) fn classify_local_failover(
    policy: &LocalFailoverPolicy,
    input: LocalFailoverInput<'_>,
) -> LocalFailoverClassification {
    if input
        .response_text
        .is_some_and(response_text_is_context_window_error)
    {
        return LocalFailoverClassification::StopErrorPattern;
    }

    if input.response_text.is_some_and(|response_text| {
        is_retryable_openai_transient_processing_error(input.status_code, response_text)
    }) {
        return LocalFailoverClassification::RetryUpstreamFailure;
    }

    if policy.stop_status_codes.contains(&input.status_code) {
        return LocalFailoverClassification::StopStatusCode;
    }

    if input.status_code >= 400
        && input.response_text.is_some_and(|text| {
            policy
                .error_stop_patterns
                .iter()
                .any(|rule| local_failover_regex_rule_matches(rule, text, input.status_code))
        })
    {
        return LocalFailoverClassification::StopErrorPattern;
    }

    if (400..500).contains(&input.status_code)
        && input
            .response_text
            .is_some_and(response_text_is_non_retryable_user_request_error)
    {
        return LocalFailoverClassification::StopErrorPattern;
    }

    if input.status_code == 200
        && input.response_text.is_some_and(|text| {
            policy
                .success_failover_patterns
                .iter()
                .any(|rule| local_failover_regex_rule_matches(rule, text, input.status_code))
        })
    {
        return LocalFailoverClassification::RetrySuccessPattern;
    }

    if policy.continue_status_codes.contains(&input.status_code) {
        return LocalFailoverClassification::RetryStatusCode;
    }

    if should_failover_local_upstream_status(input.status_code) {
        return LocalFailoverClassification::RetryUpstreamFailure;
    }

    LocalFailoverClassification::UseDefault
}

pub(crate) fn local_failover_error_message(response_text: Option<&str>) -> Option<String> {
    let parsed = parse_local_error_response(response_text);
    parsed
        .message
        .or(parsed.reason)
        .or(parsed.raw)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn should_failover_local_upstream_status(status_code: u16) -> bool {
    status_code >= 400
}

fn response_text_is_non_retryable_user_request_error(response_text: &str) -> bool {
    let lower = response_text.to_ascii_lowercase();
    lower.contains("image dimensions exceed max allowed size")
        || lower.contains("max allowed size for many-image requests")
}

fn response_text_is_context_window_error(response_text: &str) -> bool {
    let lower = response_text.to_ascii_lowercase();
    if lower.contains("context_too_large") || lower.contains("context_length_exceeded") {
        return true;
    }
    if lower.contains("maximum context length") || lower.contains("max context length") {
        return true;
    }

    let has_exceeded =
        lower.contains("exceed") || lower.contains("too large") || lower.contains("too long");
    (lower.contains("context window") && has_exceeded)
        || (lower.contains("context length") && has_exceeded)
        || (lower.contains("token limit") && lower.contains("context") && has_exceeded)
}

pub(crate) fn is_retryable_openai_transient_processing_error(
    status_code: u16,
    response_text: &str,
) -> bool {
    if !matches!(status_code, 400 | 503) {
        return false;
    }

    let lower = response_text.to_ascii_lowercase();
    if lower.contains("server_is_overloaded") || lower.contains("slow_down") {
        return true;
    }
    if status_code != 400 {
        return false;
    }

    lower.contains("selected model is at capacity")
        || lower.contains("an error occurred while processing your request")
        || (lower.contains("you can retry your request")
            && lower.contains("help.openai.com")
            && lower.contains("request id"))
}

fn parse_local_error_response(response_text: Option<&str>) -> ParsedLocalErrorResponse {
    let raw = response_text
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let Some(raw_text) = raw.clone() else {
        return ParsedLocalErrorResponse::default();
    };

    let mut parsed = ParsedLocalErrorResponse {
        raw: Some(raw_text.clone()),
        ..ParsedLocalErrorResponse::default()
    };
    let Ok(value) = serde_json::from_str::<Value>(&raw_text) else {
        parsed.message = Some(raw_text);
        return parsed;
    };

    let body_object = value.as_object();
    let error_object = body_object
        .and_then(|object| object.get("error"))
        .and_then(Value::as_object);

    parsed.message = first_non_empty_json_text(error_object, &["message", "detail", "reason"])
        .or_else(|| first_non_empty_json_text(body_object, &["errorMessage"]))
        .or_else(|| {
            body_object
                .and_then(|object| object.get("error"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
        .or_else(|| first_non_empty_json_text(body_object, &["message", "detail", "reason"]));
    parsed.reason = first_non_empty_json_text(error_object, &["reason", "code", "status"])
        .or_else(|| first_non_empty_json_text(body_object, &["reason", "code", "status"]));

    let Some(message) = parsed.message.clone() else {
        return parsed;
    };
    if !message.starts_with('{') {
        return parsed;
    }

    let Ok(nested) = serde_json::from_str::<Value>(&message) else {
        return parsed;
    };
    let nested_object = nested.as_object();
    let nested_error_object = nested_object
        .and_then(|object| object.get("error"))
        .and_then(Value::as_object);
    parsed.message =
        first_non_empty_json_text(nested_error_object, &["message", "detail", "reason"])
            .or_else(|| first_non_empty_json_text(nested_object, &["message", "detail", "reason"]))
            .or(parsed.message);
    parsed.reason = parsed
        .reason
        .or_else(|| first_non_empty_json_text(nested_error_object, &["reason", "code", "status"]))
        .or_else(|| first_non_empty_json_text(nested_object, &["reason", "code", "status"]));

    parsed
}

fn first_non_empty_json_text(
    object: Option<&serde_json::Map<String, Value>>,
    keys: &[&str],
) -> Option<String> {
    let object = object?;
    for key in keys {
        let Some(value) = object.get(*key) else {
            continue;
        };
        match value {
            Value::String(text) if !text.trim().is_empty() => return Some(text.trim().to_string()),
            Value::Number(number) => return Some(number.to_string()),
            _ => {}
        }
    }
    None
}

fn local_failover_regex_rule_matches(
    rule: &LocalFailoverRegexRule,
    response_text: &str,
    status_code: u16,
) -> bool {
    if !rule.status_codes.is_empty() && !rule.status_codes.contains(&status_code) {
        return false;
    }

    Regex::new(&rule.pattern)
        .ok()
        .is_some_and(|regex| regex.is_match(response_text))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{classify_local_failover, LocalFailoverClassification, LocalFailoverInput};
    use crate::orchestration::{LocalFailoverPolicy, LocalFailoverRegexRule};

    #[test]
    fn classifier_honors_explicit_stop_before_default_retryable_status() {
        let policy = LocalFailoverPolicy {
            stop_status_codes: [503].into_iter().collect(),
            ..LocalFailoverPolicy::default()
        };

        assert_eq!(
            classify_local_failover(&policy, LocalFailoverInput::new(503, None)),
            LocalFailoverClassification::StopStatusCode
        );
    }

    #[test]
    fn classifier_retries_openai_capacity_error_before_explicit_stop() {
        let policy = LocalFailoverPolicy {
            stop_status_codes: [400].into_iter().collect(),
            ..LocalFailoverPolicy::default()
        };

        assert_eq!(
            classify_local_failover(
                &policy,
                LocalFailoverInput::new(
                    400,
                    Some(
                        "{\"error\":{\"message\":\"Selected model is at capacity. Please try a different model.\"}}"
                    )
                )
            ),
            LocalFailoverClassification::RetryUpstreamFailure
        );
    }

    #[test]
    fn classifier_retries_openai_overload_codes() {
        for response_text in [
            "{\"error\":{\"code\":\"server_is_overloaded\"}}",
            "{\"error\":{\"code\":\"slow_down\"}}",
        ] {
            assert_eq!(
                classify_local_failover(
                    &LocalFailoverPolicy::default(),
                    LocalFailoverInput::new(503, Some(response_text))
                ),
                LocalFailoverClassification::RetryUpstreamFailure
            );
        }
    }

    #[test]
    fn classifier_detects_success_failover_pattern() {
        let policy = LocalFailoverPolicy {
            success_failover_patterns: vec![LocalFailoverRegexRule {
                pattern: "relay:.*格式错误".to_string(),
                status_codes: BTreeSet::new(),
            }],
            ..LocalFailoverPolicy::default()
        };

        assert_eq!(
            classify_local_failover(
                &policy,
                LocalFailoverInput::new(200, Some("{\"error\":\"relay: 返回格式错误\"}"))
            ),
            LocalFailoverClassification::RetrySuccessPattern
        );
    }

    #[test]
    fn classifier_detects_error_stop_pattern() {
        let policy = LocalFailoverPolicy {
            error_stop_patterns: vec![LocalFailoverRegexRule {
                pattern: "content_policy_violation".to_string(),
                status_codes: [400, 403].into_iter().collect(),
            }],
            ..LocalFailoverPolicy::default()
        };

        assert_eq!(
            classify_local_failover(
                &policy,
                LocalFailoverInput::new(400, Some("{\"error\":\"content_policy_violation\"}"))
            ),
            LocalFailoverClassification::StopErrorPattern
        );
    }

    #[test]
    fn classifier_detects_success_continue_status_code() {
        let policy = LocalFailoverPolicy {
            continue_status_codes: [200].into_iter().collect(),
            ..LocalFailoverPolicy::default()
        };

        assert_eq!(
            classify_local_failover(&policy, LocalFailoverInput::new(200, None)),
            LocalFailoverClassification::RetryStatusCode
        );
    }

    #[test]
    fn classifier_retries_all_error_statuses_without_custom_rule() {
        for (status_code, response_text) in [
            (
                400,
                "{\"error\":{\"type\":\"invalid_request_error\",\"message\":\"prompt is too long\"}}",
            ),
            (
                400,
                "{\"error\":{\"message\":\"Unsupported parameter: max_tokens is not supported with this model\"}}",
            ),
            (
                400,
                "{\"error\":{\"message\":\"Unknown parameter: 'tools[0].n'.\"}}",
            ),
            (
                400,
                "{\"error\":{\"message\":\"invalid model for this endpoint\"}}",
            ),
            (
                400,
                "{\"error\":{\"message\":\"invalid `signature` in `thinking` block: signature is for a different request\"}}",
            ),
            (
                400,
                "{\"error\":{\"message\":\"resource_exhausted: quota reached\"}}",
            ),
            (
                401,
                "{\"error\":{\"type\":\"invalid_request_error\",\"message\":\"Your authentication token has been invalidated. Please try signing in again.\"}}",
            ),
            (
                402,
                "{\"error\":{\"type\":\"invalid_request_error\",\"message\":\"payment required: credit balance exhausted\"}}",
            ),
            (
                403,
                "{\"error\":{\"type\":\"invalid_request_error\",\"message\":\"verify your account before continuing\"}}",
            ),
            (429, "{\"error\":{\"message\":\"rate limited\"}}"),
            (500, "{\"error\":{\"message\":\"upstream failed\"}}"),
        ] {
            assert_eq!(
                classify_local_failover(
                    &LocalFailoverPolicy::default(),
                    LocalFailoverInput::new(status_code, Some(response_text))
                ),
                LocalFailoverClassification::RetryUpstreamFailure
            );
        }
    }

    #[test]
    fn classifier_stops_known_user_request_image_size_errors_without_custom_rule() {
        for response_text in [
            "{\"error\":{\"message\":\"At least one of the image dimensions exceed max allowed size for many-image requests: 2000 pixels\"}}",
            "{\"error\":{\"message\":\"max allowed size for many-image requests is 2000 pixels\"}}",
        ] {
            assert_eq!(
                classify_local_failover(
                    &LocalFailoverPolicy::default(),
                    LocalFailoverInput::new(400, Some(response_text))
                ),
                LocalFailoverClassification::StopErrorPattern
            );
        }
    }

    #[test]
    fn classifier_stops_context_window_errors_without_retrying_other_accounts() {
        assert_eq!(
            classify_local_failover(
                &LocalFailoverPolicy::default(),
                LocalFailoverInput::new(
                    400,
                    Some(
                        "{\"code\":-32603,\"message\":\"Internal error\",\"data\":{\"details\":\"Your input exceeds the context window of this model.\"}}"
                    )
                )
            ),
            LocalFailoverClassification::StopErrorPattern
        );
    }

    #[test]
    fn classifier_prioritizes_context_window_errors_over_retryable_processing_text() {
        assert_eq!(
            classify_local_failover(
                &LocalFailoverPolicy::default(),
                LocalFailoverInput::new(
                    400,
                    Some(
                        "{\"error\":{\"message\":\"An error occurred while processing your request because the input exceeds the context window. You can retry your request.\"}}"
                    )
                )
            ),
            LocalFailoverClassification::StopErrorPattern
        );
    }

    #[test]
    fn classifier_keeps_embedded_rate_limit_error_in_success_response_on_default_path() {
        assert_eq!(
            classify_local_failover(
                &LocalFailoverPolicy::default(),
                LocalFailoverInput::new(
                    200,
                    Some(
                        "{\"error\":{\"message\":\"quota reached\",\"type\":\"rate_limit_error\"}}"
                    )
                )
            ),
            LocalFailoverClassification::UseDefault
        );
    }
}
