use super::{AppState, GatewayError};
use aether_data_contracts::repository::niffler_core::{
    CreateNifflerAccountRiskEventRecord, CreateNifflerBillingReservationDryRunRecord,
    CreateNifflerBillingReservationRecord, CreateNifflerErrorReturnSettingRecord,
    CreateNifflerProductPlanRecord, CreateNifflerReferralRewardLedgerRecord,
    CreateNifflerRouteAttemptRecord, CreateNifflerSettlementSnapshotRecord,
    CreateNifflerUpstreamAccountRecord, CreateNifflerUpstreamServiceRecord,
    FinalizeNifflerBillingReservationRecord, NifflerApiKeyProductPlanBindingListQuery,
    NifflerBillingReservationDryRunListQuery, NifflerBillingReservationListQuery,
    NifflerConsistencyCheckListQuery, NifflerErrorReturnSettingListQuery,
    NifflerProductPlanListQuery, NifflerProductPlanModelListQuery,
    NifflerReferralRewardLedgerListQuery, NifflerRouteAttemptListQuery,
    NifflerRuntimeRolloutSettingListQuery, NifflerRuntimeRolloutTargetScope,
    NifflerSettlementSnapshotListQuery, NifflerUpstreamAccountListQuery,
    NifflerUpstreamServiceCapabilityListQuery, NifflerUpstreamServiceListQuery,
    StoredNifflerAccountRiskEvent, StoredNifflerApiKeyProductPlanBinding,
    StoredNifflerApiKeyProductPlanBindingListPage, StoredNifflerBillingReservation,
    StoredNifflerBillingReservationDryRun, StoredNifflerBillingReservationDryRunListPage,
    StoredNifflerBillingReservationListPage, StoredNifflerConsistencyCheckListPage,
    StoredNifflerErrorReturnSetting, StoredNifflerErrorReturnSettingListPage,
    StoredNifflerProductPlan, StoredNifflerProductPlanListPage, StoredNifflerProductPlanModel,
    StoredNifflerProductPlanModelListPage, StoredNifflerReferralRewardLedger,
    StoredNifflerReferralRewardLedgerListPage, StoredNifflerRouteAttempt,
    StoredNifflerRouteAttemptListPage, StoredNifflerRuntimeRolloutSetting,
    StoredNifflerRuntimeRolloutSettingListPage, StoredNifflerSettlementSnapshot,
    StoredNifflerSettlementSnapshotListPage, StoredNifflerUpstreamAccount,
    StoredNifflerUpstreamAccountListPage, StoredNifflerUpstreamService,
    StoredNifflerUpstreamServiceCapability, StoredNifflerUpstreamServiceCapabilityListPage,
    StoredNifflerUpstreamServiceListPage, UpsertNifflerApiKeyProductPlanBindingRecord,
    UpsertNifflerProductPlanModelRecord, UpsertNifflerRuntimeRolloutSettingRecord,
    UpsertNifflerUpstreamServiceCapabilityRecord,
};

impl AppState {
    pub(crate) fn has_niffler_core_reader(&self) -> bool {
        self.data.has_niffler_core_reader()
    }

    pub(crate) fn has_niffler_core_writer(&self) -> bool {
        self.data.has_niffler_core_writer()
    }

