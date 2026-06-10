use aether_data_contracts::repository::background_tasks::{
    BackgroundTaskKind, BackgroundTaskStatus, StoredBackgroundTaskRun, UpsertBackgroundTaskRun,
};
use serde_json::json;
use uuid::Uuid;

use super::{EmailMessage, SmtpConfig, send_email_blocking};
use crate::handlers::shared::{
    decrypt_catalog_secret_with_fallbacks, system_config_bool, system_config_string,
};
use crate::task_runtime::{
    TASK_TRIGGER_AUTH_EMAIL, append_event_with_logging, now_unix_secs, upsert_run_with_logging,
};
use crate::{AppState, GatewayError};

const EMAIL_DELIVERY_MAX_ATTEMPTS: u32 = 2;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct EmailDeliveryPayload {
    pub(crate) message_type: String,
    pub(crate) to_email: String,
    pub(crate) subject: String,
    pub(crate) html_body: String,
    pub(crate) text_body: String,
}

pub(crate) async fn queue_email_delivery(
    state: &AppState,
    payload: EmailDeliveryPayload,
    created_by: Option<String>,
) -> Result<String, GatewayError> {
    if !state.has_background_task_data_writer() {
        return Err(GatewayError::Internal(
            "email delivery task storage is unavailable".to_string(),
        ));
    }

    let run_id = Uuid::new_v4().to_string();
    let now = now_unix_secs();
    let run = UpsertBackgroundTaskRun {
        id: run_id.clone(),
        task_key: format!("auth.email.delivery:{run_id}"),
        kind: BackgroundTaskKind::FireAndForget,
        trigger: TASK_TRIGGER_AUTH_EMAIL.to_string(),
        status: BackgroundTaskStatus::Queued,
        attempt: 0,
        max_attempts: EMAIL_DELIVERY_MAX_ATTEMPTS,
        owner_instance: Some(state.tunnel.local_instance_id().to_string()),
        progress_percent: 0,
        progress_message: Some("等待发送邮件".to_string()),
        payload_json: Some(serde_json::to_value(&payload).map_err(|err| {
            GatewayError::Internal(format!("email delivery payload serialize failed: {err}"))
        })?),
        result_json: Some(safe_email_delivery_summary(&payload)),
        error_message: None,
        cancel_requested: false,
        created_by,
        created_at_unix_secs: now,
        started_at_unix_secs: None,
        finished_at_unix_secs: None,
        updated_at_unix_secs: now,
    };
    upsert_run_with_logging(state, run)
        .await
        .ok_or_else(|| GatewayError::Internal("email delivery task create failed".to_string()))?;
    append_event_with_logging(
        state,
        &run_id,
        "queued",
        "email delivery queued",
        Some(json!({
            "message_type": payload.message_type,
            "to_email": mask_email_address(&payload.to_email),
        })),
    )
    .await;
    Ok(run_id)
}

pub(crate) async fn execute_email_delivery(
    state: &AppState,
    run: StoredBackgroundTaskRun,
) -> Result<(), GatewayError> {
    let payload = run
        .payload_json
        .as_ref()
        .ok_or_else(|| GatewayError::Internal("email delivery payload missing".to_string()))
        .and_then(|value| {
            serde_json::from_value::<EmailDeliveryPayload>(value.clone()).map_err(|err| {
                GatewayError::Internal(format!("email delivery payload parse failed: {err}"))
            })
        })?;
    let smtp_config = read_email_smtp_config(state).await?;
    let attempt = run.attempt.saturating_add(1).max(1);
    let started_at = run.started_at_unix_secs.or_else(|| Some(now_unix_secs()));
    let running = UpsertBackgroundTaskRun {
        id: run.id.clone(),
        task_key: run.task_key.clone(),
        kind: run.kind,
        trigger: run.trigger.clone(),
        status: BackgroundTaskStatus::Running,
        attempt,
        max_attempts: run.max_attempts.max(1),
        owner_instance: Some(state.tunnel.local_instance_id().to_string()),
        progress_percent: 10,
        progress_message: Some("正在发送邮件".to_string()),
        payload_json: run.payload_json.clone(),
        result_json: Some(safe_email_delivery_summary(&payload)),
        error_message: None,
        cancel_requested: run.cancel_requested,
        created_by: run.created_by.clone(),
        created_at_unix_secs: run.created_at_unix_secs,
        started_at_unix_secs: started_at,
        finished_at_unix_secs: None,
        updated_at_unix_secs: now_unix_secs(),
    };
    let Some(running) = upsert_run_with_logging(state, running).await else {
        return Err(GatewayError::Internal(
            "email delivery task update failed".to_string(),
        ));
    };
    append_event_with_logging(
        state,
        &run.id,
        "running",
        "email delivery started",
        Some(json!({
            "message_type": payload.message_type,
            "to_email": mask_email_address(&payload.to_email),
            "attempt": attempt,
        })),
    )
    .await;

    let email = EmailMessage {
        to_email: payload.to_email.clone(),
        subject: payload.subject.clone(),
        html_body: payload.html_body.clone(),
        text_body: payload.text_body.clone(),
    };
    let send_result = tokio::task::spawn_blocking(move || send_email_blocking(smtp_config, email))
        .await
        .map_err(|err| GatewayError::Internal(err.to_string()))?;

    match send_result {
        Ok(()) => {
            let completed = with_task_status(
                running,
                BackgroundTaskStatus::Succeeded,
                100,
                "邮件已发送",
                Some(safe_email_delivery_summary(&payload)),
                None,
                Some(now_unix_secs()),
            );
            let _ = upsert_run_with_logging(state, completed).await;
            append_event_with_logging(
                state,
                &run.id,
                "succeeded",
                "email delivery completed",
                Some(json!({
                    "message_type": payload.message_type,
                    "to_email": mask_email_address(&payload.to_email),
                })),
            )
            .await;
            Ok(())
        }
        Err(err) => {
            let message = format!("{err:?}");
            let final_failure = attempt >= running.max_attempts.max(1);
            let status = if final_failure {
                BackgroundTaskStatus::Failed
            } else {
                BackgroundTaskStatus::Retrying
            };
            let progress_message = if final_failure {
                "邮件发送失败"
            } else {
                "邮件发送失败，等待重试"
            };
            let updated = with_task_status(
                running,
                status,
                100,
                progress_message,
                Some(safe_email_delivery_summary(&payload)),
                Some(message.clone()),
                final_failure.then_some(now_unix_secs()),
            );
            let _ = upsert_run_with_logging(state, updated).await;
            append_event_with_logging(
                state,
                &run.id,
                if final_failure { "failed" } else { "retrying" },
                progress_message,
                Some(json!({
                    "message_type": payload.message_type,
                    "to_email": mask_email_address(&payload.to_email),
                    "attempt": attempt,
                    "error": message,
                })),
            )
            .await;
            Ok(())
        }
    }
}

