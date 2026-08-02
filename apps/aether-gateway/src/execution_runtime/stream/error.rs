use std::collections::BTreeMap;

use aether_contracts::{StreamFrame, StreamFramePayload};
use axum::http::StatusCode;
use base64::Engine as _;
use futures_util::StreamExt;
use serde_json::{json, Map, Value};
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::warn;

use crate::execution_runtime::ndjson::decode_stream_frame_ndjson;
use crate::execution_runtime::submission::{has_nested_error, strip_utf8_bom_and_ws};
use crate::GatewayError;
use crate::MAX_ERROR_BODY_BYTES;

#[derive(Debug)]
pub(super) enum StreamPrefetchInspection {
    NeedMore,
    NonError,
    EmbeddedError(serde_json::Value),
}

pub(super) fn decode_stream_error_body(
    headers: &BTreeMap<String, String>,
    error_body: &[u8],
) -> (Option<serde_json::Value>, Option<String>) {
    if error_body.is_empty() {
        return (None, None);
    }

    let content_type = headers
        .get("content-type")
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_default();
    let looks_json = content_type.contains("json") || content_type.ends_with("+json");
    if looks_json {
        if let Ok(json_body) = serde_json::from_slice::<serde_json::Value>(error_body) {
            return (Some(json_body), None);
        }
    }

    (
        None,
        Some(base64::engine::general_purpose::STANDARD.encode(error_body)),
    )
}

fn header_value_case_insensitive<'a>(
    headers: &'a BTreeMap<String, String>,
    name: &str,
) -> Option<&'a str> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn remove_header_case_insensitive(headers: &mut BTreeMap<String, String>, name: &str) {
    let keys = headers
        .keys()
        .filter(|key| key.eq_ignore_ascii_case(name))
        .cloned()
        .collect::<Vec<_>>();
    for key in keys {
        headers.remove(&key);
    }
}

pub(super) fn should_synthesize_non_success_stream_error_body(
    status_code: u16,
    error_body: &[u8],
) -> bool {
    !(200..300).contains(&status_code)
        && ((300..400).contains(&status_code) || error_body.is_empty())
}

pub(super) fn build_synthetic_non_success_stream_error_body(
    status_code: u16,
    headers: &BTreeMap<String, String>,
) -> Value {
    let mut error = Map::from_iter([
        (
            "type".to_string(),
            Value::String("execution_runtime_non_success_status".to_string()),
        ),
        (
            "message".to_string(),
            Value::String(format!(
                "execution runtime stream returned non-success status {status_code}"
            )),
        ),
        ("code".to_string(), Value::from(status_code)),
        ("upstream_status".to_string(), Value::from(status_code)),
    ]);
    if let Some(location) = header_value_case_insensitive(headers, "location") {
        error.insert("location".to_string(), Value::String(location.to_string()));
    }

    Value::Object(Map::from_iter([(
        "error".to_string(),
        Value::Object(error),
    )]))
}

pub(super) fn synthetic_error_response_headers(
    mut headers: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    remove_header_case_insensitive(&mut headers, "content-encoding");
    remove_header_case_insensitive(&mut headers, "content-length");
    remove_header_case_insensitive(&mut headers, "content-type");
    remove_header_case_insensitive(&mut headers, "location");
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers
}

fn client_error_status_code_for_upstream_status(status_code: u16) -> u16 {
    if (300..400).contains(&status_code) || status_code < 200 {
        StatusCode::BAD_GATEWAY.as_u16()
    } else {
        status_code
    }
}

pub(super) fn stream_client_error_status_code_for_upstream_status(status_code: u16) -> u16 {
    client_error_status_code_for_upstream_status(status_code)
}

pub(super) fn inspect_prefetched_stream_body(
    headers: &BTreeMap<String, String>,
    body: &[u8],
) -> StreamPrefetchInspection {
    if body.is_empty() {
        return StreamPrefetchInspection::NeedMore;
    }

    let stripped = strip_utf8_bom_and_ws(body);
    let content_type = headers
        .get("content-type")
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_default();
    let looks_json = content_type.contains("json") || content_type.ends_with("+json");
    if looks_json || stripped.starts_with(b"{") || stripped.starts_with(b"[") {
        if let Ok(json_body) = serde_json::from_slice::<serde_json::Value>(stripped) {
            return if has_nested_error(&json_body) {
                StreamPrefetchInspection::EmbeddedError(json_body)
            } else {
                StreamPrefetchInspection::NonError
            };
        }
    }

    if let Some(inspection) = inspect_prefetched_sse_body(body) {
        return inspection;
    }

    let text = String::from_utf8_lossy(body);
    let mut saw_meaningful_line = false;
    for line in text.lines() {
        let line = line.trim_matches('\r').trim();
        if line.is_empty() || line.starts_with(':') || line.starts_with("event:") {
            continue;
        }

        let data_line = line.strip_prefix("data: ").unwrap_or(line).trim();
        if data_line.is_empty() {
            continue;
        }
        if data_line == "[DONE]" {
            return StreamPrefetchInspection::NonError;
        }

        saw_meaningful_line = true;
        match serde_json::from_str::<serde_json::Value>(data_line) {
            Ok(json_body) => {
                return if has_nested_error(&json_body) {
                    StreamPrefetchInspection::EmbeddedError(json_body)
                } else {
                    StreamPrefetchInspection::NonError
                };
            }
            Err(_) => {
                if data_line.ends_with('}') || data_line.ends_with(']') {
                    return StreamPrefetchInspection::NonError;
                }
            }
        }
    }

    if saw_meaningful_line {
        StreamPrefetchInspection::NonError
    } else {
        StreamPrefetchInspection::NeedMore
    }
}

