use std::time::Duration;

use aether_data_contracts::repository::background_tasks::{
    BackgroundTaskKind, BackgroundTaskListQuery, BackgroundTaskStatus, StoredBackgroundTaskRun,
};
use tracing::warn;

use super::execute_email_delivery;
use crate::task_runtime::TASK_TRIGGER_AUTH_EMAIL;
use crate::{AppState, GatewayError};

const EMAIL_DELIVERY_INTERVAL: Duration = Duration::from_secs(5);
const EMAIL_DELIVERY_BATCH_SIZE: usize = 10;
const EMAIL_DELIVERY_LOCK_TTL: Duration = Duration::from_secs(180);

pub(crate) fn spawn_auth_email_delivery_worker(
    state: AppState,
) -> Option<tokio::task::JoinHandle<()>> {
    if !state.has_background_task_data_reader() || !state.has_background_task_data_writer() {
        return None;
    }

    Some(tokio::spawn(async move {
        let mut interval = tokio::time::interval(EMAIL_DELIVERY_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        interval.tick().await;
        loop {
            interval.tick().await;
            if let Err(err) = run_auth_email_delivery_once(&state).await {
                warn!(error = ?err, "auth email delivery worker failed");
            }
        }
    }))
}

async fn run_auth_email_delivery_once(state: &AppState) -> Result<(), GatewayError> {
    let runs = collect_pending_email_runs(state).await?;
    for run in runs {
        let lock_key = format!("task_runtime:lock:auth.email.delivery:{}", run.id);
        let lock = state
            .runtime_state
            .lock_try_acquire(
                &lock_key,
                state.tunnel.local_instance_id(),
                EMAIL_DELIVERY_LOCK_TTL,
            )
            .await
            .ok()
            .flatten();
        let Some(lock) = lock else {
            continue;
        };
        let result = execute_email_delivery(state, run).await;
        let _ = state.runtime_state.lock_release(&lock).await;
        if let Err(err) = result {
            warn!(error = ?err, "auth email delivery task failed");
        }
    }
    Ok(())
}

async fn collect_pending_email_runs(
    state: &AppState,
) -> Result<Vec<StoredBackgroundTaskRun>, GatewayError> {
    let mut runs = Vec::new();
    append_email_runs_by_status(state, BackgroundTaskStatus::Queued, &mut runs).await?;
    if runs.len() < EMAIL_DELIVERY_BATCH_SIZE {
        append_email_runs_by_status(state, BackgroundTaskStatus::Retrying, &mut runs).await?;
    }
    runs.truncate(EMAIL_DELIVERY_BATCH_SIZE);
    Ok(runs)
}

async fn append_email_runs_by_status(
    state: &AppState,
    status: BackgroundTaskStatus,
    runs: &mut Vec<StoredBackgroundTaskRun>,
) -> Result<(), GatewayError> {
    let remaining = EMAIL_DELIVERY_BATCH_SIZE.saturating_sub(runs.len());
    if remaining == 0 {
        return Ok(());
    }
    let page = state
        .list_background_task_runs(&BackgroundTaskListQuery {
            task_key_substring: None,
            kind: Some(BackgroundTaskKind::FireAndForget),
            status: Some(status),
            trigger: Some(TASK_TRIGGER_AUTH_EMAIL.to_string()),
            offset: 0,
            limit: remaining,
        })
        .await?;
    runs.extend(page.items);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_delivery_lock_covers_smtp_worst_case_timeout() {
        assert!(
            EMAIL_DELIVERY_LOCK_TTL >= Duration::from_secs(180),
            "邮件发送锁必须覆盖一次 SMTP 发送的多阶段超时窗口"
        );
    }
}
