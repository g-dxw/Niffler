use async_trait::async_trait;
use sqlx::{sqlite::SqliteRow, QueryBuilder, Row, Sqlite};

use super::{
    bounded_limit, bounded_offset, i64_from_u64, json_from_string, json_to_string,
    CreateNifflerAccountRiskEventRecord, CreateNifflerBillingReservationDryRunRecord,
    CreateNifflerBillingReservationRecord, CreateNifflerErrorReturnSettingRecord,
    CreateNifflerProductPlanRecord, CreateNifflerReferralRewardLedgerRecord,
    CreateNifflerRouteAttemptRecord, CreateNifflerSettlementSnapshotRecord,
    CreateNifflerUpstreamAccountRecord, CreateNifflerUpstreamServiceRecord,
    FinalizeNifflerBillingReservationRecord, NifflerAccountModelCapabilityListQuery,
    NifflerAccountProtectionAction, NifflerAccountStatus, NifflerApiKeyProductPlanBindingListQuery,
    NifflerBillingReservationDryRunListQuery, NifflerBillingReservationListQuery,
    NifflerBillingReservationStatus, NifflerConsistencyCheckListQuery, NifflerCoreReadRepository,
    NifflerCoreWriteRepository, NifflerErrorResponseScope, NifflerErrorReturnSettingListQuery,
    NifflerPauseDuration, NifflerProductPlanListQuery, NifflerProductPlanModelListQuery,
    NifflerProtocolKind, NifflerReferralRewardLedgerListQuery, NifflerReferralRewardLedgerStatus,
    NifflerRouteAttemptListQuery, NifflerRuntimeAccountModelAccessListQuery,
    NifflerRuntimeRolloutSettingListQuery, NifflerRuntimeRolloutTargetScope,
    NifflerServiceCapabilityKind, NifflerSettlementSnapshotListQuery,
    NifflerStabilityObservationListQuery, NifflerUpstreamAccountListQuery,
    NifflerUpstreamErrorHandlingStep, NifflerUpstreamServiceCapabilityListQuery,
    NifflerUpstreamServiceListQuery, NifflerUserResponseMode, StoredNifflerAccountModelCapability,
    StoredNifflerAccountModelCapabilityListPage, StoredNifflerAccountRiskEvent,
    StoredNifflerApiKeyProductPlanBinding, StoredNifflerApiKeyProductPlanBindingListPage,
    StoredNifflerBillingReservation, StoredNifflerBillingReservationDryRun,
    StoredNifflerBillingReservationDryRunListPage, StoredNifflerBillingReservationListPage,
    StoredNifflerConsistencyCheckItem, StoredNifflerConsistencyCheckListPage,
    StoredNifflerErrorReturnSetting, StoredNifflerErrorReturnSettingListPage,
    StoredNifflerProductPlan, StoredNifflerProductPlanListPage, StoredNifflerProductPlanModel,
    StoredNifflerProductPlanModelListPage, StoredNifflerReferralRewardLedger,
    StoredNifflerReferralRewardLedgerListPage, StoredNifflerRouteAttempt,
    StoredNifflerRouteAttemptListItem, StoredNifflerRouteAttemptListPage,
    StoredNifflerRuntimeAccountModelAccess, StoredNifflerRuntimeAccountModelAccessListPage,
    StoredNifflerRuntimeRolloutSetting, StoredNifflerRuntimeRolloutSettingListPage,
    StoredNifflerSettlementSnapshot, StoredNifflerSettlementSnapshotListItem,
    StoredNifflerSettlementSnapshotListPage, StoredNifflerStabilityObservation,
    StoredNifflerStabilityObservationListPage, StoredNifflerUpstreamAccount,
    StoredNifflerUpstreamAccountListPage, StoredNifflerUpstreamService,
    StoredNifflerUpstreamServiceCapability, StoredNifflerUpstreamServiceCapabilityListPage,
    StoredNifflerUpstreamServiceListPage, UpsertNifflerApiKeyProductPlanBindingRecord,
    UpsertNifflerProductPlanModelRecord, UpsertNifflerRuntimeRolloutSettingRecord,
    UpsertNifflerStabilityObservationRecord, UpsertNifflerUpstreamServiceCapabilityRecord,
};
use crate::driver::sqlite::SqlitePool;
use crate::error::SqlResultExt;
use crate::DataLayerError;

#[derive(Debug, Clone)]
pub struct SqliteNifflerCoreRepository {
    pool: SqlitePool,
}

impl SqliteNifflerCoreRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    async fn reload_service(
        &self,
        service_id: &str,
    ) -> Result<StoredNifflerUpstreamService, DataLayerError> {
        self.find_upstream_service_by_id(service_id)
            .await?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler upstream service missing after write".into(),
                )
            })
    }

    async fn reload_account(
        &self,
        account_id: &str,
    ) -> Result<StoredNifflerUpstreamAccount, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, upstream_service_id, display_name, email, phone, auth_kind, status,
  cost_multiplier, priority, cooldown_until_unix_ms, last_tested_at_unix_ms,
  last_test_error, config, created_at_unix_ms, updated_at_unix_ms
FROM niffler_upstream_accounts
WHERE id = ?
LIMIT 1
"#,
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref()
            .map(map_account_row)
            .transpose()?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler upstream account missing after write".into(),
                )
            })
    }

    async fn reload_capability(
        &self,
        capability_id: &str,
    ) -> Result<StoredNifflerUpstreamServiceCapability, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, upstream_service_id, protocol_kind, capability_kind, is_enabled,
  config, created_at_unix_ms, updated_at_unix_ms
FROM niffler_upstream_service_capabilities
WHERE id = ?
LIMIT 1
"#,
        )
        .bind(capability_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref()
            .map(map_capability_row)
            .transpose()?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler upstream service capability missing after write".into(),
                )
            })
    }

    async fn reload_product_plan(
        &self,
        product_plan_id: &str,
    ) -> Result<StoredNifflerProductPlan, DataLayerError> {
        self.find_product_plan_by_id(product_plan_id)
            .await?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue("niffler product plan missing after write".into())
            })
    }

    async fn reload_product_plan_model(
        &self,
        product_plan_id: &str,
        model_name: &str,
    ) -> Result<StoredNifflerProductPlanModel, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, product_plan_id, model_name, is_enabled, sales_multiplier_override,
  created_at_unix_ms, updated_at_unix_ms
FROM niffler_product_plan_models
WHERE product_plan_id = ? AND model_name = ?
LIMIT 1
"#,
        )
        .bind(product_plan_id)
        .bind(model_name)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref()
            .map(map_product_plan_model_row)
            .transpose()?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler product plan model missing after write".into(),
                )
            })
    }

    async fn reload_api_key_product_plan_binding(
        &self,
        api_key_id: &str,
    ) -> Result<StoredNifflerApiKeyProductPlanBinding, DataLayerError> {
        self.find_api_key_product_plan_binding_by_api_key_id(api_key_id)
            .await?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler api key product plan binding missing after write".into(),
                )
            })
    }

    async fn reload_runtime_rollout_setting(
        &self,
        target_scope: NifflerRuntimeRolloutTargetScope,
        target_id: &str,
    ) -> Result<StoredNifflerRuntimeRolloutSetting, DataLayerError> {
        self.find_runtime_rollout_setting(target_scope, target_id)
            .await?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler runtime rollout setting missing after write".into(),
                )
            })
    }

    async fn reload_error_return_setting(
        &self,
        setting_id: &str,
    ) -> Result<StoredNifflerErrorReturnSetting, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, scope, upstream_service_id, match_status_code, match_text, handling_step,
  response_mode, user_message, account_protection_action, pause_duration,
  is_active, created_at_unix_ms, updated_at_unix_ms
FROM niffler_error_return_settings
WHERE id = ?
LIMIT 1
"#,
        )
        .bind(setting_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref()
            .map(map_error_return_setting_row)
            .transpose()?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler error return setting missing after write".into(),
                )
            })
    }

    async fn reload_account_risk_event(
        &self,
        event_id: &str,
    ) -> Result<StoredNifflerAccountRiskEvent, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, upstream_service_id, upstream_account_id, request_id, user_id, api_key_id,
  model_name, rule_id, matched_text, upstream_status_code, action, created_at_unix_ms
FROM niffler_account_risk_events
WHERE id = ?
LIMIT 1
"#,
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref()
            .map(map_account_risk_event_row)
            .transpose()?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler account risk event missing after write".into(),
                )
            })
    }

    async fn reload_referral_reward_ledger_by_order_id(
        &self,
        order_id: &str,
    ) -> Result<StoredNifflerReferralRewardLedger, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, order_id, idempotency_key, inviter_user_id, invitee_user_id, rule_id,
  reward_amount_usd, rule_snapshot, status, failure_reason, retry_count,
  paid_at_unix_ms, cancelled_at_unix_ms, created_at_unix_ms, updated_at_unix_ms
FROM niffler_referral_reward_ledger
WHERE order_id = ?
LIMIT 1
"#,
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref()
            .map(map_referral_reward_ledger_row)
            .transpose()?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler referral reward ledger missing after write".into(),
                )
            })
    }

    async fn reload_settlement_snapshot_by_request_id(
        &self,
        request_id: &str,
    ) -> Result<StoredNifflerSettlementSnapshot, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, request_id, user_id, api_key_id, product_plan_id, upstream_service_id,
  upstream_account_id, requested_model_name, upstream_execution_model_name,
  image_tool_model_name, pricing_snapshot, wallet_charge_usd,
  entitlement_charge_usd, upstream_cost_usd, gross_margin_usd,
  created_at_unix_ms, finalized_at_unix_ms
FROM niffler_settlement_snapshots
WHERE request_id = ?
LIMIT 1
"#,
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref()
            .map(map_settlement_snapshot_row)
            .transpose()?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler settlement snapshot missing after write".into(),
                )
            })
    }

    async fn reload_billing_reservation_dry_run_by_request_id(
        &self,
        request_id: &str,
    ) -> Result<StoredNifflerBillingReservationDryRun, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, request_id, user_id, api_key_id, product_plan_id, requested_model_name,
  estimated_reservation_usd, legacy_final_charge_usd, difference_usd,
  estimation_source, status, created_at_unix_ms, finalized_at_unix_ms
FROM niffler_billing_reservation_dry_runs
WHERE request_id = ?
LIMIT 1
"#,
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref()
            .map(map_billing_reservation_dry_run_row)
            .transpose()?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler billing reservation dry run missing after write".into(),
                )
            })
    }

    async fn reload_billing_reservation_by_request_id(
        &self,
        request_id: &str,
    ) -> Result<StoredNifflerBillingReservation, DataLayerError> {
        self.find_billing_reservation_by_request_id(request_id)
            .await?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler billing reservation missing after write".into(),
                )
            })
    }

    async fn find_billing_reservation_by_request_id(
        &self,
        request_id: &str,
    ) -> Result<Option<StoredNifflerBillingReservation>, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, request_id, user_id, api_key_id, product_plan_id, status,
  reserved_total_usd, wallet_reserved_usd, entitlement_reserved_usd,
  reserved_at_unix_ms, expires_at_unix_ms, finalized_at_unix_ms,
  settlement_snapshot_id, release_reason, idempotency_key
FROM niffler_billing_reservations
WHERE request_id = ?
LIMIT 1
"#,
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref().map(map_billing_reservation_row).transpose()
    }
}

#[async_trait]
impl NifflerCoreReadRepository for SqliteNifflerCoreRepository {
    async fn list_upstream_services(
        &self,
        query: &NifflerUpstreamServiceListQuery,
    ) -> Result<StoredNifflerUpstreamServiceListPage, DataLayerError> {
        let total = build_service_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_service_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_service_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerUpstreamServiceListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn find_upstream_service_by_id(
        &self,
        upstream_service_id: &str,
    ) -> Result<Option<StoredNifflerUpstreamService>, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, display_name, service_kind, default_api_format, base_url,
  cost_multiplier, is_active, config, created_at_unix_ms, updated_at_unix_ms
FROM niffler_upstream_services
WHERE id = ?
LIMIT 1
"#,
        )
        .bind(upstream_service_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref().map(map_service_row).transpose()
    }