fn inspect_prefetched_sse_body(body: &[u8]) -> Option<StreamPrefetchInspection> {
    let text = String::from_utf8_lossy(body);
    let mut current_event: Option<String> = None;
    let mut data_lines = Vec::new();
    let mut saw_sse_line = false;

    for raw_line in text.split_inclusive('\n') {
        if !raw_line.ends_with('\n') {
            break;
        }
        let line = raw_line.trim_end_matches(&['\r', '\n'][..]).trim();

        if let Some(event_name) = line.strip_prefix("event:").map(str::trim) {
            saw_sse_line = true;
            current_event = Some(event_name.to_string());
            continue;
        }
        if let Some(data) = line.strip_prefix("data:").map(str::trim) {
            saw_sse_line = true;
            data_lines.push(data.to_string());
            continue;
        }
        if line.starts_with(':') {
            saw_sse_line = true;
            continue;
        }
        if !line.is_empty() {
            continue;
        }
        if data_lines.is_empty() {
            current_event = None;
            continue;
        }

        let data = data_lines.join("\n");
        data_lines.clear();
        if data == "[DONE]" {
            return Some(StreamPrefetchInspection::NonError);
        }
        let Ok(event_json) = serde_json::from_str::<Value>(&data) else {
            return Some(StreamPrefetchInspection::NonError);
        };
        let event_name = current_event.take();
        let event_type = event_json
            .get("type")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or(event_name)
            .unwrap_or_default();

        if event_type == "response.failed" {
            let error = event_json
                .pointer("/response/error")
                .or_else(|| event_json.get("error"))
                .cloned()
                .unwrap_or_else(|| {
                    json!({
                        "type": "upstream_error",
                        "message": "OpenAI upstream response failed"
                    })
                });
            return Some(StreamPrefetchInspection::EmbeddedError(json!({
                "error": error
            })));
        }
        if event_type == "error" {
            let error = event_json
                .get("error")
                .cloned()
                .unwrap_or_else(|| event_json.clone());
            return Some(StreamPrefetchInspection::EmbeddedError(json!({
                "error": error
            })));
        }
        if matches!(
            event_type.as_str(),
            "response.created" | "response.queued" | "response.in_progress"
        ) {
            continue;
        }
        if has_nested_error(&event_json) {
            return Some(StreamPrefetchInspection::EmbeddedError(event_json));
        }

        // Any other complete Responses event is observable output. Once it is sent,
        // this request must stay on the current account and may no longer fail over.
        return Some(StreamPrefetchInspection::NonError);
    }

    saw_sse_line.then_some(StreamPrefetchInspection::NeedMore)
}

pub(super) async fn collect_error_body<R>(
    lines: &mut FramedRead<R, LinesCodec>,
) -> Result<Vec<u8>, GatewayError>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut body = Vec::new();
    while let Some(frame) = read_next_frame(lines).await? {
        match frame.payload {
            StreamFramePayload::Data { chunk_b64, text } => {
                let chunk = if let Some(chunk_b64) = chunk_b64 {
                    base64::engine::general_purpose::STANDARD
                        .decode(chunk_b64)
                        .map_err(|err| GatewayError::Internal(err.to_string()))?
                } else {
                    text.unwrap_or_default().into_bytes()
                };
                body.extend_from_slice(&chunk);
                if body.len() >= MAX_ERROR_BODY_BYTES {
                    body.truncate(MAX_ERROR_BODY_BYTES);
                    break;
                }
            }
            StreamFramePayload::Telemetry { .. } => {}
            StreamFramePayload::Eof { .. } => break,
            StreamFramePayload::Error { error } => {
                warn!(error = %error.message, "execution runtime stream emitted error frame while collecting error body");
                break;
            }
            StreamFramePayload::Headers { .. } => {}
        }
    }
    Ok(body)
}

