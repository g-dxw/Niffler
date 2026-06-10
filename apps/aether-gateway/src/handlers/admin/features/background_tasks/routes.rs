use crate::handlers::admin::request::{AdminAppState, AdminRequestContext};
use crate::handlers::admin::shared::{
    attach_admin_audit_response, query_param_value, unix_secs_to_rfc3339,
};
use crate::task_runtime::{
    self, set_cancel_signal, TASK_KEY_PROVIDER_DELETE, TASK_KEY_PROVIDER_OAUTH_BATCH_IMPORT,
    TASK_TRIGGER_AUTH_EMAIL,
};
use crate::GatewayError;
use aether_data_contracts::repository::background_tasks::{
    BackgroundTaskKind, BackgroundTaskListQuery, BackgroundTaskStatus, StoredBackgroundTaskRun,
};
use axum::{
    body::{Body, Bytes},
    http,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

const DEFAULT_PAGE_SIZE: usize = 20;
const MAX_PAGE_SIZE: usize = 100;
const DEFAULT_EVENTS_PAGE_SIZE: usize = 50;

pub(super) async fn maybe_build_local_admin_background_tasks_response(
    state: &AdminAppState<'_>,
    request_context: &AdminRequestContext<'_>,
    request_body: Option<&Bytes>,
) -> Result<Option<Response<Body>>, GatewayError> {
    if request_context.route_family() != Some("tasks_manage") {
        return Ok(None);
    }

    match request_context.route_kind() {
        Some("list_tasks") if request_context.method() == http::Method::GET => {
            let query = request_context.query_string();
            let page = query_param_value(query, "page")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(1)
                .max(1);
            let page_size = query_param_value(query, "page_size")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(DEFAULT_PAGE_SIZE)
                .clamp(1, MAX_PAGE_SIZE);
            let kind = query_param_value(query, "kind")
                .map(|value| BackgroundTaskKind::from_database(value.as_str()))
                .transpose()
                .map_err(|err| GatewayError::Internal(err.to_string()))?;
            let status = query_param_value(query, "status")
                .map(|value| BackgroundTaskStatus::from_database(value.as_str()))
                .transpose()
                .map_err(|err| GatewayError::Internal(err.to_string()))?;
            let trigger = query_param_value(query, "trigger");
            let task_key_substring = query_param_value(query, "task_key");
            let offset = (page - 1).saturating_mul(page_size);
            let response = state
                .list_background_task_runs(&BackgroundTaskListQuery {
                    task_key_substring,
                    kind,
                    status,
                    trigger,
                    offset,
                    limit: page_size,
                })
                .await?;
            let pages = if response.total == 0 {
                0
            } else {
                (response.total + page_size - 1) / page_size
            };

            let items = response
                .items
                .iter()
                .map(|run| {
                    json!({
                        "id": run.id,
                        "task_key": run.task_key,
                        "kind": run.kind.as_database(),
                        "trigger": run.trigger,
                        "status": run.status.as_database(),
                        "attempt": run.attempt,
                        "max_attempts": run.max_attempts,
                        "owner_instance": run.owner_instance,
                        "progress_percent": run.progress_percent,
                        "progress_message": run.progress_message,
                        "payload": safe_task_payload_json(run),
                        "result": safe_task_result_json(run),
                        "error_message": run.error_message,
                        "cancel_requested": run.cancel_requested,
                        "created_by": run.created_by,
                        "created_at": unix_secs_to_rfc3339(run.created_at_unix_secs),
                        "started_at": run.started_at_unix_secs.and_then(unix_secs_to_rfc3339),
                        "finished_at": run.finished_at_unix_secs.and_then(unix_secs_to_rfc3339),
                        "updated_at": unix_secs_to_rfc3339(run.updated_at_unix_secs),
                    })
                })
                .collect::<Vec<_>>();
            let definitions = task_runtime::task_definitions()
                .iter()
                .map(|definition| {
                    json!({
                        "task_key": definition.key,
                        "kind": definition.kind.as_str(),
                        "trigger": definition.trigger,
                        "max_attempts": definition.retry_policy.max_attempts,
                        "singleton": definition.singleton,
                        "persist_history": definition.persist_history,
                    })
                })
                .collect::<Vec<_>>();

            return Ok(Some(
                Json(json!({
                    "items": items,
                    "total": response.total,
                    "page": page,
                    "page_size": page_size,
                    "pages": pages,
                    "definitions": definitions,
                }))
                .into_response(),
            ));
        }
        Some("stats") if request_context.method() == http::Method::GET => {
            let stats = state.summarize_background_task_runs().await?;
            return Ok(Some(
                Json(json!({
                    "total": stats.total,
                    "running_count": stats.running_count,
                    "by_status": stats.by_status,
                    "by_kind": stats.by_kind,
                    "registered_tasks": task_runtime::task_definitions().len(),
                }))
                .into_response(),
            ));
        }
        Some("detail") if request_context.method() == http::Method::GET => {
            let Some(run_id) = task_id_from_path(request_context.path()) else {
                return Ok(Some(
                    (
                        http::StatusCode::NOT_FOUND,
                        Json(json!({"detail":"Task not found"})),
                    )
                        .into_response(),
                ));
            };
            let Some(run) = state.find_background_task_run(run_id).await? else {
                return Ok(Some(
                    (
                        http::StatusCode::NOT_FOUND,
                        Json(json!({"detail":"Task not found"})),
                    )
                        .into_response(),
                ));
            };
            return Ok(Some(attach_admin_audit_response(
                Json(json!({
                    "id": run.id,
                    "task_key": run.task_key,
                    "kind": run.kind.as_database(),
                    "trigger": run.trigger,
                    "status": run.status.as_database(),
                    "attempt": run.attempt,
                    "max_attempts": run.max_attempts,
                    "owner_instance": run.owner_instance,
                    "progress_percent": run.progress_percent,
                    "progress_message": run.progress_message,
                    "payload": safe_task_payload_json(&run),
                    "result": safe_task_result_json(&run),
                    "error_message": run.error_message,
                    "cancel_requested": run.cancel_requested,
                    "created_by": run.created_by,
                    "created_at": unix_secs_to_rfc3339(run.created_at_unix_secs),
                    "started_at": run.started_at_unix_secs.and_then(unix_secs_to_rfc3339),
                    "finished_at": run.finished_at_unix_secs.and_then(unix_secs_to_rfc3339),
                    "updated_at": unix_secs_to_rfc3339(run.updated_at_unix_secs),
                }))
                .into_response(),
                "admin_task_detail_viewed",
                "view_task_detail",
                "background_task",
                run_id,
            )));
        }
        Some("events") if request_context.method() == http::Method::GET => {
            let Some(run_id) = nested_task_id_from_path(request_context.path(), "/events") else {
                return Ok(Some(
                    (
                        http::StatusCode::NOT_FOUND,
                        Json(json!({"detail":"Task not found"})),
                    )
                        .into_response(),
                ));
            };
            let query = request_context.query_string();
            let page = query_param_value(query, "page")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(1)
                .max(1);
            let page_size = query_param_value(query, "page_size")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(DEFAULT_EVENTS_PAGE_SIZE)
                .clamp(1, MAX_PAGE_SIZE);
            let offset = (page - 1).saturating_mul(page_size);
            let is_auth_email_task = state
                .find_background_task_run(run_id)
                .await?
                .as_ref()
                .is_some_and(|run| run.trigger == TASK_TRIGGER_AUTH_EMAIL);
            let events = state
                .list_background_task_events(run_id, offset, page_size)
                .await?;
            return Ok(Some(
                Json(json!({
                    "items": events.into_iter().map(|event| {
                        json!({
                            "id": event.id,
                            "run_id": event.run_id,
                            "event_type": event.event_type,
                            "message": event.message,
                            "payload": if is_auth_email_task {
                                serde_json::Value::Null
                            } else {
                                event.payload_json.unwrap_or(serde_json::Value::Null)
                            },
                            "created_at": unix_secs_to_rfc3339(event.created_at_unix_secs),
                        })
                    }).collect::<Vec<_>>(),
                    "page": page,
                    "page_size": page_size,
                }))
                .into_response(),
            ));
        }
        Some("cancel") if request_context.method() == http::Method::POST => {
            let Some(run_id) = nested_task_id_from_path(request_context.path(), "/cancel") else {
                return Ok(Some(
                    (
                        http::StatusCode::NOT_FOUND,
                        Json(json!({"detail":"Task not found"})),
                    )
                        .into_response(),
                ));
            };
            let now = task_runtime::now_unix_secs();
            let cancelled = state
                .request_cancel_background_task_run(run_id, now)
                .await?;
            if !cancelled {
                return Ok(Some(
                    (
                        http::StatusCode::NOT_FOUND,
                        Json(json!({ "detail": "Task not found" })),
                    )
                        .into_response(),
                ));
            }
            let _ = set_cancel_signal(state.app(), run_id).await;
            task_runtime::append_event_with_logging(
                state.app(),
                run_id,
                "cancel_requested",
                "cancel requested by admin",
                None,
            )
            .await;
            return Ok(Some(attach_admin_audit_response(
                Json(json!({
                    "id": run_id,
                    "status": "cancel_requested",
                    "message": "Task cancellation requested",
                }))
                .into_response(),
                "admin_task_cancel_requested",
                "cancel_task",
                "background_task",
                run_id,
            )));
        }
        Some("trigger") if request_context.method() == http::Method::POST => {
            let Some(task_key) = nested_task_id_from_path(request_context.path(), "/trigger")
            else {
                return Ok(Some(
                    (
                        http::StatusCode::NOT_FOUND,
                        Json(json!({"detail":"Task not found"})),
                    )
                        .into_response(),
                ));
            };
            let payload = parse_json_payload(request_body)?;
            if task_key == TASK_KEY_PROVIDER_DELETE {
                let provider_id = payload
                    .get("provider_id")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        GatewayError::Internal(
                            "admin task trigger provider delete requires provider_id".to_string(),
                        )
                    })?;
                let Some(run_id) =
                    task_runtime::submit_provider_delete_task(state, provider_id, Some("admin"))
                        .await?
                else {
                    return Ok(Some(
                        (
                            http::StatusCode::NOT_FOUND,
                            Json(json!({"detail":"Provider 不存在"})),
                        )
                            .into_response(),
                    ));
                };
                return Ok(Some(attach_admin_audit_response(
                    Json(json!({
                        "task_key": task_key,
                        "run_id": run_id,
                        "status": "queued",
                    }))
                    .into_response(),
                    "admin_task_triggered",
                    "trigger_task",
                    "background_task",
                    task_key,
                )));
            }
            if task_key == TASK_KEY_PROVIDER_OAUTH_BATCH_IMPORT {
                return Ok(Some(
                    (
                        http::StatusCode::BAD_REQUEST,
                        Json(json!({
                            "detail": "请使用 provider oauth batch import 专用接口触发该任务",
                        })),
                    )
                        .into_response(),
                ));
            }
            return Ok(Some(
                (
                    http::StatusCode::BAD_REQUEST,
                    Json(json!({
                        "detail": format!("Unsupported task_key: {task_key}"),
                    })),
                )
                    .into_response(),
            ));
        }
        _ => {}
    }

    Ok(None)
}