    async fn list_upstream_service_capabilities(
        &self,
        query: &NifflerUpstreamServiceCapabilityListQuery,
    ) -> Result<StoredNifflerUpstreamServiceCapabilityListPage, DataLayerError> {
        let rows = sqlx::query(
            r#"
SELECT
  id, upstream_service_id, protocol_kind, capability_kind, is_enabled,
  config, created_at_unix_ms, updated_at_unix_ms
FROM niffler_upstream_service_capabilities
WHERE upstream_service_id = ?
ORDER BY protocol_kind ASC, capability_kind ASC
"#,
        )
        .bind(query.upstream_service_id.clone())
        .fetch_all(&self.pool)
        .await
        .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_capability_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerUpstreamServiceCapabilityListPage {
            total: items.len(),
            items,
        })
    }

    async fn list_upstream_accounts(
        &self,
        query: &NifflerUpstreamAccountListQuery,
    ) -> Result<StoredNifflerUpstreamAccountListPage, DataLayerError> {
        let total = build_account_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_account_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_account_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerUpstreamAccountListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn find_upstream_account_by_id(
        &self,
        upstream_account_id: &str,
    ) -> Result<Option<StoredNifflerUpstreamAccount>, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, upstream_service_id, display_name, email, phone, auth_kind, status,
  cost_multiplier, priority, cooldown_until_unix_ms, last_tested_at_unix_ms,
  last_test_error, config, created_at_unix_ms, updated_at_unix_ms
FROM niffler_upstream_accounts
WHERE id = ?
LIMIT 1
"#,
        )
        .bind(upstream_account_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref().map(map_account_row).transpose()
    }

    async fn list_account_model_capabilities(
        &self,
        query: &NifflerAccountModelCapabilityListQuery,
    ) -> Result<StoredNifflerAccountModelCapabilityListPage, DataLayerError> {
        let total = build_account_model_capability_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_account_model_capability_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_account_model_capability_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerAccountModelCapabilityListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn list_runtime_account_model_access(
        &self,
        query: &NifflerRuntimeAccountModelAccessListQuery,
    ) -> Result<StoredNifflerRuntimeAccountModelAccessListPage, DataLayerError> {
        let total = build_runtime_account_model_access_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_runtime_account_model_access_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_runtime_account_model_access_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerRuntimeAccountModelAccessListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn list_product_plans(
        &self,
        query: &NifflerProductPlanListQuery,
    ) -> Result<StoredNifflerProductPlanListPage, DataLayerError> {
        let total = build_product_plan_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_product_plan_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_product_plan_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerProductPlanListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn find_product_plan_by_id(
        &self,
        product_plan_id: &str,
    ) -> Result<Option<StoredNifflerProductPlan>, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, display_name, is_public, is_active, sales_multiplier, description,
  created_at_unix_ms, updated_at_unix_ms
FROM niffler_product_plans
WHERE id = ?
LIMIT 1
"#,
        )
        .bind(product_plan_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref().map(map_product_plan_row).transpose()
    }

    async fn list_product_plan_models(
        &self,
        query: &NifflerProductPlanModelListQuery,
    ) -> Result<StoredNifflerProductPlanModelListPage, DataLayerError> {
        let total = build_product_plan_model_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_product_plan_model_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_product_plan_model_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerProductPlanModelListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn list_api_key_product_plan_bindings(
        &self,
        query: &NifflerApiKeyProductPlanBindingListQuery,
    ) -> Result<StoredNifflerApiKeyProductPlanBindingListPage, DataLayerError> {
        let total = build_api_key_product_plan_binding_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_api_key_product_plan_binding_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_api_key_product_plan_binding_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerApiKeyProductPlanBindingListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn find_api_key_product_plan_binding_by_api_key_id(
        &self,
        api_key_id: &str,
    ) -> Result<Option<StoredNifflerApiKeyProductPlanBinding>, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, api_key_id, product_plan_id, config, created_at_unix_ms, updated_at_unix_ms
FROM niffler_api_key_product_plan_bindings
WHERE api_key_id = ?
LIMIT 1
"#,
        )
        .bind(api_key_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref()
            .map(map_api_key_product_plan_binding_row)
            .transpose()
    }

    async fn list_runtime_rollout_settings(
        &self,
        query: &NifflerRuntimeRolloutSettingListQuery,
    ) -> Result<StoredNifflerRuntimeRolloutSettingListPage, DataLayerError> {
        let total = build_runtime_rollout_setting_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_runtime_rollout_setting_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_runtime_rollout_setting_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerRuntimeRolloutSettingListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn find_runtime_rollout_setting(
        &self,
        target_scope: NifflerRuntimeRolloutTargetScope,
        target_id: &str,
    ) -> Result<Option<StoredNifflerRuntimeRolloutSetting>, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, target_scope, target_id, enable_new_routing, enable_settlement_snapshot,
  enable_error_return_rules, enable_billing_reservation, enable_referral_ledger,
  is_active, config, created_at_unix_ms, updated_at_unix_ms
FROM niffler_runtime_rollout_settings
WHERE target_scope = ? AND target_id = ?
LIMIT 1
"#,
        )
        .bind(target_scope.as_str())
        .bind(target_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref()
            .map(map_runtime_rollout_setting_row)
            .transpose()
    }

    async fn list_error_return_settings(
        &self,
        query: &NifflerErrorReturnSettingListQuery,
    ) -> Result<StoredNifflerErrorReturnSettingListPage, DataLayerError> {
        let total = build_error_return_setting_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_error_return_setting_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_error_return_setting_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerErrorReturnSettingListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn list_settlement_snapshots(
        &self,
        query: &NifflerSettlementSnapshotListQuery,
    ) -> Result<StoredNifflerSettlementSnapshotListPage, DataLayerError> {
        let total = build_settlement_snapshot_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_settlement_snapshot_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_settlement_snapshot_list_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerSettlementSnapshotListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn list_billing_reservations(
        &self,
        query: &NifflerBillingReservationListQuery,
    ) -> Result<StoredNifflerBillingReservationListPage, DataLayerError> {
        let total = build_billing_reservation_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_billing_reservation_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_billing_reservation_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerBillingReservationListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn sum_active_billing_reservation_wallet_usd(
        &self,
        user_id: Option<&str>,
        api_key_id: Option<&str>,
        now_unix_ms: u64,
    ) -> Result<f64, DataLayerError> {
        let now_unix_ms = i64_from_u64(now_unix_ms, "now_unix_ms")?;
        let user_id = user_id.map(str::trim).filter(|value| !value.is_empty());
        let api_key_id = api_key_id.map(str::trim).filter(|value| !value.is_empty());
        let amount = match (user_id, api_key_id) {
            (Some(user_id), None) => sqlx::query_scalar::<_, Option<f64>>(
                r#"
SELECT COALESCE(SUM(wallet_reserved_usd), 0.0)
FROM niffler_billing_reservations
WHERE status = 'active' AND expires_at_unix_ms > ? AND user_id = ?
"#,
            )
            .bind(now_unix_ms)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?,
            (None, Some(api_key_id)) => sqlx::query_scalar::<_, Option<f64>>(
                r#"
SELECT COALESCE(SUM(wallet_reserved_usd), 0.0)
FROM niffler_billing_reservations
WHERE status = 'active' AND expires_at_unix_ms > ? AND api_key_id = ?
"#,
            )
            .bind(now_unix_ms)
            .bind(api_key_id)
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?,
            (Some(user_id), Some(api_key_id)) => sqlx::query_scalar::<_, Option<f64>>(
                r#"
SELECT COALESCE(SUM(wallet_reserved_usd), 0.0)
FROM niffler_billing_reservations
WHERE status = 'active' AND expires_at_unix_ms > ? AND user_id = ? AND api_key_id = ?
"#,
            )
            .bind(now_unix_ms)
            .bind(user_id)
            .bind(api_key_id)
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?,
            (None, None) => Some(0.0),
        };
        Ok(amount.unwrap_or_default().max(0.0))
    }

    async fn list_billing_reservation_dry_runs(
        &self,
        query: &NifflerBillingReservationDryRunListQuery,
    ) -> Result<StoredNifflerBillingReservationDryRunListPage, DataLayerError> {
        let total = build_billing_reservation_dry_run_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_billing_reservation_dry_run_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_billing_reservation_dry_run_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerBillingReservationDryRunListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn list_referral_reward_ledger(
        &self,
        query: &NifflerReferralRewardLedgerListQuery,
    ) -> Result<StoredNifflerReferralRewardLedgerListPage, DataLayerError> {
        let total = build_referral_reward_ledger_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_referral_reward_ledger_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_referral_reward_ledger_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerReferralRewardLedgerListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn list_route_attempts(
        &self,
        query: &NifflerRouteAttemptListQuery,
    ) -> Result<StoredNifflerRouteAttemptListPage, DataLayerError> {
        let total = build_route_attempt_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_route_attempt_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_route_attempt_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerRouteAttemptListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn list_consistency_checks(
        &self,
        query: &NifflerConsistencyCheckListQuery,
    ) -> Result<StoredNifflerConsistencyCheckListPage, DataLayerError> {
        let rows = build_consistency_check_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_consistency_check_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerConsistencyCheckListPage {
            total: query.offset.saturating_add(items.len()),
            items,
        })
    }

    async fn list_stability_observations(
        &self,
        query: &NifflerStabilityObservationListQuery,
    ) -> Result<StoredNifflerStabilityObservationListPage, DataLayerError> {
        let total = build_stability_observation_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_stability_observation_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_stability_observation_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerStabilityObservationListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }
}

#[async_trait]
impl NifflerCoreWriteRepository for SqliteNifflerCoreRepository {
    async fn create_upstream_service(
        &self,
        record: CreateNifflerUpstreamServiceRecord,
    ) -> Result<StoredNifflerUpstreamService, DataLayerError> {
        record.validate()?;
        let config = json_to_string(record.config.as_ref(), "niffler_upstream_services.config")?;
        sqlx::query(
            r#"
INSERT INTO niffler_upstream_services (
  id, display_name, service_kind, default_api_format, base_url,
  cost_multiplier, is_active, config, created_at_unix_ms, updated_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&record.id)
        .bind(&record.display_name)
        .bind(&record.service_kind)
        .bind(&record.default_api_format)
        .bind(&record.base_url)
        .bind(record.cost_multiplier)
        .bind(record.is_active)
        .bind(config)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_service(&record.id).await
    }

    async fn create_upstream_account(
        &self,
        record: CreateNifflerUpstreamAccountRecord,
    ) -> Result<StoredNifflerUpstreamAccount, DataLayerError> {
        record.validate()?;
        let config = json_to_string(record.config.as_ref(), "niffler_upstream_accounts.config")?;
        sqlx::query(
            r#"
INSERT INTO niffler_upstream_accounts (
  id, upstream_service_id, display_name, email, phone, auth_kind, status,
  cost_multiplier, priority, cooldown_until_unix_ms, last_tested_at_unix_ms,
  last_test_error, config, created_at_unix_ms, updated_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&record.id)
        .bind(&record.upstream_service_id)
        .bind(&record.display_name)
        .bind(&record.email)
        .bind(&record.phone)
        .bind(&record.auth_kind)
        .bind(record.status.as_str())
        .bind(record.cost_multiplier)
        .bind(record.priority)
        .bind(
            record
                .cooldown_until_unix_ms
                .map(|value| i64_from_u64(value, "cooldown_until_unix_ms"))
                .transpose()?,
        )
        .bind(
            record
                .last_tested_at_unix_ms
                .map(|value| i64_from_u64(value, "last_tested_at_unix_ms"))
                .transpose()?,
        )
        .bind(&record.last_test_error)
        .bind(config)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_account(&record.id).await
    }

    async fn upsert_upstream_service_capability(
        &self,
        record: UpsertNifflerUpstreamServiceCapabilityRecord,
    ) -> Result<StoredNifflerUpstreamServiceCapability, DataLayerError> {
        record.validate()?;
        let config = json_to_string(
            record.config.as_ref(),
            "niffler_upstream_service_capabilities.config",
        )?;
        sqlx::query(
            r#"
INSERT INTO niffler_upstream_service_capabilities (
  id, upstream_service_id, protocol_kind, capability_kind, is_enabled,
  config, created_at_unix_ms, updated_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(upstream_service_id, protocol_kind, capability_kind) DO UPDATE SET
  id = excluded.id,
  is_enabled = excluded.is_enabled,
  config = excluded.config,
  updated_at_unix_ms = excluded.updated_at_unix_ms
"#,
        )
        .bind(&record.id)
        .bind(&record.upstream_service_id)
        .bind(record.protocol_kind.as_str())
        .bind(record.capability_kind.as_str())
        .bind(record.is_enabled)
        .bind(config)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_capability(&record.id).await
    }

    async fn create_product_plan(
        &self,
        record: CreateNifflerProductPlanRecord,
    ) -> Result<StoredNifflerProductPlan, DataLayerError> {
        record.validate()?;
        sqlx::query(
            r#"
INSERT INTO niffler_product_plans (
  id, display_name, is_public, is_active, sales_multiplier, description,
  created_at_unix_ms, updated_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&record.id)
        .bind(&record.display_name)
        .bind(record.is_public)
        .bind(record.is_active)
        .bind(record.sales_multiplier)
        .bind(&record.description)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_product_plan(&record.id).await
    }

    async fn upsert_product_plan_model(
        &self,
        record: UpsertNifflerProductPlanModelRecord,
    ) -> Result<StoredNifflerProductPlanModel, DataLayerError> {
        record.validate()?;
        sqlx::query(
            r#"
INSERT INTO niffler_product_plan_models (
  id, product_plan_id, model_name, is_enabled, sales_multiplier_override,
  created_at_unix_ms, updated_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(product_plan_id, model_name) DO UPDATE SET
  is_enabled = excluded.is_enabled,
  sales_multiplier_override = excluded.sales_multiplier_override,
  updated_at_unix_ms = excluded.updated_at_unix_ms
"#,
        )
        .bind(&record.id)
        .bind(&record.product_plan_id)
        .bind(&record.model_name)
        .bind(record.is_enabled)
        .bind(record.sales_multiplier_override)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_product_plan_model(&record.product_plan_id, &record.model_name)
            .await
    }

    async fn upsert_api_key_product_plan_binding(
        &self,
        record: UpsertNifflerApiKeyProductPlanBindingRecord,
    ) -> Result<StoredNifflerApiKeyProductPlanBinding, DataLayerError> {
        record.validate()?;
        let config = json_to_string(
            record.config.as_ref(),
            "niffler_api_key_product_plan_bindings.config",
        )?;
        sqlx::query(
            r#"
INSERT INTO niffler_api_key_product_plan_bindings (
  id, api_key_id, product_plan_id, config, created_at_unix_ms, updated_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?)
ON CONFLICT(api_key_id) DO UPDATE SET
  product_plan_id = excluded.product_plan_id,
  config = excluded.config,
  updated_at_unix_ms = excluded.updated_at_unix_ms
"#,
        )
        .bind(&record.id)
        .bind(&record.api_key_id)
        .bind(&record.product_plan_id)
        .bind(config)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_api_key_product_plan_binding(&record.api_key_id)
            .await
    }

    async fn upsert_runtime_rollout_setting(
        &self,
        record: UpsertNifflerRuntimeRolloutSettingRecord,
    ) -> Result<StoredNifflerRuntimeRolloutSetting, DataLayerError> {
        record.validate()?;
        let config = json_to_string(
            record.config.as_ref(),
            "niffler_runtime_rollout_settings.config",
        )?;
        sqlx::query(
            r#"
INSERT INTO niffler_runtime_rollout_settings (
  id, target_scope, target_id, enable_new_routing, enable_settlement_snapshot,
  enable_error_return_rules, enable_billing_reservation, enable_referral_ledger,
  is_active, config, created_at_unix_ms, updated_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT (target_scope, target_id) DO UPDATE SET
  enable_new_routing = excluded.enable_new_routing,
  enable_settlement_snapshot = excluded.enable_settlement_snapshot,
  enable_error_return_rules = excluded.enable_error_return_rules,
  enable_billing_reservation = excluded.enable_billing_reservation,
  enable_referral_ledger = excluded.enable_referral_ledger,
  is_active = excluded.is_active,
  config = excluded.config,
  updated_at_unix_ms = excluded.updated_at_unix_ms
"#,
        )
        .bind(&record.id)
        .bind(record.target_scope.as_str())
        .bind(&record.target_id)
        .bind(record.enable_new_routing)
        .bind(record.enable_settlement_snapshot)
        .bind(record.enable_error_return_rules)
        .bind(record.enable_billing_reservation)
        .bind(record.enable_referral_ledger)
        .bind(record.is_active)
        .bind(config)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_runtime_rollout_setting(record.target_scope, &record.target_id)
            .await
    }

    async fn create_error_return_setting(
        &self,
        record: CreateNifflerErrorReturnSettingRecord,
    ) -> Result<StoredNifflerErrorReturnSetting, DataLayerError> {
        record.validate()?;
        sqlx::query(
            r#"
INSERT INTO niffler_error_return_settings (
  id, scope, upstream_service_id, match_status_code, match_text, handling_step,
  response_mode, user_message, account_protection_action, pause_duration,
  is_active, created_at_unix_ms, updated_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&record.id)
        .bind(record.scope.as_str())
        .bind(&record.upstream_service_id)
        .bind(record.match_status_code.map(i32::from))
        .bind(&record.match_text)
        .bind(record.handling_step.map(|value| value.as_str()))
        .bind(record.response_mode.as_str())
        .bind(&record.user_message)
        .bind(record.account_protection_action.as_str())
        .bind(record.pause_duration.map(|value| value.as_str()))
        .bind(record.is_active)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_error_return_setting(&record.id).await
    }

    async fn create_account_risk_event(
        &self,
        record: CreateNifflerAccountRiskEventRecord,
    ) -> Result<StoredNifflerAccountRiskEvent, DataLayerError> {
        record.validate()?;
        sqlx::query(
            r#"
INSERT INTO niffler_account_risk_events (
  id, upstream_service_id, upstream_account_id, request_id, user_id, api_key_id,
  model_name, rule_id, matched_text, upstream_status_code, action, created_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&record.id)
        .bind(&record.upstream_service_id)
        .bind(&record.upstream_account_id)
        .bind(&record.request_id)
        .bind(&record.user_id)
        .bind(&record.api_key_id)
        .bind(&record.model_name)
        .bind(&record.rule_id)
        .bind(&record.matched_text)
        .bind(record.upstream_status_code.map(i32::from))
        .bind(record.action.as_str())
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_account_risk_event(&record.id).await
    }

    async fn create_settlement_snapshot(
        &self,
        record: CreateNifflerSettlementSnapshotRecord,
    ) -> Result<StoredNifflerSettlementSnapshot, DataLayerError> {
        record.validate()?;
        let pricing_snapshot = json_to_string(
            Some(&record.pricing_snapshot),
            "niffler_settlement_snapshots.pricing_snapshot",
        )?
        .ok_or_else(|| {
            DataLayerError::InvalidInput(
                "niffler_settlement_snapshots.pricing_snapshot is required".to_string(),
            )
        })?;
        let finalized_at_unix_ms = record
            .finalized_at_unix_ms
            .map(|value| i64_from_u64(value, "finalized_at_unix_ms"))
            .transpose()?;
        sqlx::query(
            r#"
INSERT INTO niffler_settlement_snapshots (
  id, request_id, user_id, api_key_id, product_plan_id, upstream_service_id,
  upstream_account_id, requested_model_name, upstream_execution_model_name,
  image_tool_model_name, pricing_snapshot, wallet_charge_usd,
  entitlement_charge_usd, upstream_cost_usd, gross_margin_usd,
  created_at_unix_ms, finalized_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT (request_id) DO UPDATE SET
  user_id = excluded.user_id,
  api_key_id = excluded.api_key_id,
  product_plan_id = excluded.product_plan_id,
  upstream_service_id = excluded.upstream_service_id,
  upstream_account_id = excluded.upstream_account_id,
  requested_model_name = excluded.requested_model_name,
  upstream_execution_model_name = excluded.upstream_execution_model_name,
  image_tool_model_name = excluded.image_tool_model_name,
  pricing_snapshot = excluded.pricing_snapshot,
  wallet_charge_usd = excluded.wallet_charge_usd,
  entitlement_charge_usd = excluded.entitlement_charge_usd,
  upstream_cost_usd = excluded.upstream_cost_usd,
  gross_margin_usd = excluded.gross_margin_usd,
  created_at_unix_ms = excluded.created_at_unix_ms,
  finalized_at_unix_ms = excluded.finalized_at_unix_ms
"#,
        )
        .bind(&record.id)
        .bind(&record.request_id)
        .bind(&record.user_id)
        .bind(&record.api_key_id)
        .bind(&record.product_plan_id)
        .bind(&record.upstream_service_id)
        .bind(&record.upstream_account_id)
        .bind(&record.requested_model_name)
        .bind(&record.upstream_execution_model_name)
        .bind(&record.image_tool_model_name)
        .bind(&pricing_snapshot)
        .bind(record.wallet_charge_usd)
        .bind(record.entitlement_charge_usd)
        .bind(record.upstream_cost_usd)
        .bind(record.gross_margin_usd)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(finalized_at_unix_ms)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_settlement_snapshot_by_request_id(&record.request_id)
            .await
    }

    async fn create_billing_reservation(
        &self,
        record: CreateNifflerBillingReservationRecord,
    ) -> Result<StoredNifflerBillingReservation, DataLayerError> {
        record.validate()?;
        let reserved_at_unix_ms = i64_from_u64(record.reserved_at_unix_ms, "reserved_at_unix_ms")?;
        let expires_at_unix_ms = i64_from_u64(record.expires_at_unix_ms, "expires_at_unix_ms")?;
        let mut tx = self.pool.begin().await.map_sql_err()?;
        let insert_result = sqlx::query(
            r#"
INSERT OR IGNORE INTO niffler_billing_reservations (
  id, request_id, user_id, api_key_id, product_plan_id, status,
  reserved_total_usd, wallet_reserved_usd, entitlement_reserved_usd,
  reserved_at_unix_ms, expires_at_unix_ms, finalized_at_unix_ms,
  settlement_snapshot_id, release_reason, idempotency_key
)
VALUES (?, ?, ?, ?, ?, 'active', ?, ?, ?, ?, ?, NULL, NULL, NULL, ?)
"#,
        )
        .bind(&record.id)
        .bind(&record.request_id)
        .bind(&record.user_id)
        .bind(&record.api_key_id)
        .bind(&record.product_plan_id)
        .bind(record.reserved_total_usd)
        .bind(record.wallet_reserved_usd)
        .bind(record.entitlement_reserved_usd)
        .bind(reserved_at_unix_ms)
        .bind(expires_at_unix_ms)
        .bind(&record.idempotency_key)
        .execute(&mut *tx)
        .await
        .map_sql_err()?;
        if insert_result.rows_affected() > 0 {
            sqlx::query(
                r#"
INSERT OR IGNORE INTO niffler_billing_reservation_events (
  id, reservation_id, event_kind, amount_usd, reason, idempotency_key, actor_id,
  created_at_unix_ms
)
VALUES (?, ?, 'reserved', ?, NULL, ?, ?, ?)
"#,
            )
            .bind(&record.event_id)
            .bind(&record.id)
            .bind(record.reserved_total_usd)
            .bind(&record.event_idempotency_key)
            .bind(&record.actor_id)
            .bind(reserved_at_unix_ms)
            .execute(&mut *tx)
            .await
            .map_sql_err()?;
        }
        tx.commit().await.map_sql_err()?;
        self.reload_billing_reservation_by_request_id(&record.request_id)
            .await
    }

    async fn finalize_billing_reservation_by_request_id(
        &self,
        record: FinalizeNifflerBillingReservationRecord,
    ) -> Result<Option<StoredNifflerBillingReservation>, DataLayerError> {
        record.validate()?;
        let finalized_at_unix_ms =
            i64_from_u64(record.finalized_at_unix_ms, "finalized_at_unix_ms")?;
        let mut tx = self.pool.begin().await.map_sql_err()?;
        let row = sqlx::query(
            r#"
SELECT id, status, reserved_total_usd
FROM niffler_billing_reservations
WHERE request_id = ?
LIMIT 1
"#,
        )
        .bind(&record.request_id)
        .fetch_optional(&mut *tx)
        .await
        .map_sql_err()?;
        if let Some(row) = row.as_ref() {
            let reservation_id: String = row.try_get("id").map_sql_err()?;
            let status: String = row.try_get("status").map_sql_err()?;
            if status == "active" {
                let amount_usd: f64 = row.try_get("reserved_total_usd").map_sql_err()?;
                sqlx::query(
                    r#"
UPDATE niffler_billing_reservations
SET status = ?,
    finalized_at_unix_ms = ?,
    settlement_snapshot_id = ?,
    release_reason = ?
WHERE request_id = ? AND status = 'active'
"#,
                )
                .bind(record.status.as_str())
                .bind(finalized_at_unix_ms)
                .bind(&record.settlement_snapshot_id)
                .bind(&record.release_reason)
                .bind(&record.request_id)
                .execute(&mut *tx)
                .await
                .map_sql_err()?;
                sqlx::query(
                    r#"
INSERT OR IGNORE INTO niffler_billing_reservation_events (
  id, reservation_id, event_kind, amount_usd, reason, idempotency_key, actor_id,
  created_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)
"#,
                )
                .bind(&record.event_id)
                .bind(&reservation_id)
                .bind(record.event_kind().as_str())
                .bind(amount_usd)
                .bind(&record.release_reason)
                .bind(&record.event_idempotency_key)
                .bind(&record.actor_id)
                .bind(finalized_at_unix_ms)
                .execute(&mut *tx)
                .await
                .map_sql_err()?;
            }
        }
        tx.commit().await.map_sql_err()?;
        self.find_billing_reservation_by_request_id(&record.request_id)
            .await
    }

    async fn create_billing_reservation_dry_run(
        &self,
        record: CreateNifflerBillingReservationDryRunRecord,
    ) -> Result<StoredNifflerBillingReservationDryRun, DataLayerError> {
        record.validate()?;
        let finalized_at_unix_ms = record
            .finalized_at_unix_ms
            .map(|value| i64_from_u64(value, "finalized_at_unix_ms"))
            .transpose()?;
        sqlx::query(
            r#"
INSERT INTO niffler_billing_reservation_dry_runs (
  id, request_id, user_id, api_key_id, product_plan_id, requested_model_name,
  estimated_reservation_usd, legacy_final_charge_usd, difference_usd,
  estimation_source, status, created_at_unix_ms, finalized_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT (request_id) DO UPDATE SET
  user_id = excluded.user_id,
  api_key_id = excluded.api_key_id,
  product_plan_id = excluded.product_plan_id,
  requested_model_name = excluded.requested_model_name,
  estimated_reservation_usd = excluded.estimated_reservation_usd,
  legacy_final_charge_usd = excluded.legacy_final_charge_usd,
  difference_usd = excluded.difference_usd,
  estimation_source = excluded.estimation_source,
  status = excluded.status,
  created_at_unix_ms = excluded.created_at_unix_ms,
  finalized_at_unix_ms = excluded.finalized_at_unix_ms
"#,
        )
        .bind(&record.id)
        .bind(&record.request_id)
        .bind(&record.user_id)
        .bind(&record.api_key_id)
        .bind(&record.product_plan_id)
        .bind(&record.requested_model_name)
        .bind(record.estimated_reservation_usd)
        .bind(record.legacy_final_charge_usd)
        .bind(record.difference_usd)
        .bind(&record.estimation_source)
        .bind(&record.status)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(finalized_at_unix_ms)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_billing_reservation_dry_run_by_request_id(&record.request_id)
            .await
    }

    async fn create_referral_reward_ledger(
        &self,
        record: CreateNifflerReferralRewardLedgerRecord,
    ) -> Result<StoredNifflerReferralRewardLedger, DataLayerError> {
        record.validate()?;
        let rule_snapshot = json_to_string(
            Some(&record.rule_snapshot),
            "niffler_referral_reward_ledger.rule_snapshot",
        )?;
        sqlx::query(
            r#"
INSERT OR IGNORE INTO niffler_referral_reward_ledger (
  id, order_id, idempotency_key, inviter_user_id, invitee_user_id, rule_id,
  reward_amount_usd, rule_snapshot, status, failure_reason, retry_count,
  paid_at_unix_ms, cancelled_at_unix_ms, created_at_unix_ms, updated_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, 0, NULL, NULL, ?, ?)
"#,
        )
        .bind(&record.id)
        .bind(&record.order_id)
        .bind(&record.idempotency_key)
        .bind(&record.inviter_user_id)
        .bind(&record.invitee_user_id)
        .bind(&record.rule_id)
        .bind(record.reward_amount_usd)
        .bind(rule_snapshot)
        .bind(record.status.as_str())
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_referral_reward_ledger_by_order_id(&record.order_id)
            .await
    }

    async fn create_route_attempt(
        &self,
        record: CreateNifflerRouteAttemptRecord,
    ) -> Result<StoredNifflerRouteAttempt, DataLayerError> {
        record.validate()?;
        let attempt_index = i32::try_from(record.attempt_index).map_err(|_| {
            DataLayerError::InvalidInput("route_attempts.attempt_index is too large".to_string())
        })?;
        let latency_ms = record
            .latency_ms
            .map(|value| i64_from_u64(value, "latency_ms"))
            .transpose()?;
        sqlx::query(
            r#"
INSERT INTO niffler_route_attempts (
  id, request_id, upstream_service_id, upstream_account_id, product_plan_id,
  model_name, attempt_index, status, skip_reason, upstream_status_code,
  latency_ms, created_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(id) DO UPDATE SET
  request_id = excluded.request_id,
  upstream_service_id = excluded.upstream_service_id,
  upstream_account_id = excluded.upstream_account_id,
  product_plan_id = excluded.product_plan_id,
  model_name = excluded.model_name,
  attempt_index = excluded.attempt_index,
  status = excluded.status,
  skip_reason = excluded.skip_reason,
  upstream_status_code = excluded.upstream_status_code,
  latency_ms = excluded.latency_ms,
  created_at_unix_ms = excluded.created_at_unix_ms
"#,
        )
        .bind(&record.id)
        .bind(&record.request_id)
        .bind(&record.upstream_service_id)
        .bind(&record.upstream_account_id)
        .bind(&record.product_plan_id)
        .bind(&record.model_name)
        .bind(attempt_index)
        .bind(&record.status)
        .bind(&record.skip_reason)
        .bind(record.upstream_status_code.map(i32::from))
        .bind(latency_ms)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        Ok(StoredNifflerRouteAttempt {
            id: record.id,
            request_id: record.request_id,
            upstream_service_id: record.upstream_service_id,
            upstream_account_id: record.upstream_account_id,
            product_plan_id: record.product_plan_id,
            model_name: record.model_name,
            attempt_index: record.attempt_index,
            status: record.status,
            skip_reason: record.skip_reason,
            upstream_status_code: record.upstream_status_code,
            latency_ms: record.latency_ms,
            created_at_unix_ms: record.created_at_unix_ms,
        })
    }

    async fn upsert_stability_observation(
        &self,
        record: UpsertNifflerStabilityObservationRecord,
    ) -> Result<StoredNifflerStabilityObservation, DataLayerError> {
        record.validate()?;
        let blocker_codes = json_to_string(
            Some(&serde_json::json!(record.blocker_codes)),
            "niffler_stability_observations.blocker_codes",
        )?;
        let summary = json_to_string(
            record.summary.as_ref(),
            "niffler_stability_observations.summary",
        )?;
        sqlx::query(
            r#"
INSERT INTO niffler_stability_observations (
  id, window_start_unix_ms, window_end_unix_ms, status, rollback_drill_status,
  consistency_checked_count, consistency_issue_count, unknown_upstream_count,
  legacy_write_call_count, billing_reservation_exception_count, referral_exception_count,
  blocker_codes, summary, created_at_unix_ms, updated_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(window_start_unix_ms, window_end_unix_ms) DO UPDATE SET
  status = excluded.status,
  rollback_drill_status = excluded.rollback_drill_status,
  consistency_checked_count = excluded.consistency_checked_count,
  consistency_issue_count = excluded.consistency_issue_count,
  unknown_upstream_count = excluded.unknown_upstream_count,
  legacy_write_call_count = excluded.legacy_write_call_count,
  billing_reservation_exception_count = excluded.billing_reservation_exception_count,
  referral_exception_count = excluded.referral_exception_count,
  blocker_codes = excluded.blocker_codes,
  summary = excluded.summary,
  updated_at_unix_ms = excluded.updated_at_unix_ms
"#,
        )
        .bind(&record.id)
        .bind(i64_from_u64(
            record.window_start_unix_ms,
            "window_start_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.window_end_unix_ms,
            "window_end_unix_ms",
        )?)
        .bind(&record.status)
        .bind(&record.rollback_drill_status)
        .bind(i64_from_u64(
            record.consistency_checked_count,
            "consistency_checked_count",
        )?)
        .bind(i64_from_u64(
            record.consistency_issue_count,
            "consistency_issue_count",
        )?)
        .bind(i64_from_u64(
            record.unknown_upstream_count,
            "unknown_upstream_count",
        )?)
        .bind(i64_from_u64(
            record.legacy_write_call_count,
            "legacy_write_call_count",
        )?)
        .bind(i64_from_u64(
            record.billing_reservation_exception_count,
            "billing_reservation_exception_count",
        )?)
        .bind(i64_from_u64(
            record.referral_exception_count,
            "referral_exception_count",
        )?)
        .bind(blocker_codes)
        .bind(summary)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        reload_stability_observation_by_window(
            &self.pool,
            record.window_start_unix_ms,
            record.window_end_unix_ms,
        )
        .await
    }
}

async fn reload_stability_observation_by_window(
    pool: &SqlitePool,
    window_start_unix_ms: u64,
    window_end_unix_ms: u64,
) -> Result<StoredNifflerStabilityObservation, DataLayerError> {
    let row = sqlx::query(
        r#"
SELECT
  id, window_start_unix_ms, window_end_unix_ms, status, rollback_drill_status,
  consistency_checked_count, consistency_issue_count, unknown_upstream_count,
  legacy_write_call_count, billing_reservation_exception_count, referral_exception_count,
  blocker_codes, summary, created_at_unix_ms, updated_at_unix_ms
FROM niffler_stability_observations
WHERE window_start_unix_ms = ? AND window_end_unix_ms = ?
LIMIT 1
"#,
    )
    .bind(i64_from_u64(window_start_unix_ms, "window_start_unix_ms")?)
    .bind(i64_from_u64(window_end_unix_ms, "window_end_unix_ms")?)
    .fetch_optional(pool)
    .await
    .map_sql_err()?;
    row.as_ref()
        .map(map_stability_observation_row)
        .transpose()?
        .ok_or_else(|| {
            DataLayerError::UnexpectedValue(
                "niffler stability observation missing after write".into(),
            )
        })
}

fn build_service_count_query(query: &NifflerUpstreamServiceListQuery) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_upstream_services");
    push_service_filters(&mut builder, query);
    builder
}

fn build_service_rows_query(query: &NifflerUpstreamServiceListQuery) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT id, display_name, service_kind, default_api_format, base_url, cost_multiplier, \
         is_active, config, created_at_unix_ms, updated_at_unix_ms FROM niffler_upstream_services",
    );
    push_service_filters(&mut builder, query);
    builder.push(" ORDER BY created_at_unix_ms DESC, display_name ASC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_service_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerUpstreamServiceListQuery,
) {
    let mut has_where = false;
    if !query.include_inactive {
        builder.push(" WHERE is_active = TRUE");
        has_where = true;
    }
    if let Some(search) = query
        .search
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("(LOWER(display_name) LIKE LOWER(");
        builder.push_bind(format!("%{search}%"));
        builder.push(") OR LOWER(service_kind) LIKE LOWER(");
        builder.push_bind(format!("%{search}%"));
        builder.push("))");
    }
}

fn build_account_count_query(query: &NifflerUpstreamAccountListQuery) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_upstream_accounts");
    push_account_filters(&mut builder, query);
    builder
}

fn build_account_rows_query(query: &NifflerUpstreamAccountListQuery) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT id, upstream_service_id, display_name, email, phone, auth_kind, status, \
         cost_multiplier, priority, cooldown_until_unix_ms, last_tested_at_unix_ms, \
         last_test_error, config, created_at_unix_ms, updated_at_unix_ms \
         FROM niffler_upstream_accounts",
    );
    push_account_filters(&mut builder, query);
    builder.push(" ORDER BY priority ASC, created_at_unix_ms DESC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_account_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerUpstreamAccountListQuery,
) {
    let mut has_where = false;
    if let Some(service_id) = query
        .upstream_service_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(" WHERE upstream_service_id = ");
        builder.push_bind(service_id.clone());
        has_where = true;
    }
    if let Some(status) = query.status {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("status = ");
        builder.push_bind(status.as_str());
        has_where = true;
    }
    if let Some(search) = query
        .search
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("(LOWER(display_name) LIKE LOWER(");
        builder.push_bind(format!("%{search}%"));
        builder.push(") OR LOWER(email) LIKE LOWER(");
        builder.push_bind(format!("%{search}%"));
        builder.push(") OR LOWER(phone) LIKE LOWER(");
        builder.push_bind(format!("%{search}%"));
        builder.push("))");
    }
}

fn build_account_model_capability_count_query(
    query: &NifflerAccountModelCapabilityListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_account_model_capabilities");
    push_account_model_capability_filters(&mut builder, query);
    builder
}

fn build_account_model_capability_rows_query(
    query: &NifflerAccountModelCapabilityListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT id, upstream_service_id, upstream_account_id, model_name, is_enabled, \
         source, last_checked_at_unix_ms, last_error, created_at_unix_ms, updated_at_unix_ms \
         FROM niffler_account_model_capabilities",
    );
    push_account_model_capability_filters(&mut builder, query);
    builder.push(" ORDER BY model_name ASC, upstream_account_id ASC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_account_model_capability_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerAccountModelCapabilityListQuery,
) {
    let mut has_where = false;
    if let Some(service_id) = query
        .upstream_service_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(" WHERE upstream_service_id = ");
        builder.push_bind(service_id.clone());
        has_where = true;
    }
    if let Some(account_id) = query
        .upstream_account_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("upstream_account_id = ");
        builder.push_bind(account_id.clone());
        has_where = true;
    }
    if let Some(model_name) = query
        .model_name
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("model_name = ");
        builder.push_bind(model_name.clone());
        has_where = true;
    }
    if query.enabled_only {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("is_enabled = TRUE");
    }
}

fn build_runtime_account_model_access_count_query(
    query: &NifflerRuntimeAccountModelAccessListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT COUNT(*) \
         FROM niffler_account_model_capabilities c \
         INNER JOIN niffler_upstream_accounts a ON a.id = c.upstream_account_id \
         INNER JOIN niffler_upstream_services s ON s.id = c.upstream_service_id",
    );
    push_runtime_account_model_access_filters(&mut builder, query);
    builder
}

fn build_runtime_account_model_access_rows_query(
    query: &NifflerRuntimeAccountModelAccessListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT c.upstream_service_id, c.upstream_account_id \
         FROM niffler_account_model_capabilities c \
         INNER JOIN niffler_upstream_accounts a ON a.id = c.upstream_account_id \
         INNER JOIN niffler_upstream_services s ON s.id = c.upstream_service_id",
    );
    push_runtime_account_model_access_filters(&mut builder, query);
    builder.push(" ORDER BY a.priority ASC, c.upstream_account_id ASC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_runtime_account_model_access_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerRuntimeAccountModelAccessListQuery,
) {
    let now_unix_ms = i64::try_from(query.now_unix_ms).unwrap_or(i64::MAX);
    builder.push(" WHERE c.model_name = ");
    builder.push_bind(query.model_name.clone());
    builder.push(" AND c.is_enabled = TRUE");
    builder.push(" AND s.is_active = TRUE");
    builder.push(" AND a.upstream_service_id = c.upstream_service_id");
    builder.push(" AND a.status = ");
    builder.push_bind(NifflerAccountStatus::Available.as_str());
    builder.push(" AND (a.cooldown_until_unix_ms IS NULL OR a.cooldown_until_unix_ms <= ");
    builder.push_bind(now_unix_ms);
    builder.push(")");
}

fn build_product_plan_count_query(query: &NifflerProductPlanListQuery) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_product_plans");
    push_product_plan_filters(&mut builder, query);
    builder
}

fn build_product_plan_rows_query(query: &NifflerProductPlanListQuery) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT id, display_name, is_public, is_active, sales_multiplier, description, \
         created_at_unix_ms, updated_at_unix_ms FROM niffler_product_plans",
    );
    push_product_plan_filters(&mut builder, query);
    builder.push(" ORDER BY created_at_unix_ms DESC, display_name ASC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_product_plan_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerProductPlanListQuery,
) {
    let mut has_where = false;
    if !query.include_inactive {
        builder.push(" WHERE is_active = TRUE");
        has_where = true;
    }
    if query.public_only {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("is_public = TRUE");
        has_where = true;
    }
    if let Some(search) = query
        .search
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("(LOWER(display_name) LIKE LOWER(");
        builder.push_bind(format!("%{search}%"));
        builder.push(") OR LOWER(description) LIKE LOWER(");
        builder.push_bind(format!("%{search}%"));
        builder.push("))");
    }
}

fn build_product_plan_model_count_query(
    query: &NifflerProductPlanModelListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_product_plan_models");
    push_product_plan_model_filters(&mut builder, query);
    builder
}

fn build_product_plan_model_rows_query(
    query: &NifflerProductPlanModelListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT id, product_plan_id, model_name, is_enabled, sales_multiplier_override, \
         created_at_unix_ms, updated_at_unix_ms FROM niffler_product_plan_models",
    );
    push_product_plan_model_filters(&mut builder, query);
    builder.push(" ORDER BY model_name ASC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_product_plan_model_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerProductPlanModelListQuery,
) {
    builder.push(" WHERE product_plan_id = ");
    builder.push_bind(query.product_plan_id.clone());
    if query.enabled_only {
        builder.push(" AND is_enabled = TRUE");
    }
    if let Some(search) = query
        .search
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        builder.push(" AND LOWER(model_name) LIKE LOWER(");
        builder.push_bind(format!("%{search}%"));
        builder.push(")");
    }
}

fn build_api_key_product_plan_binding_count_query(
    query: &NifflerApiKeyProductPlanBindingListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder =
        QueryBuilder::new("SELECT COUNT(*) FROM niffler_api_key_product_plan_bindings");
    push_api_key_product_plan_binding_filters(&mut builder, query);
    builder
}

fn build_api_key_product_plan_binding_rows_query(
    query: &NifflerApiKeyProductPlanBindingListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT id, api_key_id, product_plan_id, config, created_at_unix_ms, updated_at_unix_ms \
         FROM niffler_api_key_product_plan_bindings",
    );
    push_api_key_product_plan_binding_filters(&mut builder, query);
    builder.push(" ORDER BY updated_at_unix_ms DESC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_api_key_product_plan_binding_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerApiKeyProductPlanBindingListQuery,
) {
    if let Some(product_plan_id) = query
        .product_plan_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(" WHERE product_plan_id = ");
        builder.push_bind(product_plan_id.clone());
    }
}

fn build_runtime_rollout_setting_count_query(
    query: &NifflerRuntimeRolloutSettingListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_runtime_rollout_settings");
    push_runtime_rollout_setting_filters(&mut builder, query);
    builder
}

fn build_runtime_rollout_setting_rows_query(
    query: &NifflerRuntimeRolloutSettingListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT id, target_scope, target_id, enable_new_routing, enable_settlement_snapshot, \
         enable_error_return_rules, enable_billing_reservation, enable_referral_ledger, \
         is_active, config, created_at_unix_ms, updated_at_unix_ms \
         FROM niffler_runtime_rollout_settings",
    );
    push_runtime_rollout_setting_filters(&mut builder, query);
    builder.push(" ORDER BY is_active DESC, updated_at_unix_ms DESC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_runtime_rollout_setting_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerRuntimeRolloutSettingListQuery,
) {
    let mut has_where = false;
    if let Some(target_scope) = query.target_scope {
        builder.push(" WHERE target_scope = ");
        builder.push_bind(target_scope.as_str());
        has_where = true;
    }
    if !query.include_inactive {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("is_active = 1");
    }
}

fn build_error_return_setting_count_query(
    query: &NifflerErrorReturnSettingListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_error_return_settings");
    push_error_return_setting_filters(&mut builder, query);
    builder
}

fn build_error_return_setting_rows_query(
    query: &NifflerErrorReturnSettingListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT id, scope, upstream_service_id, match_status_code, match_text, handling_step, \
         response_mode, user_message, account_protection_action, pause_duration, is_active, \
         created_at_unix_ms, updated_at_unix_ms FROM niffler_error_return_settings",
    );
    push_error_return_setting_filters(&mut builder, query);
    builder.push(" ORDER BY scope ASC, is_active DESC, created_at_unix_ms DESC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_error_return_setting_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerErrorReturnSettingListQuery,
) {
    let mut has_where = false;
    if let Some(scope) = query.scope {
        builder.push(" WHERE scope = ");
        builder.push_bind(scope.as_str());
        has_where = true;
    }
    if let Some(upstream_service_id) = query
        .upstream_service_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("upstream_service_id = ");
        builder.push_bind(upstream_service_id.clone());
        has_where = true;
    }
    if !query.include_inactive {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("is_active = 1");
    }
}

fn build_settlement_snapshot_count_query(
    query: &NifflerSettlementSnapshotListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_settlement_snapshots ss");
    push_settlement_snapshot_filters(&mut builder, query);
    builder
}

fn build_settlement_snapshot_rows_query(
    query: &NifflerSettlementSnapshotListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT ss.id, ss.request_id, ss.user_id, ss.api_key_id, ss.product_plan_id, \
         pp.display_name AS product_plan_name, ss.upstream_service_id, \
         us.display_name AS upstream_service_name, ss.upstream_account_id, \
         ua.display_name AS upstream_account_display_name, ua.email AS upstream_account_email, \
         ua.phone AS upstream_account_phone, ss.requested_model_name, \
         ss.upstream_execution_model_name, ss.image_tool_model_name, ss.pricing_snapshot, \
         ss.wallet_charge_usd, ss.entitlement_charge_usd, ss.upstream_cost_usd, \
         ss.gross_margin_usd, ss.created_at_unix_ms, ss.finalized_at_unix_ms \
         FROM niffler_settlement_snapshots ss \
         LEFT JOIN niffler_product_plans pp ON pp.id = ss.product_plan_id \
         LEFT JOIN niffler_upstream_services us ON us.id = ss.upstream_service_id \
         LEFT JOIN niffler_upstream_accounts ua ON ua.id = ss.upstream_account_id",
    );
    push_settlement_snapshot_filters(&mut builder, query);
    builder.push(" ORDER BY ss.created_at_unix_ms DESC, ss.request_id ASC LIMIT ");
    builder.push_bind(bounded_limit(query.limit.min(100)));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_settlement_snapshot_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerSettlementSnapshotListQuery,
) {
    let mut has_where = false;
    if let Some(request_id) = query
        .request_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(" WHERE ss.request_id = ");
        builder.push_bind(request_id.clone());
        has_where = true;
    }
    if let Some(user_id) = query
        .user_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("ss.user_id = ");
        builder.push_bind(user_id.clone());
        has_where = true;
    }
    if let Some(api_key_id) = query
        .api_key_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("ss.api_key_id = ");
        builder.push_bind(api_key_id.clone());
        has_where = true;
    }
    if let Some(product_plan_id) = query
        .product_plan_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("ss.product_plan_id = ");
        builder.push_bind(product_plan_id.clone());
    }
}

