use std::collections::BTreeSet;

use aether_data_contracts::repository::usage::StoredRequestUsageAudit;
use sqlx::{Postgres, Row};
use uuid::Uuid;

use super::{
    find_usage_by_request_id_in_tx, insert_usage_counter_delta_with_id_in_tx,
    UsageCounterDeltaInsert, USAGE_COUNTER_KIND_PROVIDER_API_KEY,
    USAGE_COUNTER_KIND_PROVIDER_API_KEY_WINDOW,
};
use crate::error::SqlxResultExt;
use crate::repository::usage::{
    provider_api_key_usage_contribution, ProviderApiKeyUsageContribution, ProviderApiKeyUsageDelta,
};
use crate::DataLayerError;

const FIND_CONTRIBUTION_SQL: &str = r#"
SELECT
  provider_api_key_id,
  request_count,
  success_count,
  error_count,
  total_tokens,
  CAST(total_cost_usd AS DOUBLE PRECISION) AS total_cost_usd,
  total_response_time_ms,
  last_used_at_unix_secs,
  usage_created_at_unix_secs,
  window_request_count,
  window_total_tokens,
  CAST(window_total_cost_usd AS DOUBLE PRECISION) AS window_total_cost_usd,
  revision
FROM provider_api_key_usage_contributions
WHERE request_id = $1
FOR UPDATE
"#;

const UPSERT_CONTRIBUTION_SQL: &str = r#"
INSERT INTO provider_api_key_usage_contributions (
  request_id,
  provider_api_key_id,
  request_count,
  success_count,
  error_count,
  total_tokens,
  total_cost_usd,
  total_response_time_ms,
  last_used_at_unix_secs,
  usage_created_at_unix_secs,
  window_request_count,
  window_total_tokens,
  window_total_cost_usd,
  revision,
  updated_at
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW()
)
ON CONFLICT (request_id)
DO UPDATE SET
  provider_api_key_id = EXCLUDED.provider_api_key_id,
  request_count = EXCLUDED.request_count,
  success_count = EXCLUDED.success_count,
  error_count = EXCLUDED.error_count,
  total_tokens = EXCLUDED.total_tokens,
  total_cost_usd = EXCLUDED.total_cost_usd,
  total_response_time_ms = EXCLUDED.total_response_time_ms,
  last_used_at_unix_secs = EXCLUDED.last_used_at_unix_secs,
  usage_created_at_unix_secs = EXCLUDED.usage_created_at_unix_secs,
  window_request_count = EXCLUDED.window_request_count,
  window_total_tokens = EXCLUDED.window_total_tokens,
  window_total_cost_usd = EXCLUDED.window_total_cost_usd,
  revision = EXCLUDED.revision,
  updated_at = NOW()
"#;

#[derive(Debug)]
struct StoredContribution {
    contribution: Option<ProviderApiKeyUsageContribution>,
    revision: i64,
}

pub(crate) async fn sync_provider_api_key_usage_contribution_for_request_in_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    request_id: &str,
) -> Result<(), DataLayerError> {
    let Some(usage) = find_usage_by_request_id_in_tx(tx, request_id).await? else {
        return Ok(());
    };
    sync_provider_api_key_usage_contribution_in_tx(tx, &usage).await
}

pub(crate) async fn backfill_provider_api_key_usage_contribution_for_request_in_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    request_id: &str,
) -> Result<(), DataLayerError> {
    let Some(usage) = find_usage_by_request_id_in_tx(tx, request_id).await? else {
        return Ok(());
    };
    sync_provider_api_key_usage_contribution_internal_in_tx(tx, &usage, false).await
}

pub(crate) async fn sync_provider_api_key_usage_contribution_in_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    usage: &StoredRequestUsageAudit,
) -> Result<(), DataLayerError> {
    sync_provider_api_key_usage_contribution_internal_in_tx(tx, usage, true).await
}

async fn sync_provider_api_key_usage_contribution_internal_in_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    usage: &StoredRequestUsageAudit,
    enqueue_projection_deltas: bool,
) -> Result<(), DataLayerError> {
    let desired = provider_api_key_usage_contribution(usage).map(normalize_contribution);
    let previous = find_contribution_in_tx(tx, &usage.request_id).await?;
    let before = previous
        .as_ref()
        .and_then(|state| state.contribution.as_ref());

    if before == desired.as_ref() {
        return Ok(());
    }
    if previous.is_none() && desired.is_none() {
        return Ok(());
    }

    if enqueue_projection_deltas {
        lock_provider_api_key_usage_projection_keys_in_tx(
            tx,
            before
                .map(|contribution| contribution.key_id.as_str())
                .into_iter()
                .chain(
                    desired
                        .as_ref()
                        .map(|contribution| contribution.key_id.as_str()),
                ),
        )
        .await?;
    }

    let revision = previous
        .as_ref()
        .map(|state| state.revision.saturating_add(1))
        .unwrap_or(1);
    upsert_contribution_in_tx(tx, &usage.request_id, desired.as_ref(), revision).await?;

    if !enqueue_projection_deltas {
        return Ok(());
    }

    match (before, desired.as_ref()) {
        (Some(before), Some(after)) if before.key_id == after.key_id => {
            let delta = ProviderApiKeyUsageDelta::between(before, after);
            enqueue_provider_delta_in_tx(
                tx,
                &usage.request_id,
                before.key_id.as_str(),
                revision,
                &delta,
            )
            .await?;
        }
        _ => {
            if let Some(before) = before {
                let delta = ProviderApiKeyUsageDelta::removal(before);
                enqueue_provider_delta_in_tx(
                    tx,
                    &usage.request_id,
                    before.key_id.as_str(),
                    revision,
                    &delta,
                )
                .await?;
            }
            if let Some(after) = desired.as_ref() {
                let delta = ProviderApiKeyUsageDelta::addition(after);
                enqueue_provider_delta_in_tx(
                    tx,
                    &usage.request_id,
                    after.key_id.as_str(),
                    revision,
                    &delta,
                )
                .await?;
            }
        }
    }

    Ok(())
}