fn safe_task_payload_json(run: &StoredBackgroundTaskRun) -> serde_json::Value {
    if run.trigger == TASK_TRIGGER_AUTH_EMAIL {
        serde_json::Value::Null
    } else {
        run.payload_json.clone().unwrap_or(serde_json::Value::Null)
    }
}

fn safe_task_result_json(run: &StoredBackgroundTaskRun) -> serde_json::Value {
    if run.trigger != TASK_TRIGGER_AUTH_EMAIL {
        return run.result_json.clone().unwrap_or(serde_json::Value::Null);
    }
    let source = run.result_json.as_ref().or(run.payload_json.as_ref());
    let message_type = source
        .and_then(|value| value.get("message_type"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let to_email = source
        .and_then(|value| value.get("to_email"))
        .and_then(serde_json::Value::as_str)
        .map(mask_email_address);
    json!({
        "message_type": message_type,
        "to_email": to_email,
    })
}

fn mask_email_address(email: &str) -> String {
    let email = email.trim();
    let Some((local, domain)) = email.split_once('@') else {
        return "***".to_string();
    };
    let first = local.chars().next().unwrap_or('*');
    format!("{first}***@{domain}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stored_run(
        trigger: &str,
        payload_json: Option<serde_json::Value>,
        result_json: Option<serde_json::Value>,
    ) -> StoredBackgroundTaskRun {
        StoredBackgroundTaskRun {
            id: "task-1".to_string(),
            task_key: "task.key".to_string(),
            kind: BackgroundTaskKind::FireAndForget,
            trigger: trigger.to_string(),
            status: BackgroundTaskStatus::Queued,
            attempt: 0,
            max_attempts: 1,
            owner_instance: None,
            progress_percent: 0,
            progress_message: None,
            payload_json,
            result_json,
            error_message: None,
            cancel_requested: false,
            created_by: None,
            created_at_unix_secs: 1,
            started_at_unix_secs: None,
            finished_at_unix_secs: None,
            updated_at_unix_secs: 1,
        }
    }

    #[test]
    fn auth_email_task_response_hides_payload_and_keeps_safe_result() {
        let run = stored_run(
            TASK_TRIGGER_AUTH_EMAIL,
            Some(json!({
                "message_type": "verification",
                "to_email": "person@example.com",
                "subject": "验证码",
                "html_body": "<p>123456</p>",
                "text_body": "123456",
            })),
            None,
        );

        assert_eq!(safe_task_payload_json(&run), serde_json::Value::Null);
        let result = safe_task_result_json(&run);
        assert_eq!(result["message_type"], "verification");
        assert_eq!(result["to_email"], "p***@example.com");
        let result_text = result.to_string();
        assert!(!result_text.contains("123456"));
        assert!(!result_text.contains("html_body"));
        assert!(!result_text.contains("text_body"));
        assert!(!result_text.contains("subject"));
    }

    #[test]
    fn non_email_task_response_keeps_payload_and_result() {
        let payload = json!({ "provider_id": "provider-1" });
        let result = json!({ "deleted": 3 });
        let run = stored_run("manual", Some(payload.clone()), Some(result.clone()));

        assert_eq!(safe_task_payload_json(&run), payload);
        assert_eq!(safe_task_result_json(&run), result);
    }
}

fn task_id_from_path(request_path: &str) -> Option<&str> {
    let task_id = request_path.strip_prefix("/api/admin/tasks/")?;
    if task_id.is_empty() || task_id.contains('/') || task_id == "stats" {
        return None;
    }
    Some(task_id)
}

fn nested_task_id_from_path<'a>(request_path: &'a str, suffix: &str) -> Option<&'a str> {
    let task_id = request_path
        .strip_prefix("/api/admin/tasks/")?
        .strip_suffix(suffix)?;
    if task_id.is_empty() || task_id.contains('/') {
        return None;
    }
    Some(task_id)
}

fn parse_json_payload(request_body: Option<&Bytes>) -> Result<serde_json::Value, GatewayError> {
    let Some(body) = request_body else {
        return Ok(json!({}));
    };
    if body.is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_slice::<serde_json::Value>(body)
        .map_err(|err| GatewayError::Internal(format!("invalid json body: {err}")))
}