fn build_consistency_check_rows_query(
    query: &NifflerConsistencyCheckListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT ss.request_id, ss.user_id, ss.api_key_id, ss.product_plan_id, \
         pp.display_name AS product_plan_name, u.status AS usage_status, \
         COALESCE(uss.billing_status, u.billing_status) AS usage_billing_status, \
         CAST(u.total_cost_usd AS REAL) AS usage_total_cost_usd, \
         CASE WHEN COALESCE(uss.wallet_recharge_balance_before, u.wallet_recharge_balance_before) IS NULL \
              AND COALESCE(uss.wallet_recharge_balance_after, u.wallet_recharge_balance_after) IS NULL \
              AND COALESCE(uss.wallet_gift_balance_before, u.wallet_gift_balance_before) IS NULL \
              AND COALESCE(uss.wallet_gift_balance_after, u.wallet_gift_balance_after) IS NULL \
              THEN NULL \
              ELSE MAX(0.0, COALESCE(COALESCE(uss.wallet_recharge_balance_before, u.wallet_recharge_balance_before), 0.0) \
                   - COALESCE(COALESCE(uss.wallet_recharge_balance_after, u.wallet_recharge_balance_after), 0.0)) \
                 + MAX(0.0, COALESCE(COALESCE(uss.wallet_gift_balance_before, u.wallet_gift_balance_before), 0.0) \
                   - COALESCE(COALESCE(uss.wallet_gift_balance_after, u.wallet_gift_balance_after), 0.0)) \
         END AS legacy_wallet_charge_usd, \
         COALESCE((SELECT SUM(eul.amount_usd) FROM entitlement_usage_ledgers eul WHERE eul.request_id = ss.request_id), 0.0) \
           AS legacy_entitlement_charge_usd, \
         ss.wallet_charge_usd AS niffler_wallet_charge_usd, \
         ss.entitlement_charge_usd AS niffler_entitlement_charge_usd, \
         ss.wallet_charge_usd + ss.entitlement_charge_usd AS niffler_total_charge_usd, \
         br.id AS reservation_id, br.status AS reservation_status, \
         br.release_reason AS reservation_release_reason, \
         COALESCE((SELECT COUNT(*) FROM niffler_route_attempts ra WHERE ra.request_id = ss.request_id), 0) \
           AS route_attempt_count, \
         COALESCE((SELECT COUNT(*) FROM niffler_route_attempts ra WHERE ra.request_id = ss.request_id AND ra.status = 'success'), 0) \
           AS successful_route_attempt_count, \
         ss.created_at_unix_ms \
         FROM niffler_settlement_snapshots ss \
         LEFT JOIN niffler_product_plans pp ON pp.id = ss.product_plan_id \
         LEFT JOIN \"usage\" u ON u.request_id = ss.request_id \
         LEFT JOIN usage_settlement_snapshots uss ON uss.request_id = ss.request_id \
         LEFT JOIN niffler_billing_reservations br ON br.request_id = ss.request_id",
    );
    push_consistency_check_filters(&mut builder, query);
    builder.push(" ORDER BY ss.created_at_unix_ms DESC, ss.request_id ASC LIMIT ");
    builder.push_bind(bounded_limit(query.limit.min(100)));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_consistency_check_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerConsistencyCheckListQuery,
) {
    let mut has_where = false;
    if let Some(request_id) = query
        .request_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(" WHERE ss.request_id = ");
        builder.push_bind(request_id.clone());
        has_where = true;
    }
    if let Some(user_id) = query
        .user_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("ss.user_id = ");
        builder.push_bind(user_id.clone());
        has_where = true;
    }
    if let Some(api_key_id) = query
        .api_key_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("ss.api_key_id = ");
        builder.push_bind(api_key_id.clone());
        has_where = true;
    }
    if let Some(product_plan_id) = query
        .product_plan_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("ss.product_plan_id = ");
        builder.push_bind(product_plan_id.clone());
    }
}

