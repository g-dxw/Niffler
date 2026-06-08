mod mysql;
mod postgres;
mod sqlite;

pub use aether_data_contracts::repository::niffler_core::{
    CreateNifflerAccountRiskEventRecord, CreateNifflerBillingReservationDryRunRecord,
    CreateNifflerBillingReservationRecord, CreateNifflerErrorReturnSettingRecord,
    CreateNifflerProductPlanRecord, CreateNifflerReferralRewardLedgerRecord,
    CreateNifflerRouteAttemptRecord, CreateNifflerSettlementSnapshotRecord,
    CreateNifflerUpstreamAccountRecord, CreateNifflerUpstreamServiceRecord,
    FinalizeNifflerBillingReservationRecord, NifflerAccountProtectionAction, NifflerAccountStatus,
    NifflerApiKeyProductPlanBindingListQuery, NifflerBillingReservationDryRunListQuery,
    NifflerBillingReservationListQuery, NifflerBillingReservationStatus,
    NifflerConsistencyCheckListQuery, NifflerCoreReadRepository, NifflerCoreRepository,
    NifflerCoreWriteRepository, NifflerErrorResponseScope, NifflerErrorReturnSettingListQuery,
    NifflerPauseDuration, NifflerProductPlanListQuery, NifflerProductPlanModelListQuery,
    NifflerProtocolKind, NifflerReferralRewardLedgerListQuery, NifflerReferralRewardLedgerStatus,
    NifflerRouteAttemptListQuery, NifflerRuntimeRolloutSettingListQuery,
    NifflerRuntimeRolloutTargetScope, NifflerServiceCapabilityKind,
    NifflerSettlementSnapshotListQuery, NifflerUpstreamAccountListQuery,
    NifflerUpstreamErrorHandlingStep, NifflerUpstreamServiceCapabilityListQuery,
    NifflerUpstreamServiceListQuery, NifflerUserResponseMode, StoredNifflerAccountRiskEvent,
    StoredNifflerApiKeyProductPlanBinding, StoredNifflerApiKeyProductPlanBindingListPage,
    StoredNifflerBillingReservation, StoredNifflerBillingReservationDryRun,
    StoredNifflerBillingReservationDryRunListPage, StoredNifflerBillingReservationListPage,
    StoredNifflerConsistencyCheckItem, StoredNifflerConsistencyCheckListPage,
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
pub use mysql::MysqlNifflerCoreRepository;
pub use postgres::SqlxNifflerCoreRepository;
pub use sqlite::SqliteNifflerCoreRepository;

pub(crate) fn i64_from_u64(value: u64, field: &str) -> Result<i64, crate::DataLayerError> {
    i64::try_from(value).map_err(|_| {
        crate::DataLayerError::InvalidInput(format!("{field} is too large for database"))
    })
}

pub(crate) fn u64_from_i64(value: i64, field: &str) -> Result<u64, crate::DataLayerError> {
    u64::try_from(value).map_err(|_| {
        crate::DataLayerError::UnexpectedValue(format!("{field} is negative: {value}"))
    })
}

pub(crate) fn json_to_string(
    value: Option<&serde_json::Value>,
    field: &str,
) -> Result<Option<String>, crate::DataLayerError> {
    value
        .map(|value| {
            serde_json::to_string(value).map_err(|err| {
                crate::DataLayerError::InvalidInput(format!("{field} is not valid JSON: {err}"))
            })
        })
        .transpose()
}

pub(crate) fn json_from_string(
    value: Option<String>,
    field: &str,
) -> Result<Option<serde_json::Value>, crate::DataLayerError> {
    value
        .map(|value| {
            serde_json::from_str(&value).map_err(|err| {
                crate::DataLayerError::UnexpectedValue(format!(
                    "{field} contains invalid JSON: {err}"
                ))
            })
        })
        .transpose()
}

pub(crate) fn bounded_limit(limit: usize) -> i64 {
    i64::try_from(limit.clamp(1, 200)).unwrap_or(200)
}

pub(crate) fn bounded_offset(offset: usize) -> i64 {
    i64::try_from(offset).unwrap_or(i64::MAX)
}