pub(crate) async fn lock_provider_api_key_usage_projection_keys_in_tx<'a>(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    key_ids: impl IntoIterator<Item = &'a str>,
) -> Result<(), DataLayerError> {
    let key_ids = key_ids
        .into_iter()
        .map(str::trim)
        .filter(|key_id| !key_id.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    for key_id in key_ids {
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1)::BIGINT)")
            .bind(format!("aether:provider-usage-projection:{key_id}"))
            .execute(&mut **tx)
            .await
            .map_postgres_err()?;
    }
    Ok(())
}

async fn find_contribution_in_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    request_id: &str,
) -> Result<Option<StoredContribution>, DataLayerError> {
    let row = sqlx::query(FIND_CONTRIBUTION_SQL)
        .bind(request_id)
        .fetch_optional(&mut **tx)
        .await
        .map_postgres_err()?;
    let Some(row) = row else {
        return Ok(None);
    };

    let revision = row.try_get::<i64, _>("revision").map_postgres_err()?;
    let provider_api_key_id = row
        .try_get::<Option<String>, _>("provider_api_key_id")
        .map_postgres_err()?;
    let contribution = match provider_api_key_id {
        Some(key_id) => Some(ProviderApiKeyUsageContribution {
            key_id,
            request_count: row.try_get("request_count").map_postgres_err()?,
            success_count: row.try_get("success_count").map_postgres_err()?,
            error_count: row.try_get("error_count").map_postgres_err()?,
            total_tokens: row.try_get("total_tokens").map_postgres_err()?,
            total_cost_usd: row.try_get("total_cost_usd").map_postgres_err()?,
            total_response_time_ms: row.try_get("total_response_time_ms").map_postgres_err()?,
            last_used_at_unix_secs: optional_i64_to_u64(
                "last_used_at_unix_secs",
                row.try_get("last_used_at_unix_secs").map_postgres_err()?,
            )?,
            usage_created_at_unix_secs: optional_i64_to_u64(
                "usage_created_at_unix_secs",
                row.try_get("usage_created_at_unix_secs")
                    .map_postgres_err()?,
            )?,
            window_request_count: row.try_get("window_request_count").map_postgres_err()?,
            window_total_tokens: row.try_get("window_total_tokens").map_postgres_err()?,
            window_total_cost_usd: row.try_get("window_total_cost_usd").map_postgres_err()?,
        }),
        None => None,
    };

    Ok(Some(StoredContribution {
        contribution,
        revision,
    }))
}

async fn upsert_contribution_in_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    request_id: &str,
    contribution: Option<&ProviderApiKeyUsageContribution>,
    revision: i64,
) -> Result<(), DataLayerError> {
    let empty = ProviderApiKeyUsageContribution::default();
    let contribution = contribution.unwrap_or(&empty);
    let key_id = contribution
        .key_id
        .trim()
        .is_empty()
        .then_some(None)
        .unwrap_or(Some(contribution.key_id.as_str()));

    sqlx::query(UPSERT_CONTRIBUTION_SQL)
        .bind(request_id)
        .bind(key_id)
        .bind(contribution.request_count)
        .bind(contribution.success_count)
        .bind(contribution.error_count)
        .bind(contribution.total_tokens)
        .bind(contribution.total_cost_usd)
        .bind(contribution.total_response_time_ms)
        .bind(optional_u64_to_i64(
            "last_used_at_unix_secs",
            contribution.last_used_at_unix_secs,
        )?)
        .bind(optional_u64_to_i64(
            "usage_created_at_unix_secs",
            contribution.usage_created_at_unix_secs,
        )?)
        .bind(contribution.window_request_count)
        .bind(contribution.window_total_tokens)
        .bind(contribution.window_total_cost_usd)
        .bind(revision)
        .execute(&mut **tx)
        .await
        .map_postgres_err()?;
    Ok(())
}