fn build_stability_observation_count_query(
    query: &NifflerStabilityObservationListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_stability_observations");
    push_stability_observation_filters(&mut builder, query);
    builder
}

fn build_stability_observation_rows_query(
    query: &NifflerStabilityObservationListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT id, window_start_unix_ms, window_end_unix_ms, status, rollback_drill_status, \
         consistency_checked_count, consistency_issue_count, unknown_upstream_count, \
         legacy_write_call_count, billing_reservation_exception_count, referral_exception_count, \
         blocker_codes, summary, created_at_unix_ms, updated_at_unix_ms \
         FROM niffler_stability_observations",
    );
    push_stability_observation_filters(&mut builder, query);
    builder.push(" ORDER BY window_end_unix_ms DESC LIMIT ");
    builder.push_bind(bounded_limit(query.limit.min(100)));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_stability_observation_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerStabilityObservationListQuery,
) {
    if let Some(status) = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        builder.push(" WHERE status = ");
        builder.push_bind(status.to_string());
    }
}

fn build_billing_reservation_count_query(
    query: &NifflerBillingReservationListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_billing_reservations");
    push_billing_reservation_filters(&mut builder, query);
    builder
}

fn build_billing_reservation_rows_query(
    query: &NifflerBillingReservationListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT id, request_id, user_id, api_key_id, product_plan_id, status, \
         reserved_total_usd, wallet_reserved_usd, entitlement_reserved_usd, \
         reserved_at_unix_ms, expires_at_unix_ms, finalized_at_unix_ms, \
         settlement_snapshot_id, release_reason, idempotency_key \
         FROM niffler_billing_reservations",
    );
    push_billing_reservation_filters(&mut builder, query);
    builder.push(" ORDER BY reserved_at_unix_ms DESC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_billing_reservation_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerBillingReservationListQuery,
) {
    let mut has_where = false;
    if let Some(status) = query.status {
        builder.push(" WHERE status = ");
        builder.push_bind(status.as_str());
        has_where = true;
    }
    if let Some(user_id) = query
        .user_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("user_id = ");
        builder.push_bind(user_id.clone());
        has_where = true;
    }
    if let Some(api_key_id) = query
        .api_key_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("api_key_id = ");
        builder.push_bind(api_key_id.clone());
        has_where = true;
    }
    if let Some(request_id) = query
        .request_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("request_id = ");
        builder.push_bind(request_id.clone());
        has_where = true;
    }
    if let Some(expires_at_lte_unix_ms) = query.expires_at_lte_unix_ms {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("expires_at_unix_ms <= ");
        builder.push_bind(i64::try_from(expires_at_lte_unix_ms).unwrap_or(i64::MAX));
    }
}