    pub(crate) async fn list_niffler_upstream_services(
        &self,
        query: &NifflerUpstreamServiceListQuery,
    ) -> Result<StoredNifflerUpstreamServiceListPage, GatewayError> {
        self.data
            .list_niffler_upstream_services(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn find_niffler_upstream_service_by_id(
        &self,
        upstream_service_id: &str,
    ) -> Result<Option<StoredNifflerUpstreamService>, GatewayError> {
        self.data
            .find_niffler_upstream_service_by_id(upstream_service_id)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_niffler_upstream_service_capabilities(
        &self,
        query: &NifflerUpstreamServiceCapabilityListQuery,
    ) -> Result<StoredNifflerUpstreamServiceCapabilityListPage, GatewayError> {
        self.data
            .list_niffler_upstream_service_capabilities(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_niffler_upstream_accounts(
        &self,
        query: &NifflerUpstreamAccountListQuery,
    ) -> Result<StoredNifflerUpstreamAccountListPage, GatewayError> {
        self.data
            .list_niffler_upstream_accounts(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn find_niffler_upstream_account_by_id(
        &self,
        upstream_account_id: &str,
    ) -> Result<Option<StoredNifflerUpstreamAccount>, GatewayError> {
        self.data
            .find_niffler_upstream_account_by_id(upstream_account_id)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_niffler_product_plans(
        &self,
        query: &NifflerProductPlanListQuery,
    ) -> Result<StoredNifflerProductPlanListPage, GatewayError> {
        self.data
            .list_niffler_product_plans(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn find_niffler_product_plan_by_id(
        &self,
        product_plan_id: &str,
    ) -> Result<Option<StoredNifflerProductPlan>, GatewayError> {
        self.data
            .find_niffler_product_plan_by_id(product_plan_id)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_niffler_product_plan_models(
        &self,
        query: &NifflerProductPlanModelListQuery,
    ) -> Result<StoredNifflerProductPlanModelListPage, GatewayError> {
        self.data
            .list_niffler_product_plan_models(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_niffler_api_key_product_plan_bindings(
        &self,
        query: &NifflerApiKeyProductPlanBindingListQuery,
    ) -> Result<StoredNifflerApiKeyProductPlanBindingListPage, GatewayError> {
        self.data
            .list_niffler_api_key_product_plan_bindings(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn find_niffler_api_key_product_plan_binding_by_api_key_id(
        &self,
        api_key_id: &str,
    ) -> Result<Option<StoredNifflerApiKeyProductPlanBinding>, GatewayError> {
        self.data
            .find_niffler_api_key_product_plan_binding_by_api_key_id(api_key_id)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_niffler_runtime_rollout_settings(
        &self,
        query: &NifflerRuntimeRolloutSettingListQuery,
    ) -> Result<StoredNifflerRuntimeRolloutSettingListPage, GatewayError> {
        self.data
            .list_niffler_runtime_rollout_settings(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn find_niffler_runtime_rollout_setting(
        &self,
        target_scope: NifflerRuntimeRolloutTargetScope,
        target_id: &str,
    ) -> Result<Option<StoredNifflerRuntimeRolloutSetting>, GatewayError> {
        self.data
            .find_niffler_runtime_rollout_setting(target_scope, target_id)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_niffler_error_return_settings(
        &self,
        query: &NifflerErrorReturnSettingListQuery,
    ) -> Result<StoredNifflerErrorReturnSettingListPage, GatewayError> {
        self.data
            .list_niffler_error_return_settings(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_niffler_billing_reservations(
        &self,
        query: &NifflerBillingReservationListQuery,
    ) -> Result<StoredNifflerBillingReservationListPage, GatewayError> {
        self.data
            .list_niffler_billing_reservations(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn sum_active_niffler_billing_reservation_wallet_usd(
        &self,
        user_id: Option<&str>,
        api_key_id: Option<&str>,
        now_unix_ms: u64,
    ) -> Result<f64, GatewayError> {
        self.data
            .sum_active_niffler_billing_reservation_wallet_usd(user_id, api_key_id, now_unix_ms)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_niffler_billing_reservation_dry_runs(
        &self,
        query: &NifflerBillingReservationDryRunListQuery,
    ) -> Result<StoredNifflerBillingReservationDryRunListPage, GatewayError> {
        self.data
            .list_niffler_billing_reservation_dry_runs(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_niffler_settlement_snapshots(
        &self,
        query: &NifflerSettlementSnapshotListQuery,
    ) -> Result<StoredNifflerSettlementSnapshotListPage, GatewayError> {
        self.data
            .list_niffler_settlement_snapshots(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_niffler_referral_reward_ledger(
        &self,
        query: &NifflerReferralRewardLedgerListQuery,
    ) -> Result<StoredNifflerReferralRewardLedgerListPage, GatewayError> {
        self.data
            .list_niffler_referral_reward_ledger(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_niffler_route_attempts(
        &self,
        query: &NifflerRouteAttemptListQuery,
    ) -> Result<StoredNifflerRouteAttemptListPage, GatewayError> {
        self.data
            .list_niffler_route_attempts(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_niffler_consistency_checks(
        &self,
        query: &NifflerConsistencyCheckListQuery,
    ) -> Result<StoredNifflerConsistencyCheckListPage, GatewayError> {
        self.data
            .list_niffler_consistency_checks(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn create_niffler_upstream_service(
        &self,
        record: CreateNifflerUpstreamServiceRecord,
    ) -> Result<Option<StoredNifflerUpstreamService>, GatewayError> {
        self.data
            .create_niffler_upstream_service(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn create_niffler_upstream_account(
        &self,
        record: CreateNifflerUpstreamAccountRecord,
    ) -> Result<Option<StoredNifflerUpstreamAccount>, GatewayError> {
        self.data
            .create_niffler_upstream_account(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn upsert_niffler_upstream_service_capability(
        &self,
        record: UpsertNifflerUpstreamServiceCapabilityRecord,
    ) -> Result<Option<StoredNifflerUpstreamServiceCapability>, GatewayError> {
        self.data
            .upsert_niffler_upstream_service_capability(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn create_niffler_product_plan(
        &self,
        record: CreateNifflerProductPlanRecord,
    ) -> Result<Option<StoredNifflerProductPlan>, GatewayError> {
        self.data
            .create_niffler_product_plan(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn upsert_niffler_product_plan_model(
        &self,
        record: UpsertNifflerProductPlanModelRecord,
    ) -> Result<Option<StoredNifflerProductPlanModel>, GatewayError> {
        self.data
            .upsert_niffler_product_plan_model(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn upsert_niffler_api_key_product_plan_binding(
        &self,
        record: UpsertNifflerApiKeyProductPlanBindingRecord,
    ) -> Result<Option<StoredNifflerApiKeyProductPlanBinding>, GatewayError> {
        let saved = self
            .data
            .upsert_niffler_api_key_product_plan_binding(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))?;
        self.niffler_runtime_rollout_decision_cache.clear();
        Ok(saved)
    }

    pub(crate) async fn upsert_niffler_runtime_rollout_setting(
        &self,
        record: UpsertNifflerRuntimeRolloutSettingRecord,
    ) -> Result<Option<StoredNifflerRuntimeRolloutSetting>, GatewayError> {
        let saved = self
            .data
            .upsert_niffler_runtime_rollout_setting(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))?;
        self.niffler_runtime_rollout_decision_cache.clear();
        Ok(saved)
    }

    pub(crate) async fn create_niffler_error_return_setting(
        &self,
        record: CreateNifflerErrorReturnSettingRecord,
    ) -> Result<Option<StoredNifflerErrorReturnSetting>, GatewayError> {
        self.data
            .create_niffler_error_return_setting(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn create_niffler_account_risk_event(
        &self,
        record: CreateNifflerAccountRiskEventRecord,
    ) -> Result<Option<StoredNifflerAccountRiskEvent>, GatewayError> {
        self.data
            .create_niffler_account_risk_event(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn create_niffler_settlement_snapshot(
        &self,
        record: CreateNifflerSettlementSnapshotRecord,
    ) -> Result<Option<StoredNifflerSettlementSnapshot>, GatewayError> {
        self.data
            .create_niffler_settlement_snapshot(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn create_niffler_billing_reservation(
        &self,
        record: CreateNifflerBillingReservationRecord,
    ) -> Result<Option<StoredNifflerBillingReservation>, GatewayError> {
        self.data
            .create_niffler_billing_reservation(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn finalize_niffler_billing_reservation_by_request_id(
        &self,
        record: FinalizeNifflerBillingReservationRecord,
    ) -> Result<Option<StoredNifflerBillingReservation>, GatewayError> {
        self.data
            .finalize_niffler_billing_reservation_by_request_id(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn create_niffler_billing_reservation_dry_run(
        &self,
        record: CreateNifflerBillingReservationDryRunRecord,
    ) -> Result<Option<StoredNifflerBillingReservationDryRun>, GatewayError> {
        self.data
            .create_niffler_billing_reservation_dry_run(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn create_niffler_referral_reward_ledger(
        &self,
        record: CreateNifflerReferralRewardLedgerRecord,
    ) -> Result<Option<StoredNifflerReferralRewardLedger>, GatewayError> {
        self.data
            .create_niffler_referral_reward_ledger(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn create_niffler_route_attempt(
        &self,
        record: CreateNifflerRouteAttemptRecord,
    ) -> Result<Option<StoredNifflerRouteAttempt>, GatewayError> {
        self.data
            .create_niffler_route_attempt(record)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }
}
