use async_trait::async_trait;
use sqlx::{sqlite::SqliteRow, QueryBuilder, Row, Sqlite};

use super::{
    bounded_limit, bounded_offset, i64_from_u64, json_from_string, json_to_string,
    CreateNifflerBillingReservationDryRunRecord, CreateNifflerErrorReturnSettingRecord,
    CreateNifflerProductPlanRecord, CreateNifflerRouteAttemptRecord,
    CreateNifflerSettlementSnapshotRecord, CreateNifflerUpstreamAccountRecord,
    CreateNifflerUpstreamServiceRecord, NifflerAccountProtectionAction, NifflerAccountStatus,
    NifflerApiKeyProductPlanBindingListQuery, NifflerBillingReservationDryRunListQuery,
    NifflerBillingReservationListQuery, NifflerBillingReservationStatus, NifflerCoreReadRepository,
    NifflerCoreWriteRepository, NifflerErrorResponseScope, NifflerErrorReturnSettingListQuery,
    NifflerPauseDuration, NifflerProductPlanListQuery, NifflerProductPlanModelListQuery,
    NifflerProtocolKind, NifflerReferralRewardLedgerListQuery, NifflerReferralRewardLedgerStatus,
    NifflerRouteAttemptListQuery, NifflerRuntimeRolloutSettingListQuery,
    NifflerRuntimeRolloutTargetScope, NifflerServiceCapabilityKind,
    NifflerSettlementSnapshotListQuery, NifflerUpstreamAccountListQuery,
    NifflerUpstreamErrorHandlingStep, NifflerUpstreamServiceCapabilityListQuery,
    NifflerUpstreamServiceListQuery, NifflerUserResponseMode,
    StoredNifflerApiKeyProductPlanBinding, StoredNifflerApiKeyProductPlanBindingListPage,
    StoredNifflerBillingReservation, StoredNifflerBillingReservationDryRun,
    StoredNifflerBillingReservationDryRunListPage, StoredNifflerBillingReservationListPage,
    StoredNifflerErrorReturnSetting, StoredNifflerErrorReturnSettingListPage,
    StoredNifflerProductPlan, StoredNifflerProductPlanListPage, StoredNifflerProductPlanModel,
    StoredNifflerProductPlanModelListPage, StoredNifflerReferralRewardLedger,
    StoredNifflerReferralRewardLedgerListPage, StoredNifflerRouteAttempt,
    StoredNifflerRouteAttemptListItem, StoredNifflerRouteAttemptListPage,
    StoredNifflerRuntimeRolloutSetting, StoredNifflerRuntimeRolloutSettingListPage,
    StoredNifflerSettlementSnapshot, StoredNifflerSettlementSnapshotListItem,
    StoredNifflerSettlementSnapshotListPage, StoredNifflerUpstreamAccount,
    StoredNifflerUpstreamAccountListPage, StoredNifflerUpstreamService,
    StoredNifflerUpstreamServiceCapability, StoredNifflerUpstreamServiceCapabilityListPage,
    StoredNifflerUpstreamServiceListPage, UpsertNifflerApiKeyProductPlanBindingRecord,
    UpsertNifflerProductPlanModelRecord, UpsertNifflerRuntimeRolloutSettingRecord,
    UpsertNifflerUpstreamServiceCapabilityRecord,
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