fn build_billing_reservation_dry_run_count_query(
    query: &NifflerBillingReservationDryRunListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder =
        QueryBuilder::new("SELECT COUNT(*) FROM niffler_billing_reservation_dry_runs");
    push_billing_reservation_dry_run_filters(&mut builder, query);
    builder
}

fn build_billing_reservation_dry_run_rows_query(
    query: &NifflerBillingReservationDryRunListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT id, request_id, user_id, api_key_id, product_plan_id, requested_model_name, \
         estimated_reservation_usd, legacy_final_charge_usd, difference_usd, \
         estimation_source, status, created_at_unix_ms, finalized_at_unix_ms \
         FROM niffler_billing_reservation_dry_runs",
    );
    push_billing_reservation_dry_run_filters(&mut builder, query);
    builder.push(" ORDER BY created_at_unix_ms DESC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_billing_reservation_dry_run_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerBillingReservationDryRunListQuery,
) {
    let mut has_where = false;
    if let Some(status) = query
        .status
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(" WHERE status = ");
        builder.push_bind(status.clone());
        has_where = true;
    }
    if let Some(user_id) = query
        .user_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("user_id = ");
        builder.push_bind(user_id.clone());
        has_where = true;
    }
    if let Some(api_key_id) = query
        .api_key_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("api_key_id = ");
        builder.push_bind(api_key_id.clone());
        has_where = true;
    }
    if let Some(product_plan_id) = query
        .product_plan_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("product_plan_id = ");
        builder.push_bind(product_plan_id.clone());
        has_where = true;
    }
    if let Some(request_id) = query
        .request_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("request_id = ");
        builder.push_bind(request_id.clone());
    }
}

fn build_referral_reward_ledger_count_query(
    query: &NifflerReferralRewardLedgerListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_referral_reward_ledger");
    push_referral_reward_ledger_filters(&mut builder, query);
    builder
}

fn build_referral_reward_ledger_rows_query(
    query: &NifflerReferralRewardLedgerListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT id, order_id, idempotency_key, inviter_user_id, invitee_user_id, rule_id, \
         reward_amount_usd, rule_snapshot, status, failure_reason, retry_count, \
         paid_at_unix_ms, cancelled_at_unix_ms, created_at_unix_ms, updated_at_unix_ms \
         FROM niffler_referral_reward_ledger",
    );
    push_referral_reward_ledger_filters(&mut builder, query);
    builder.push(" ORDER BY created_at_unix_ms DESC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_referral_reward_ledger_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerReferralRewardLedgerListQuery,
) {
    let mut has_where = false;
    if let Some(status) = query.status {
        builder.push(" WHERE status = ");
        builder.push_bind(status.as_str());
        has_where = true;
    }
    if let Some(inviter_user_id) = query
        .inviter_user_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("inviter_user_id = ");
        builder.push_bind(inviter_user_id.clone());
        has_where = true;
    }
    if let Some(invitee_user_id) = query
        .invitee_user_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("invitee_user_id = ");
        builder.push_bind(invitee_user_id.clone());
        has_where = true;
    }
    if let Some(order_id) = query
        .order_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("order_id = ");
        builder.push_bind(order_id.clone());
    }
}

fn build_route_attempt_count_query(
    query: &NifflerRouteAttemptListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_route_attempts ra");
    push_route_attempt_filters(&mut builder, query);
    builder
}

fn build_route_attempt_rows_query(
    query: &NifflerRouteAttemptListQuery,
) -> QueryBuilder<'_, Sqlite> {
    let mut builder = QueryBuilder::new(
        "SELECT ra.id, ra.request_id, ra.upstream_service_id, us.display_name AS upstream_service_name, \
         ra.upstream_account_id, ua.display_name AS upstream_account_display_name, \
         ua.email AS upstream_account_email, ua.phone AS upstream_account_phone, \
         ra.product_plan_id, pp.display_name AS product_plan_name, ra.model_name, \
         ra.attempt_index, ra.status, ra.skip_reason, ra.upstream_status_code, ra.latency_ms, \
         ra.created_at_unix_ms FROM niffler_route_attempts ra \
         LEFT JOIN niffler_upstream_services us ON us.id = ra.upstream_service_id \
         LEFT JOIN niffler_upstream_accounts ua ON ua.id = ra.upstream_account_id \
         LEFT JOIN niffler_product_plans pp ON pp.id = ra.product_plan_id",
    );
    push_route_attempt_filters(&mut builder, query);
    builder.push(
        " ORDER BY ra.created_at_unix_ms DESC, ra.request_id ASC, ra.attempt_index ASC LIMIT ",
    );
    builder.push_bind(bounded_limit(query.limit.min(100)));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_route_attempt_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    query: &NifflerRouteAttemptListQuery,
) {
    let mut has_where = false;
    if let Some(request_id) = query
        .request_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(" WHERE ra.request_id = ");
        builder.push_bind(request_id.clone());
        has_where = true;
    }
    if let Some(upstream_service_id) = query
        .upstream_service_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("ra.upstream_service_id = ");
        builder.push_bind(upstream_service_id.clone());
        has_where = true;
    }
    if let Some(upstream_account_id) = query
        .upstream_account_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("ra.upstream_account_id = ");
        builder.push_bind(upstream_account_id.clone());
        has_where = true;
    }
    if let Some(status) = query
        .status
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("ra.status = ");
        builder.push_bind(status.clone());
    }
}