pub(crate) fn safe_email_delivery_summary(payload: &EmailDeliveryPayload) -> serde_json::Value {
    json!({
        "message_type": payload.message_type,
        "to_email": mask_email_address(&payload.to_email),
    })
}

pub(crate) fn mask_email_address(email: &str) -> String {
    let email = email.trim();
    let Some((local, domain)) = email.split_once('@') else {
        return "***".to_string();
    };
    let first = local.chars().next().unwrap_or('*');
    format!("{first}***@{domain}")
}

async fn read_email_smtp_config(state: &AppState) -> Result<SmtpConfig, GatewayError> {
    let smtp_host = state.read_system_config_json_value("smtp_host").await?;
    let smtp_from_email = state
        .read_system_config_json_value("smtp_from_email")
        .await?;
    let Some(host) = system_config_string(smtp_host.as_ref()) else {
        return Err(GatewayError::Internal("SMTP 服务器地址未配置".to_string()));
    };
    let Some(from_email) = system_config_string(smtp_from_email.as_ref()) else {
        return Err(GatewayError::Internal("发件人邮箱未配置".to_string()));
    };
    let smtp_port = state.read_system_config_json_value("smtp_port").await?;
    let smtp_user = state.read_system_config_json_value("smtp_user").await?;
    let smtp_password = state.read_system_config_json_value("smtp_password").await?;
    let smtp_use_tls = state.read_system_config_json_value("smtp_use_tls").await?;
    let smtp_use_ssl = state.read_system_config_json_value("smtp_use_ssl").await?;
    let smtp_from_name = state
        .read_system_config_json_value("smtp_from_name")
        .await?;

    let password = system_config_string(smtp_password.as_ref()).map(|value| {
        decrypt_catalog_secret_with_fallbacks(state.encryption_key(), &value).unwrap_or(value)
    });

    Ok(SmtpConfig {
        host,
        port: system_config_u16(smtp_port.as_ref(), 587),
        user: system_config_string(smtp_user.as_ref()),
        password,
        use_tls: system_config_bool(smtp_use_tls.as_ref(), true),
        use_ssl: system_config_bool(smtp_use_ssl.as_ref(), false),
        from_email,
        from_name: system_config_string(smtp_from_name.as_ref())
            .unwrap_or_else(|| "Niffler".to_string()),
    })
}

fn system_config_u16(value: Option<&serde_json::Value>, default: u16) -> u16 {
    match value {
        Some(serde_json::Value::Number(value)) => value
            .as_u64()
            .and_then(|value| u16::try_from(value).ok())
            .unwrap_or(default),
        Some(serde_json::Value::String(value)) => value.trim().parse::<u16>().unwrap_or(default),
        _ => default,
    }
}

fn with_task_status(
    run: StoredBackgroundTaskRun,
    status: BackgroundTaskStatus,
    progress_percent: u16,
    progress_message: &str,
    result_json: Option<serde_json::Value>,
    error_message: Option<String>,
    finished_at_unix_secs: Option<u64>,
) -> UpsertBackgroundTaskRun {
    UpsertBackgroundTaskRun {
        id: run.id,
        task_key: run.task_key,
        kind: run.kind,
        trigger: run.trigger,
        status,
        attempt: run.attempt,
        max_attempts: run.max_attempts,
        owner_instance: run.owner_instance,
        progress_percent,
        progress_message: Some(progress_message.to_string()),
        payload_json: run.payload_json,
        result_json,
        error_message,
        cancel_requested: run.cancel_requested,
        created_by: run.created_by,
        created_at_unix_secs: run.created_at_unix_secs,
        started_at_unix_secs: run.started_at_unix_secs,
        finished_at_unix_secs,
        updated_at_unix_secs: now_unix_secs(),
    }
}