async fn enqueue_provider_delta_in_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    request_id: &str,
    key_id: &str,
    revision: i64,
    delta: &ProviderApiKeyUsageDelta,
) -> Result<(), DataLayerError> {
    if key_id.trim().is_empty() || delta.is_noop() {
        return Ok(());
    }

    insert_usage_counter_delta_with_id_in_tx(
        tx,
        contribution_event_id(request_id, key_id, revision, "main"),
        UsageCounterDeltaInsert {
            request_id,
            kind: USAGE_COUNTER_KIND_PROVIDER_API_KEY,
            target_id: key_id,
            request_count_delta: delta.request_count,
            total_requests_delta: 0,
            success_count_delta: delta.success_count,
            error_count_delta: delta.error_count,
            dns_failures_delta: 0,
            stream_errors_delta: 0,
            total_tokens_delta: delta.total_tokens,
            total_cost_usd_delta: round_money(delta.total_cost_usd),
            total_response_time_ms_delta: delta.total_response_time_ms,
            window_request_count_delta: 0,
            window_total_tokens_delta: 0,
            window_total_cost_usd_delta: 0.0,
            last_used_at_unix_secs: None,
            last_used_ip: None,
            candidate_last_used_at_unix_secs: delta.candidate_last_used_at_unix_secs,
            removed_last_used_at_unix_secs: delta.removed_last_used_at_unix_secs,
            usage_created_at_unix_secs: None,
        },
    )
    .await?;

    if delta.window_request_count == 0
        && delta.window_total_tokens == 0
        && delta.window_total_cost_usd == 0.0
    {
        return Ok(());
    }

    insert_usage_counter_delta_with_id_in_tx(
        tx,
        contribution_event_id(request_id, key_id, revision, "window"),
        UsageCounterDeltaInsert {
            request_id,
            kind: USAGE_COUNTER_KIND_PROVIDER_API_KEY_WINDOW,
            target_id: key_id,
            request_count_delta: 0,
            total_requests_delta: 0,
            success_count_delta: 0,
            error_count_delta: 0,
            dns_failures_delta: 0,
            stream_errors_delta: 0,
            total_tokens_delta: 0,
            total_cost_usd_delta: 0.0,
            total_response_time_ms_delta: 0,
            window_request_count_delta: delta.window_request_count,
            window_total_tokens_delta: delta.window_total_tokens,
            window_total_cost_usd_delta: round_money(delta.window_total_cost_usd),
            last_used_at_unix_secs: None,
            last_used_ip: None,
            candidate_last_used_at_unix_secs: None,
            removed_last_used_at_unix_secs: None,
            usage_created_at_unix_secs: delta.usage_created_at_unix_secs,
        },
    )
    .await
}

fn contribution_event_id(request_id: &str, key_id: &str, revision: i64, kind: &str) -> String {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("aether:provider-usage:{request_id}:{key_id}:{revision}:{kind}").as_bytes(),
    )
    .to_string()
}

fn normalize_contribution(
    mut contribution: ProviderApiKeyUsageContribution,
) -> ProviderApiKeyUsageContribution {
    contribution.total_cost_usd = normalize_money(contribution.total_cost_usd);
    contribution.window_total_cost_usd = normalize_money(contribution.window_total_cost_usd);
    contribution
}

fn normalize_money(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    (value.max(0.0) * 100_000_000.0).round() / 100_000_000.0
}

fn round_money(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    (value * 100_000_000.0).round() / 100_000_000.0
}

fn optional_u64_to_i64(
    field_name: &str,
    value: Option<u64>,
) -> Result<Option<i64>, DataLayerError> {
    value
        .map(|value| {
            i64::try_from(value).map_err(|_| {
                DataLayerError::UnexpectedValue(format!("{field_name} exceeds i64: {value}"))
            })
        })
        .transpose()
}

fn optional_i64_to_u64(
    field_name: &str,
    value: Option<i64>,
) -> Result<Option<u64>, DataLayerError> {
    value
        .map(|value| {
            u64::try_from(value).map_err(|_| {
                DataLayerError::UnexpectedValue(format!("{field_name} is negative: {value}"))
            })
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::{contribution_event_id, normalize_money};

    #[test]
    fn contribution_event_id_is_stable_and_projection_specific() {
        let first = contribution_event_id("request-1", "key-1", 2, "main");
        let repeated = contribution_event_id("request-1", "key-1", 2, "main");
        let window = contribution_event_id("request-1", "key-1", 2, "window");

        assert_eq!(first, repeated);
        assert_ne!(first, window);
    }

    #[test]
    fn normalize_money_matches_database_precision() {
        assert_eq!(normalize_money(0.123456789), 0.12345679);
        assert_eq!(normalize_money(-1.0), 0.0);
        assert_eq!(normalize_money(f64::NAN), 0.0);
    }

    #[test]
    fn normalize_money_keeps_two_micro_costs_additive_at_database_precision() {
        let first = normalize_money(0.000000006);
        let second = normalize_money(0.000000006);

        assert_eq!(first, 0.00000001);
        assert_eq!(second, 0.00000001);
        assert_eq!(first + second, 0.00000002);
    }
}