fn map_service_row(row: &SqliteRow) -> Result<StoredNifflerUpstreamService, DataLayerError> {
    Ok(StoredNifflerUpstreamService {
        id: row.try_get("id").map_sql_err()?,
        display_name: row.try_get("display_name").map_sql_err()?,
        service_kind: row.try_get("service_kind").map_sql_err()?,
        default_api_format: row.try_get("default_api_format").map_sql_err()?,
        base_url: row.try_get("base_url").map_sql_err()?,
        cost_multiplier: row.try_get("cost_multiplier").map_sql_err()?,
        is_active: row.try_get("is_active").map_sql_err()?,
        config: json_from_string(
            row.try_get("config").map_sql_err()?,
            "niffler_upstream_services.config",
        )?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_account_row(row: &SqliteRow) -> Result<StoredNifflerUpstreamAccount, DataLayerError> {
    let status: String = row.try_get("status").map_sql_err()?;
    Ok(StoredNifflerUpstreamAccount {
        id: row.try_get("id").map_sql_err()?,
        upstream_service_id: row.try_get("upstream_service_id").map_sql_err()?,
        display_name: row.try_get("display_name").map_sql_err()?,
        email: row.try_get("email").map_sql_err()?,
        phone: row.try_get("phone").map_sql_err()?,
        auth_kind: row.try_get("auth_kind").map_sql_err()?,
        status: NifflerAccountStatus::from_database(&status)?,
        cost_multiplier: row.try_get("cost_multiplier").map_sql_err()?,
        priority: row.try_get("priority").map_sql_err()?,
        cooldown_until_unix_ms: row
            .try_get::<Option<i64>, _>("cooldown_until_unix_ms")
            .map_sql_err()?
            .map(|value| super::u64_from_i64(value, "cooldown_until_unix_ms"))
            .transpose()?,
        last_tested_at_unix_ms: row
            .try_get::<Option<i64>, _>("last_tested_at_unix_ms")
            .map_sql_err()?
            .map(|value| super::u64_from_i64(value, "last_tested_at_unix_ms"))
            .transpose()?,
        last_test_error: row.try_get("last_test_error").map_sql_err()?,
        config: json_from_string(
            row.try_get("config").map_sql_err()?,
            "niffler_upstream_accounts.config",
        )?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_account_model_capability_row(
    row: &SqliteRow,
) -> Result<StoredNifflerAccountModelCapability, DataLayerError> {
    Ok(StoredNifflerAccountModelCapability {
        id: row.try_get("id").map_sql_err()?,
        upstream_service_id: row.try_get("upstream_service_id").map_sql_err()?,
        upstream_account_id: row.try_get("upstream_account_id").map_sql_err()?,
        model_name: row.try_get("model_name").map_sql_err()?,
        is_enabled: row.try_get("is_enabled").map_sql_err()?,
        source: row.try_get("source").map_sql_err()?,
        last_checked_at_unix_ms: row
            .try_get::<Option<i64>, _>("last_checked_at_unix_ms")
            .map_sql_err()?
            .map(|value| super::u64_from_i64(value, "last_checked_at_unix_ms"))
            .transpose()?,
        last_error: row.try_get("last_error").map_sql_err()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_runtime_account_model_access_row(
    row: &SqliteRow,
) -> Result<StoredNifflerRuntimeAccountModelAccess, DataLayerError> {
    Ok(StoredNifflerRuntimeAccountModelAccess {
        upstream_service_id: row.try_get("upstream_service_id").map_sql_err()?,
        upstream_account_id: row.try_get("upstream_account_id").map_sql_err()?,
    })
}

fn map_product_plan_row(row: &SqliteRow) -> Result<StoredNifflerProductPlan, DataLayerError> {
    Ok(StoredNifflerProductPlan {
        id: row.try_get("id").map_sql_err()?,
        display_name: row.try_get("display_name").map_sql_err()?,
        is_public: row.try_get("is_public").map_sql_err()?,
        is_active: row.try_get("is_active").map_sql_err()?,
        sales_multiplier: row.try_get("sales_multiplier").map_sql_err()?,
        description: row.try_get("description").map_sql_err()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_product_plan_model_row(
    row: &SqliteRow,
) -> Result<StoredNifflerProductPlanModel, DataLayerError> {
    Ok(StoredNifflerProductPlanModel {
        id: row.try_get("id").map_sql_err()?,
        product_plan_id: row.try_get("product_plan_id").map_sql_err()?,
        model_name: row.try_get("model_name").map_sql_err()?,
        is_enabled: row.try_get("is_enabled").map_sql_err()?,
        sales_multiplier_override: row.try_get("sales_multiplier_override").map_sql_err()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_api_key_product_plan_binding_row(
    row: &SqliteRow,
) -> Result<StoredNifflerApiKeyProductPlanBinding, DataLayerError> {
    Ok(StoredNifflerApiKeyProductPlanBinding {
        id: row.try_get("id").map_sql_err()?,
        api_key_id: row.try_get("api_key_id").map_sql_err()?,
        product_plan_id: row.try_get("product_plan_id").map_sql_err()?,
        config: json_from_string(
            row.try_get("config").map_sql_err()?,
            "niffler_api_key_product_plan_bindings.config",
        )?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_runtime_rollout_setting_row(
    row: &SqliteRow,
) -> Result<StoredNifflerRuntimeRolloutSetting, DataLayerError> {
    let target_scope: String = row.try_get("target_scope").map_sql_err()?;
    Ok(StoredNifflerRuntimeRolloutSetting {
        id: row.try_get("id").map_sql_err()?,
        target_scope: NifflerRuntimeRolloutTargetScope::from_database(&target_scope)?,
        target_id: row.try_get("target_id").map_sql_err()?,
        enable_new_routing: row.try_get("enable_new_routing").map_sql_err()?,
        enable_settlement_snapshot: row.try_get("enable_settlement_snapshot").map_sql_err()?,
        enable_error_return_rules: row.try_get("enable_error_return_rules").map_sql_err()?,
        enable_billing_reservation: row.try_get("enable_billing_reservation").map_sql_err()?,
        enable_referral_ledger: row.try_get("enable_referral_ledger").map_sql_err()?,
        is_active: row.try_get("is_active").map_sql_err()?,
        config: json_from_string(
            row.try_get("config").map_sql_err()?,
            "niffler_runtime_rollout_settings.config",
        )?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_capability_row(
    row: &SqliteRow,
) -> Result<StoredNifflerUpstreamServiceCapability, DataLayerError> {
    let protocol_kind: String = row.try_get("protocol_kind").map_sql_err()?;
    let capability_kind: String = row.try_get("capability_kind").map_sql_err()?;
    Ok(StoredNifflerUpstreamServiceCapability {
        id: row.try_get("id").map_sql_err()?,
        upstream_service_id: row.try_get("upstream_service_id").map_sql_err()?,
        protocol_kind: NifflerProtocolKind::from_database(&protocol_kind)?,
        capability_kind: NifflerServiceCapabilityKind::from_database(&capability_kind)?,
        is_enabled: row.try_get("is_enabled").map_sql_err()?,
        config: json_from_string(
            row.try_get("config").map_sql_err()?,
            "niffler_upstream_service_capabilities.config",
        )?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_error_return_setting_row(
    row: &SqliteRow,
) -> Result<StoredNifflerErrorReturnSetting, DataLayerError> {
    let scope: String = row.try_get("scope").map_sql_err()?;
    let handling_step: Option<String> = row.try_get("handling_step").map_sql_err()?;
    let response_mode: String = row.try_get("response_mode").map_sql_err()?;
    let account_protection_action: String =
        row.try_get("account_protection_action").map_sql_err()?;
    let pause_duration: Option<String> = row.try_get("pause_duration").map_sql_err()?;
    let match_status_code = row
        .try_get::<Option<i32>, _>("match_status_code")
        .map_sql_err()?
        .map(|value| {
            u16::try_from(value).map_err(|_| {
                DataLayerError::UnexpectedValue(format!(
                    "match_status_code is outside u16 range: {value}"
                ))
            })
        })
        .transpose()?;
    Ok(StoredNifflerErrorReturnSetting {
        id: row.try_get("id").map_sql_err()?,
        scope: NifflerErrorResponseScope::from_database(&scope)?,
        upstream_service_id: row.try_get("upstream_service_id").map_sql_err()?,
        match_status_code,
        match_text: row.try_get("match_text").map_sql_err()?,
        handling_step: handling_step
            .as_deref()
            .map(NifflerUpstreamErrorHandlingStep::from_database)
            .transpose()?,
        response_mode: NifflerUserResponseMode::from_database(&response_mode)?,
        user_message: row.try_get("user_message").map_sql_err()?,
        account_protection_action: NifflerAccountProtectionAction::from_database(
            &account_protection_action,
        )?,
        pause_duration: pause_duration
            .as_deref()
            .map(NifflerPauseDuration::from_database)
            .transpose()?,
        is_active: row.try_get("is_active").map_sql_err()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_account_risk_event_row(
    row: &SqliteRow,
) -> Result<StoredNifflerAccountRiskEvent, DataLayerError> {
    let action: String = row.try_get("action").map_sql_err()?;
    let upstream_status_code = row
        .try_get::<Option<i32>, _>("upstream_status_code")
        .map_sql_err()?
        .map(|value| {
            u16::try_from(value).map_err(|_| {
                DataLayerError::UnexpectedValue(format!(
                    "upstream_status_code is outside u16 range: {value}"
                ))
            })
        })
        .transpose()?;
    Ok(StoredNifflerAccountRiskEvent {
        id: row.try_get("id").map_sql_err()?,
        upstream_service_id: row.try_get("upstream_service_id").map_sql_err()?,
        upstream_account_id: row.try_get("upstream_account_id").map_sql_err()?,
        request_id: row.try_get("request_id").map_sql_err()?,
        user_id: row.try_get("user_id").map_sql_err()?,
        api_key_id: row.try_get("api_key_id").map_sql_err()?,
        model_name: row.try_get("model_name").map_sql_err()?,
        rule_id: row.try_get("rule_id").map_sql_err()?,
        matched_text: row.try_get("matched_text").map_sql_err()?,
        upstream_status_code,
        action: NifflerAccountProtectionAction::from_database(&action)?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
    })
}

fn map_settlement_snapshot_row(
    row: &SqliteRow,
) -> Result<StoredNifflerSettlementSnapshot, DataLayerError> {
    Ok(StoredNifflerSettlementSnapshot {
        id: row.try_get("id").map_sql_err()?,
        request_id: row.try_get("request_id").map_sql_err()?,
        user_id: row.try_get("user_id").map_sql_err()?,
        api_key_id: row.try_get("api_key_id").map_sql_err()?,
        product_plan_id: row.try_get("product_plan_id").map_sql_err()?,
        upstream_service_id: row.try_get("upstream_service_id").map_sql_err()?,
        upstream_account_id: row.try_get("upstream_account_id").map_sql_err()?,
        requested_model_name: row.try_get("requested_model_name").map_sql_err()?,
        upstream_execution_model_name: row
            .try_get("upstream_execution_model_name")
            .map_sql_err()?,
        image_tool_model_name: row.try_get("image_tool_model_name").map_sql_err()?,
        pricing_snapshot: json_from_string(
            Some(row.try_get("pricing_snapshot").map_sql_err()?),
            "niffler_settlement_snapshots.pricing_snapshot",
        )?
        .ok_or_else(|| {
            DataLayerError::UnexpectedValue(
                "niffler_settlement_snapshots.pricing_snapshot is null".to_string(),
            )
        })?,
        wallet_charge_usd: row.try_get("wallet_charge_usd").map_sql_err()?,
        entitlement_charge_usd: row.try_get("entitlement_charge_usd").map_sql_err()?,
        upstream_cost_usd: row.try_get("upstream_cost_usd").map_sql_err()?,
        gross_margin_usd: row.try_get("gross_margin_usd").map_sql_err()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        finalized_at_unix_ms: row
            .try_get::<Option<i64>, _>("finalized_at_unix_ms")
            .map_sql_err()?
            .map(|value| super::u64_from_i64(value, "finalized_at_unix_ms"))
            .transpose()?,
    })
}

fn map_settlement_snapshot_list_row(
    row: &SqliteRow,
) -> Result<StoredNifflerSettlementSnapshotListItem, DataLayerError> {
    Ok(StoredNifflerSettlementSnapshotListItem {
        id: row.try_get("id").map_sql_err()?,
        request_id: row.try_get("request_id").map_sql_err()?,
        user_id: row.try_get("user_id").map_sql_err()?,
        api_key_id: row.try_get("api_key_id").map_sql_err()?,
        product_plan_id: row.try_get("product_plan_id").map_sql_err()?,
        product_plan_name: row.try_get("product_plan_name").map_sql_err()?,
        upstream_service_id: row.try_get("upstream_service_id").map_sql_err()?,
        upstream_service_name: row.try_get("upstream_service_name").map_sql_err()?,
        upstream_account_id: row.try_get("upstream_account_id").map_sql_err()?,
        upstream_account_display_name: row
            .try_get("upstream_account_display_name")
            .map_sql_err()?,
        upstream_account_email: row.try_get("upstream_account_email").map_sql_err()?,
        upstream_account_phone: row.try_get("upstream_account_phone").map_sql_err()?,
        requested_model_name: row.try_get("requested_model_name").map_sql_err()?,
        upstream_execution_model_name: row
            .try_get("upstream_execution_model_name")
            .map_sql_err()?,
        image_tool_model_name: row.try_get("image_tool_model_name").map_sql_err()?,
        pricing_snapshot: json_from_string(
            Some(row.try_get("pricing_snapshot").map_sql_err()?),
            "niffler_settlement_snapshots.pricing_snapshot",
        )?
        .ok_or_else(|| {
            DataLayerError::UnexpectedValue(
                "niffler_settlement_snapshots.pricing_snapshot is null".to_string(),
            )
        })?,
        wallet_charge_usd: row.try_get("wallet_charge_usd").map_sql_err()?,
        entitlement_charge_usd: row.try_get("entitlement_charge_usd").map_sql_err()?,
        upstream_cost_usd: row.try_get("upstream_cost_usd").map_sql_err()?,
        gross_margin_usd: row.try_get("gross_margin_usd").map_sql_err()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        finalized_at_unix_ms: row
            .try_get::<Option<i64>, _>("finalized_at_unix_ms")
            .map_sql_err()?
            .map(|value| super::u64_from_i64(value, "finalized_at_unix_ms"))
            .transpose()?,
    })
}

fn map_consistency_check_row(
    row: &SqliteRow,
) -> Result<StoredNifflerConsistencyCheckItem, DataLayerError> {
    let reservation_status = row
        .try_get::<Option<String>, _>("reservation_status")
        .map_sql_err()?
        .as_deref()
        .map(NifflerBillingReservationStatus::from_database)
        .transpose()?;
    let usage_status: Option<String> = row
        .try_get::<Option<String>, _>("usage_status")
        .map_sql_err()?;
    let usage_billing_status: Option<String> = row
        .try_get::<Option<String>, _>("usage_billing_status")
        .map_sql_err()?;
    let legacy_wallet_charge_usd: Option<f64> = row
        .try_get::<Option<f64>, _>("legacy_wallet_charge_usd")
        .map_sql_err()?;
    let legacy_entitlement_charge_usd: f64 = row
        .try_get::<f64, _>("legacy_entitlement_charge_usd")
        .map_sql_err()?;
    let niffler_wallet_charge_usd: f64 = row
        .try_get::<f64, _>("niffler_wallet_charge_usd")
        .map_sql_err()?;
    let niffler_entitlement_charge_usd: f64 = row
        .try_get::<f64, _>("niffler_entitlement_charge_usd")
        .map_sql_err()?;
    let niffler_total_charge_usd: f64 = row
        .try_get::<f64, _>("niffler_total_charge_usd")
        .map_sql_err()?;
    let route_attempt_count = i64_to_u64(
        row.try_get::<i64, _>("route_attempt_count").map_sql_err()?,
        "route_attempt_count",
    )?;
    let successful_route_attempt_count = i64_to_u64(
        row.try_get::<i64, _>("successful_route_attempt_count")
            .map_sql_err()?,
        "successful_route_attempt_count",
    )?;
    let wallet_difference_usd =
        legacy_wallet_charge_usd.map(|legacy| niffler_wallet_charge_usd - legacy);
    let entitlement_difference_usd = niffler_entitlement_charge_usd - legacy_entitlement_charge_usd;
    let total_difference_usd = legacy_wallet_charge_usd.map(|legacy_wallet| {
        niffler_total_charge_usd - legacy_wallet - legacy_entitlement_charge_usd
    });
    let issue_codes = consistency_issue_codes(ConsistencyIssueInput {
        usage_status: usage_status.as_deref(),
        usage_billing_status: usage_billing_status.as_deref(),
        legacy_wallet_charge_usd,
        wallet_difference_usd,
        entitlement_difference_usd,
        total_difference_usd,
        reservation_status,
        route_attempt_count,
    });
    let consistency_status = if issue_codes.is_empty() {
        "ok".to_string()
    } else {
        "needs_review".to_string()
    };
    Ok(StoredNifflerConsistencyCheckItem {
        request_id: row.try_get::<String, _>("request_id").map_sql_err()?,
        user_id: row.try_get::<Option<String>, _>("user_id").map_sql_err()?,
        api_key_id: row
            .try_get::<Option<String>, _>("api_key_id")
            .map_sql_err()?,
        product_plan_id: row
            .try_get::<Option<String>, _>("product_plan_id")
            .map_sql_err()?,
        product_plan_name: row
            .try_get::<Option<String>, _>("product_plan_name")
            .map_sql_err()?,
        usage_status,
        usage_billing_status,
        usage_total_cost_usd: row
            .try_get::<Option<f64>, _>("usage_total_cost_usd")
            .map_sql_err()?,
        legacy_wallet_charge_usd,
        legacy_entitlement_charge_usd,
        niffler_wallet_charge_usd,
        niffler_entitlement_charge_usd,
        niffler_total_charge_usd,
        wallet_difference_usd,
        entitlement_difference_usd,
        total_difference_usd,
        reservation_id: row
            .try_get::<Option<String>, _>("reservation_id")
            .map_sql_err()?,
        reservation_status,
        reservation_release_reason: row
            .try_get::<Option<String>, _>("reservation_release_reason")
            .map_sql_err()?,
        route_attempt_count,
        successful_route_attempt_count,
        issue_codes,
        consistency_status,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get::<i64, _>("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
    })
}

const CONSISTENCY_TOLERANCE_USD: f64 = 0.000_001;

struct ConsistencyIssueInput<'a> {
    usage_status: Option<&'a str>,
    usage_billing_status: Option<&'a str>,
    legacy_wallet_charge_usd: Option<f64>,
    wallet_difference_usd: Option<f64>,
    entitlement_difference_usd: f64,
    total_difference_usd: Option<f64>,
    reservation_status: Option<NifflerBillingReservationStatus>,
    route_attempt_count: u64,
}

fn consistency_issue_codes(input: ConsistencyIssueInput<'_>) -> Vec<String> {
    let mut issues = Vec::new();
    if input.usage_status.is_none() {
        issues.push("missing_legacy_usage".to_string());
    }
    if input.usage_billing_status.is_none() {
        issues.push("missing_legacy_settlement".to_string());
    } else if input.usage_billing_status != Some("settled") {
        issues.push("legacy_not_settled".to_string());
    }
    if input.legacy_wallet_charge_usd.is_none() {
        issues.push("missing_legacy_wallet_charge".to_string());
    }
    if input
        .wallet_difference_usd
        .is_some_and(|value| value.abs() > CONSISTENCY_TOLERANCE_USD)
    {
        issues.push("wallet_charge_mismatch".to_string());
    }
    if input.entitlement_difference_usd.abs() > CONSISTENCY_TOLERANCE_USD {
        issues.push("entitlement_charge_mismatch".to_string());
    }
    if input
        .total_difference_usd
        .is_some_and(|value| value.abs() > CONSISTENCY_TOLERANCE_USD)
    {
        issues.push("total_charge_mismatch".to_string());
    }
    match input.reservation_status {
        None => issues.push("missing_billing_reservation".to_string()),
        Some(NifflerBillingReservationStatus::Active) => {
            issues.push("reservation_not_finalized".to_string())
        }
        Some(NifflerBillingReservationStatus::ManualReview) => {
            issues.push("reservation_manual_review".to_string())
        }
        Some(NifflerBillingReservationStatus::Settled)
        | Some(NifflerBillingReservationStatus::Released)
        | Some(NifflerBillingReservationStatus::Expired) => {}
    }
    if input.route_attempt_count == 0 {
        issues.push("missing_route_attempt".to_string());
    }
    issues
}

fn map_stability_observation_row(
    row: &SqliteRow,
) -> Result<StoredNifflerStabilityObservation, DataLayerError> {
    let blocker_codes_raw = row.try_get::<String, _>("blocker_codes").map_sql_err()?;
    let blocker_codes_value = json_from_string(
        Some(blocker_codes_raw),
        "niffler_stability_observations.blocker_codes",
    )?
    .ok_or_else(|| {
        DataLayerError::UnexpectedValue(
            "niffler stability blocker_codes cannot be null".to_string(),
        )
    })?;
    let observation = StoredNifflerStabilityObservation {
        id: row.try_get("id").map_sql_err()?,
        window_start_unix_ms: super::u64_from_i64(
            row.try_get("window_start_unix_ms").map_sql_err()?,
            "window_start_unix_ms",
        )?,
        window_end_unix_ms: super::u64_from_i64(
            row.try_get("window_end_unix_ms").map_sql_err()?,
            "window_end_unix_ms",
        )?,
        status: row.try_get("status").map_sql_err()?,
        rollback_drill_status: row.try_get("rollback_drill_status").map_sql_err()?,
        consistency_checked_count: super::u64_from_i64(
            row.try_get("consistency_checked_count").map_sql_err()?,
            "consistency_checked_count",
        )?,
        consistency_issue_count: super::u64_from_i64(
            row.try_get("consistency_issue_count").map_sql_err()?,
            "consistency_issue_count",
        )?,
        unknown_upstream_count: super::u64_from_i64(
            row.try_get("unknown_upstream_count").map_sql_err()?,
            "unknown_upstream_count",
        )?,
        legacy_write_call_count: super::u64_from_i64(
            row.try_get("legacy_write_call_count").map_sql_err()?,
            "legacy_write_call_count",
        )?,
        billing_reservation_exception_count: super::u64_from_i64(
            row.try_get("billing_reservation_exception_count")
                .map_sql_err()?,
            "billing_reservation_exception_count",
        )?,
        referral_exception_count: super::u64_from_i64(
            row.try_get("referral_exception_count").map_sql_err()?,
            "referral_exception_count",
        )?,
        blocker_codes: parse_stability_blocker_codes(blocker_codes_value)?,
        summary: json_from_string(
            row.try_get::<Option<String>, _>("summary").map_sql_err()?,
            "niffler_stability_observations.summary",
        )?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    };
    observation.validate()?;
    Ok(observation)
}

fn parse_stability_blocker_codes(value: serde_json::Value) -> Result<Vec<String>, DataLayerError> {
    let Some(items) = value.as_array() else {
        return Err(DataLayerError::UnexpectedValue(
            "niffler stability blocker_codes must be an array".to_string(),
        ));
    };
    Ok(items
        .iter()
        .filter_map(|item| item.as_str().map(ToOwned::to_owned))
        .collect())
}

fn i64_to_u64(value: i64, field: &str) -> Result<u64, DataLayerError> {
    u64::try_from(value)
        .map_err(|_| DataLayerError::UnexpectedValue(format!("{field} is negative: {value}")))
}

fn map_billing_reservation_row(
    row: &SqliteRow,
) -> Result<StoredNifflerBillingReservation, DataLayerError> {
    let status: String = row.try_get("status").map_sql_err()?;
    Ok(StoredNifflerBillingReservation {
        id: row.try_get("id").map_sql_err()?,
        request_id: row.try_get("request_id").map_sql_err()?,
        user_id: row.try_get("user_id").map_sql_err()?,
        api_key_id: row.try_get("api_key_id").map_sql_err()?,
        product_plan_id: row.try_get("product_plan_id").map_sql_err()?,
        status: NifflerBillingReservationStatus::from_database(&status)?,
        reserved_total_usd: row.try_get("reserved_total_usd").map_sql_err()?,
        wallet_reserved_usd: row.try_get("wallet_reserved_usd").map_sql_err()?,
        entitlement_reserved_usd: row.try_get("entitlement_reserved_usd").map_sql_err()?,
        reserved_at_unix_ms: super::u64_from_i64(
            row.try_get("reserved_at_unix_ms").map_sql_err()?,
            "reserved_at_unix_ms",
        )?,
        expires_at_unix_ms: super::u64_from_i64(
            row.try_get("expires_at_unix_ms").map_sql_err()?,
            "expires_at_unix_ms",
        )?,
        finalized_at_unix_ms: row
            .try_get::<Option<i64>, _>("finalized_at_unix_ms")
            .map_sql_err()?
            .map(|value| super::u64_from_i64(value, "finalized_at_unix_ms"))
            .transpose()?,
        settlement_snapshot_id: row.try_get("settlement_snapshot_id").map_sql_err()?,
        release_reason: row.try_get("release_reason").map_sql_err()?,
        idempotency_key: row.try_get("idempotency_key").map_sql_err()?,
    })
}

fn map_billing_reservation_dry_run_row(
    row: &SqliteRow,
) -> Result<StoredNifflerBillingReservationDryRun, DataLayerError> {
    Ok(StoredNifflerBillingReservationDryRun {
        id: row.try_get("id").map_sql_err()?,
        request_id: row.try_get("request_id").map_sql_err()?,
        user_id: row.try_get("user_id").map_sql_err()?,
        api_key_id: row.try_get("api_key_id").map_sql_err()?,
        product_plan_id: row.try_get("product_plan_id").map_sql_err()?,
        requested_model_name: row.try_get("requested_model_name").map_sql_err()?,
        estimated_reservation_usd: row.try_get("estimated_reservation_usd").map_sql_err()?,
        legacy_final_charge_usd: row.try_get("legacy_final_charge_usd").map_sql_err()?,
        difference_usd: row.try_get("difference_usd").map_sql_err()?,
        estimation_source: row.try_get("estimation_source").map_sql_err()?,
        status: row.try_get("status").map_sql_err()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        finalized_at_unix_ms: row
            .try_get::<Option<i64>, _>("finalized_at_unix_ms")
            .map_sql_err()?
            .map(|value| super::u64_from_i64(value, "finalized_at_unix_ms"))
            .transpose()?,
    })
}

fn map_referral_reward_ledger_row(
    row: &SqliteRow,
) -> Result<StoredNifflerReferralRewardLedger, DataLayerError> {
    let status: String = row.try_get("status").map_sql_err()?;
    let retry_count: i32 = row.try_get("retry_count").map_sql_err()?;
    Ok(StoredNifflerReferralRewardLedger {
        id: row.try_get("id").map_sql_err()?,
        order_id: row.try_get("order_id").map_sql_err()?,
        idempotency_key: row.try_get("idempotency_key").map_sql_err()?,
        inviter_user_id: row.try_get("inviter_user_id").map_sql_err()?,
        invitee_user_id: row.try_get("invitee_user_id").map_sql_err()?,
        rule_id: row.try_get("rule_id").map_sql_err()?,
        reward_amount_usd: row.try_get("reward_amount_usd").map_sql_err()?,
        rule_snapshot: json_from_string(
            Some(row.try_get("rule_snapshot").map_sql_err()?),
            "niffler_referral_reward_ledger.rule_snapshot",
        )?
        .ok_or_else(|| {
            DataLayerError::UnexpectedValue(
                "niffler_referral_reward_ledger.rule_snapshot is null".to_string(),
            )
        })?,
        status: NifflerReferralRewardLedgerStatus::from_database(&status)?,
        failure_reason: row.try_get("failure_reason").map_sql_err()?,
        retry_count: u32::try_from(retry_count).map_err(|_| {
            DataLayerError::UnexpectedValue(format!(
                "referral reward retry_count is negative: {retry_count}"
            ))
        })?,
        paid_at_unix_ms: row
            .try_get::<Option<i64>, _>("paid_at_unix_ms")
            .map_sql_err()?
            .map(|value| super::u64_from_i64(value, "paid_at_unix_ms"))
            .transpose()?,
        cancelled_at_unix_ms: row
            .try_get::<Option<i64>, _>("cancelled_at_unix_ms")
            .map_sql_err()?
            .map(|value| super::u64_from_i64(value, "cancelled_at_unix_ms"))
            .transpose()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_route_attempt_row(
    row: &SqliteRow,
) -> Result<StoredNifflerRouteAttemptListItem, DataLayerError> {
    let attempt_index: i32 = row.try_get("attempt_index").map_sql_err()?;
    let upstream_status_code = row
        .try_get::<Option<i32>, _>("upstream_status_code")
        .map_sql_err()?
        .map(|value| {
            u16::try_from(value).map_err(|_| {
                DataLayerError::UnexpectedValue(format!(
                    "upstream_status_code is outside u16 range: {value}"
                ))
            })
        })
        .transpose()?;
    Ok(StoredNifflerRouteAttemptListItem {
        id: row.try_get("id").map_sql_err()?,
        request_id: row.try_get("request_id").map_sql_err()?,
        upstream_service_id: row.try_get("upstream_service_id").map_sql_err()?,
        upstream_service_name: row.try_get("upstream_service_name").map_sql_err()?,
        upstream_account_id: row.try_get("upstream_account_id").map_sql_err()?,
        upstream_account_display_name: row
            .try_get("upstream_account_display_name")
            .map_sql_err()?,
        upstream_account_email: row.try_get("upstream_account_email").map_sql_err()?,
        upstream_account_phone: row.try_get("upstream_account_phone").map_sql_err()?,
        product_plan_id: row.try_get("product_plan_id").map_sql_err()?,
        product_plan_name: row.try_get("product_plan_name").map_sql_err()?,
        model_name: row.try_get("model_name").map_sql_err()?,
        attempt_index: u32::try_from(attempt_index).map_err(|_| {
            DataLayerError::UnexpectedValue(format!(
                "route attempt_index is negative: {attempt_index}"
            ))
        })?,
        status: row.try_get("status").map_sql_err()?,
        skip_reason: row.try_get("skip_reason").map_sql_err()?,
        upstream_status_code,
        latency_ms: row
            .try_get::<Option<i64>, _>("latency_ms")
            .map_sql_err()?
            .map(|value| super::u64_from_i64(value, "latency_ms"))
            .transpose()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::migrate::run_sqlite_migrations;
    use sqlx::Row;

    async fn test_repository() -> SqliteNifflerCoreRepository {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("sqlite pool should connect");
        run_sqlite_migrations(&pool)
            .await
            .expect("sqlite migrations should run");
        SqliteNifflerCoreRepository::new(pool)
    }

    fn reservation_record(
        request_id: &str,
        wallet_reserved_usd: f64,
        expires_at_unix_ms: u64,
    ) -> CreateNifflerBillingReservationRecord {
        CreateNifflerBillingReservationRecord {
            id: format!("reservation-{request_id}"),
            request_id: request_id.to_string(),
            user_id: Some("user-1".to_string()),
            api_key_id: Some("key-1".to_string()),
            product_plan_id: Some("plan-1".to_string()),
            reserved_total_usd: wallet_reserved_usd,
            wallet_reserved_usd,
            entitlement_reserved_usd: 0.0,
            reserved_at_unix_ms: 1_000,
            expires_at_unix_ms,
            idempotency_key: format!("reservation-idempotency-{request_id}"),
            event_id: format!("reserved-event-{request_id}"),
            event_idempotency_key: format!("reserved-event-idempotency-{request_id}"),
            actor_id: Some("test".to_string()),
        }
    }

    fn service_record(id: &str, is_active: bool) -> CreateNifflerUpstreamServiceRecord {
        CreateNifflerUpstreamServiceRecord {
            id: id.to_string(),
            display_name: id.to_string(),
            service_kind: "openai_compatible".to_string(),
            default_api_format: Some("openai:chat".to_string()),
            base_url: Some(format!("https://{id}.example.com")),
            cost_multiplier: 1.0,
            is_active,
            config: None,
            created_at_unix_ms: 1_000,
            updated_at_unix_ms: 1_000,
        }
    }

    fn account_record(
        id: &str,
        upstream_service_id: &str,
        status: NifflerAccountStatus,
        cooldown_until_unix_ms: Option<u64>,
    ) -> CreateNifflerUpstreamAccountRecord {
        CreateNifflerUpstreamAccountRecord {
            id: id.to_string(),
            upstream_service_id: upstream_service_id.to_string(),
            display_name: id.to_string(),
            email: None,
            phone: None,
            auth_kind: "api_key".to_string(),
            status,
            cost_multiplier: 1.0,
            priority: 0,
            cooldown_until_unix_ms,
            last_tested_at_unix_ms: None,
            last_test_error: None,
            config: None,
            created_at_unix_ms: 1_000,
            updated_at_unix_ms: 1_000,
        }
    }

    async fn insert_account_model_capability(
        repository: &SqliteNifflerCoreRepository,
        id: &str,
        upstream_service_id: &str,
        upstream_account_id: &str,
        model_name: &str,
        is_enabled: bool,
    ) {
        sqlx::query(
            r#"
INSERT INTO niffler_account_model_capabilities (
  id, upstream_service_id, upstream_account_id, model_name, is_enabled,
  source, last_checked_at_unix_ms, last_error, created_at_unix_ms, updated_at_unix_ms
)
VALUES (?, ?, ?, ?, ?, NULL, NULL, NULL, ?, ?)
"#,
        )
        .bind(id)
        .bind(upstream_service_id)
        .bind(upstream_account_id)
        .bind(model_name)
        .bind(is_enabled)
        .bind(1_000_i64)
        .bind(1_000_i64)
        .execute(&repository.pool)
        .await
        .expect("account model capability should insert");
    }

    #[tokio::test]
    async fn sqlite_runtime_account_model_access_only_returns_schedulable_accounts() {
        let repository = test_repository().await;
        repository
            .create_upstream_service(service_record("service-active", true))
            .await
            .expect("active service should insert");
        repository
            .create_upstream_service(service_record("service-disabled", false))
            .await
            .expect("disabled service should insert");
        for record in [
            account_record(
                "account-ok",
                "service-active",
                NifflerAccountStatus::Available,
                None,
            ),
            account_record(
                "account-invalid",
                "service-active",
                NifflerAccountStatus::Invalid,
                None,
            ),
            account_record(
                "account-cooling",
                "service-active",
                NifflerAccountStatus::Available,
                Some(3_000),
            ),
            account_record(
                "account-disabled-capability",
                "service-active",
                NifflerAccountStatus::Available,
                None,
            ),
            account_record(
                "account-disabled-service",
                "service-disabled",
                NifflerAccountStatus::Available,
                None,
            ),
        ] {
            repository
                .create_upstream_account(record)
                .await
                .expect("account should insert");
        }
        for (id, service_id, account_id, is_enabled) in [
            ("cap-ok", "service-active", "account-ok", true),
            (
                "cap-disabled",
                "service-active",
                "account-disabled-capability",
                false,
            ),
            ("cap-invalid", "service-active", "account-invalid", true),
            ("cap-cooling", "service-active", "account-cooling", true),
            (
                "cap-disabled-service",
                "service-disabled",
                "account-disabled-service",
                true,
            ),
        ] {
            insert_account_model_capability(
                &repository,
                id,
                service_id,
                account_id,
                "gpt-5",
                is_enabled,
            )
            .await;
        }

        let page = repository
            .list_runtime_account_model_access(&NifflerRuntimeAccountModelAccessListQuery {
                model_name: "gpt-5".to_string(),
                now_unix_ms: 2_000,
                offset: 0,
                limit: 10,
            })
            .await
            .expect("runtime access query should succeed");

        assert_eq!(page.total, 1);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].upstream_service_id, "service-active");
        assert_eq!(page.items[0].upstream_account_id, "account-ok");
    }

    #[tokio::test]
    async fn sqlite_consistency_checks_report_ok_for_matched_request() {
        let repository = test_repository().await;
        sqlx::query(
            r#"
INSERT INTO "usage" (
  request_id, user_id, api_key_id, status, billing_status, total_cost_usd,
  wallet_recharge_balance_before, wallet_recharge_balance_after,
  wallet_gift_balance_before, wallet_gift_balance_after,
  created_at_unix_ms, updated_at_unix_secs
)
VALUES (?, ?, ?, 'completed', 'settled', 0.5, 10.0, 9.5, 0.0, 0.0, 1000, 1)
"#,
        )
        .bind("request-matched")
        .bind("user-1")
        .bind("key-1")
        .execute(&repository.pool)
        .await
        .expect("usage row should insert");
        sqlx::query(
            r#"
INSERT INTO usage_settlement_snapshots (
  request_id, billing_status, wallet_recharge_balance_before,
  wallet_recharge_balance_after, wallet_gift_balance_before,
  wallet_gift_balance_after, created_at, updated_at
)
VALUES (?, 'settled', 10.0, 9.5, 0.0, 0.0, 1, 1)
"#,
        )
        .bind("request-matched")
        .execute(&repository.pool)
        .await
        .expect("usage settlement snapshot should insert");

        repository
            .create_billing_reservation(reservation_record("request-matched", 0.5, 3_000))
            .await
            .expect("reservation should create");
        let snapshot = repository
            .create_settlement_snapshot(CreateNifflerSettlementSnapshotRecord {
                id: "snapshot-request-matched".to_string(),
                request_id: "request-matched".to_string(),
                user_id: Some("user-1".to_string()),
                api_key_id: Some("key-1".to_string()),
                product_plan_id: Some("plan-1".to_string()),
                upstream_service_id: None,
                upstream_account_id: None,
                requested_model_name: "gpt-test".to_string(),
                upstream_execution_model_name: None,
                image_tool_model_name: None,
                pricing_snapshot: serde_json::json!({"source": "test"}),
                wallet_charge_usd: 0.5,
                entitlement_charge_usd: 0.0,
                upstream_cost_usd: 0.2,
                gross_margin_usd: 0.3,
                created_at_unix_ms: 1_000,
                finalized_at_unix_ms: Some(1_100),
            })
            .await
            .expect("settlement snapshot should create");
        repository
            .finalize_billing_reservation_by_request_id(FinalizeNifflerBillingReservationRecord {
                request_id: "request-matched".to_string(),
                status: NifflerBillingReservationStatus::Settled,
                finalized_at_unix_ms: 1_100,
                settlement_snapshot_id: Some(snapshot.id),
                release_reason: None,
                event_id: "settled-event-request-matched".to_string(),
                event_idempotency_key: "settled-event-idempotency-request-matched".to_string(),
                actor_id: Some("system".to_string()),
            })
            .await
            .expect("reservation should finalize");
        repository
            .create_route_attempt(CreateNifflerRouteAttemptRecord {
                id: "route-attempt-request-matched".to_string(),
                request_id: "request-matched".to_string(),
                upstream_service_id: None,
                upstream_account_id: None,
                product_plan_id: Some("plan-1".to_string()),
                model_name: "gpt-test".to_string(),
                attempt_index: 0,
                status: "success".to_string(),
                skip_reason: None,
                upstream_status_code: Some(200),
                latency_ms: Some(120),
                created_at_unix_ms: 1_050,
            })
            .await
            .expect("route attempt should create");

        let page = repository
            .list_consistency_checks(&NifflerConsistencyCheckListQuery {
                product_plan_id: Some("plan-1".to_string()),
                limit: 10,
                ..Default::default()
            })
            .await
            .expect("consistency checks should list");

        assert_eq!(page.total, 1);
        assert_eq!(page.items.len(), 1);
        let item = &page.items[0];
        assert_eq!(item.request_id, "request-matched");
        assert_eq!(item.consistency_status, "ok");
        assert!(item.issue_codes.is_empty());
        assert_eq!(item.route_attempt_count, 1);
        assert_eq!(item.successful_route_attempt_count, 1);
        assert_eq!(
            item.reservation_status,
            Some(NifflerBillingReservationStatus::Settled)
        );
        assert_eq!(item.wallet_difference_usd, Some(0.0));
        assert_eq!(item.total_difference_usd, Some(0.0));
    }

    #[tokio::test]
    async fn sqlite_billing_reservation_create_sum_filter_and_finalize() {
        let repository = test_repository().await;
        repository
            .create_billing_reservation(reservation_record("request-expired", 1.25, 2_000))
            .await
            .expect("expired reservation should create");
        repository
            .create_billing_reservation(reservation_record("request-open", 2.5, 4_000))
            .await
            .expect("open reservation should create");

        let active_before_expiry = repository
            .sum_active_billing_reservation_wallet_usd(Some("user-1"), Some("key-1"), 1_500)
            .await
            .expect("active sum should read");
        assert!((active_before_expiry - 3.75).abs() < f64::EPSILON);

        let expired_page = repository
            .list_billing_reservations(&NifflerBillingReservationListQuery {
                status: Some(NifflerBillingReservationStatus::Active),
                user_id: None,
                api_key_id: None,
                request_id: None,
                expires_at_lte_unix_ms: Some(2_500),
                offset: 0,
                limit: 10,
            })
            .await
            .expect("expired reservations should list");
        assert_eq!(expired_page.items.len(), 1);
        assert_eq!(expired_page.items[0].request_id, "request-expired");

        let finalized = repository
            .finalize_billing_reservation_by_request_id(FinalizeNifflerBillingReservationRecord {
                request_id: "request-expired".to_string(),
                status: NifflerBillingReservationStatus::Expired,
                finalized_at_unix_ms: 2_500,
                settlement_snapshot_id: None,
                release_reason: Some("reservation_expired".to_string()),
                event_id: "expired-event-request-expired".to_string(),
                event_idempotency_key: "expired-event-idempotency-request-expired".to_string(),
                actor_id: Some("system".to_string()),
            })
            .await
            .expect("reservation should finalize")
            .expect("reservation should exist");
        assert_eq!(finalized.status, NifflerBillingReservationStatus::Expired);

        let active_after_finalize = repository
            .sum_active_billing_reservation_wallet_usd(Some("user-1"), Some("key-1"), 1_500)
            .await
            .expect("active sum should read after finalize");
        assert!((active_after_finalize - 2.5).abs() < f64::EPSILON);

        let repeated = repository
            .finalize_billing_reservation_by_request_id(FinalizeNifflerBillingReservationRecord {
                request_id: "request-expired".to_string(),
                status: NifflerBillingReservationStatus::Released,
                finalized_at_unix_ms: 3_000,
                settlement_snapshot_id: None,
                release_reason: Some("late_release_should_not_overwrite".to_string()),
                event_id: "released-event-request-expired".to_string(),
                event_idempotency_key: "released-event-idempotency-request-expired".to_string(),
                actor_id: Some("system".to_string()),
            })
            .await
            .expect("repeat finalize should be idempotent")
            .expect("reservation should still exist");
        assert_eq!(repeated.status, NifflerBillingReservationStatus::Expired);
        assert_eq!(
            repeated.release_reason.as_deref(),
            Some("reservation_expired")
        );

        let expired_events: i64 = sqlx::query(
            "SELECT COUNT(*) AS count FROM niffler_billing_reservation_events WHERE event_kind = 'expired'",
        )
        .fetch_one(&repository.pool)
        .await
        .expect("expired event count should read")
        .try_get("count")
        .expect("count should decode");
        assert_eq!(expired_events, 1);
    }

    #[tokio::test]
    async fn sqlite_stability_observation_upsert_is_window_idempotent() {
        let repository = test_repository().await;
        let first = repository
            .upsert_stability_observation(UpsertNifflerStabilityObservationRecord {
                id: "stability-1".to_string(),
                window_start_unix_ms: 1_000,
                window_end_unix_ms: 86_401_000,
                status: "pending".to_string(),
                rollback_drill_status: "not_recorded".to_string(),
                consistency_checked_count: 10,
                consistency_issue_count: 0,
                unknown_upstream_count: 0,
                legacy_write_call_count: 0,
                billing_reservation_exception_count: 0,
                referral_exception_count: 0,
                blocker_codes: vec!["rollback_drill_not_recorded".to_string()],
                summary: Some(serde_json::json!({"source": "test"})),
                created_at_unix_ms: 86_401_000,
                updated_at_unix_ms: 86_401_000,
            })
            .await
            .expect("first observation should upsert");
        assert_eq!(first.status, "pending");
        assert_eq!(first.blocker_codes, vec!["rollback_drill_not_recorded"]);

        let second = repository
            .upsert_stability_observation(UpsertNifflerStabilityObservationRecord {
                id: "stability-2".to_string(),
                window_start_unix_ms: 1_000,
                window_end_unix_ms: 86_401_000,
                status: "reset_required".to_string(),
                rollback_drill_status: "failed".to_string(),
                consistency_checked_count: 11,
                consistency_issue_count: 1,
                unknown_upstream_count: 1,
                legacy_write_call_count: 0,
                billing_reservation_exception_count: 0,
                referral_exception_count: 0,
                blocker_codes: vec![
                    "consistency_issue".to_string(),
                    "unknown_upstream".to_string(),
                    "rollback_drill_failed".to_string(),
                ],
                summary: Some(serde_json::json!({"source": "updated"})),
                created_at_unix_ms: 86_401_000,
                updated_at_unix_ms: 86_402_000,
            })
            .await
            .expect("second observation should update same window");
        assert_eq!(second.id, "stability-1");
        assert_eq!(second.status, "reset_required");
        assert_eq!(second.consistency_checked_count, 11);
        assert_eq!(second.unknown_upstream_count, 1);

        let page = repository
            .list_stability_observations(&NifflerStabilityObservationListQuery {
                status: Some("reset_required".to_string()),
                offset: 0,
                limit: 10,
            })
            .await
            .expect("observations should list");
        assert_eq!(page.total, 1);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].status, "reset_required");
    }
}