pub(super) async fn read_next_frame<R>(
    lines: &mut FramedRead<R, LinesCodec>,
) -> Result<Option<StreamFrame>, GatewayError>
where
    R: tokio::io::AsyncRead + Unpin,
{
    while let Some(line) = lines.next().await {
        let line = line.map_err(|err| GatewayError::Internal(err.to_string()))?;
        if line.trim().is_empty() {
            continue;
        }
        let frame = decode_stream_frame_ndjson(line.as_bytes())?;
        return Ok(Some(frame));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::{inspect_prefetched_stream_body, StreamPrefetchInspection};

    fn sse_headers() -> BTreeMap<String, String> {
        BTreeMap::from([("content-type".to_string(), "text/event-stream".to_string())])
    }

    #[test]
    fn openai_responses_preamble_needs_more_data() {
        let body = concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\",\"response\":{\"status\":\"in_progress\"}}\n\n"
        );

        assert!(matches!(
            inspect_prefetched_stream_body(&sse_headers(), body.as_bytes()),
            StreamPrefetchInspection::NeedMore
        ));
    }

    #[test]
    fn lifecycle_events_do_not_commit_client_output() {
        let mut body = String::new();
        for _ in 0..8 {
            body.push_str(concat!(
                "event: response.in_progress\n",
                "data: {\"type\":\"response.in_progress\",\"response\":{\"status\":\"in_progress\"}}\n\n"
            ));
        }

        assert!(matches!(
            inspect_prefetched_stream_body(&sse_headers(), body.as_bytes()),
            StreamPrefetchInspection::NeedMore
        ));
    }

    #[test]
    fn openai_responses_failed_event_becomes_embedded_error() {
        let body = concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\",\"response\":{\"status\":\"in_progress\"}}\n\n",
            "event: response.failed\n",
            "data: {\"type\":\"response.failed\",\"response\":{\"status\":\"failed\",\"error\":{\"code\":\"server_is_overloaded\",\"message\":\"Selected model is at capacity. Please try a different model.\"}}}\n\n"
        );

        let StreamPrefetchInspection::EmbeddedError(error) =
            inspect_prefetched_stream_body(&sse_headers(), body.as_bytes())
        else {
            panic!("response.failed should be detected as an embedded error");
        };
        assert_eq!(
            error,
            json!({
                "error": {
                    "code": "server_is_overloaded",
                    "message": "Selected model is at capacity. Please try a different model."
                }
            })
        );
    }

    #[test]
    fn response_failed_after_queued_event_is_an_embedded_error() {
        let body = concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\"}\n\n",
            "event: response.queued\n",
            "data: {\"type\":\"response.queued\",\"response\":{\"status\":\"queued\"}}\n\n",
            "event: response.failed\n",
            "data: {\"type\":\"response.failed\",\"response\":{\"status\":\"failed\",\"error\":{\"code\":\"server_is_overloaded\"}}}\n\n"
        );

        let StreamPrefetchInspection::EmbeddedError(body) =
            inspect_prefetched_stream_body(&sse_headers(), body.as_bytes())
        else {
            panic!("response.queued must not commit client output");
        };
        assert_eq!(body["error"]["code"], "server_is_overloaded");
    }

    #[test]
    fn incomplete_response_failed_event_waits_for_the_event_boundary() {
        let body = concat!(
            "event: response.failed\n",
            "data: {\"type\":\"response.failed\",\"response\":{\"error\":{\"code\":\"server_is_overloaded\"}}}"
        );

        assert!(matches!(
            inspect_prefetched_stream_body(&sse_headers(), body.as_bytes()),
            StreamPrefetchInspection::NeedMore
        ));
    }

    #[test]
    fn sse_event_name_does_not_leak_into_the_next_event() {
        let body = concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\"}\n\n",
            "data: {\"response\":{\"status\":\"in_progress\"}}\n\n"
        );

        assert!(matches!(
            inspect_prefetched_stream_body(&sse_headers(), body.as_bytes()),
            StreamPrefetchInspection::NonError
        ));
    }

    #[test]
    fn top_level_error_event_is_an_embedded_error() {
        let body = concat!(
            "event: error\n",
            "data: {\"type\":\"error\",\"code\":\"server_is_overloaded\",\"message\":\"retry\"}\n\n"
        );

        let StreamPrefetchInspection::EmbeddedError(body) =
            inspect_prefetched_stream_body(&sse_headers(), body.as_bytes())
        else {
            panic!("top-level error should be detected");
        };
        assert_eq!(body["error"]["code"], "server_is_overloaded");
    }

    #[test]
    fn openai_responses_visible_output_ends_prefetch() {
        let body = concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\",\"response\":{\"status\":\"in_progress\"}}\n\n",
            "event: response.output_text.delta\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"hello\"}\n\n"
        );

        assert!(matches!(
            inspect_prefetched_stream_body(&sse_headers(), body.as_bytes()),
            StreamPrefetchInspection::NonError
        ));
    }
}
