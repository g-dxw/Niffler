use super::{AppState, GatewayError};
use aether_data_contracts::repository::niffler_core::{
    CreateNifflerUpstreamAccountRecord, CreateNifflerUpstreamServiceRecord,
    NifflerUpstreamAccountListQuery, NifflerUpstreamServiceListQuery, StoredNifflerUpstreamAccount,
    StoredNifflerUpstreamAccountListPage, StoredNifflerUpstreamService,
    StoredNifflerUpstreamServiceCapability, StoredNifflerUpstreamServiceListPage,
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

    pub(crate) async fn list_niffler_upstream_accounts(
        &self,
        query: &NifflerUpstreamAccountListQuery,
    ) -> Result<StoredNifflerUpstreamAccountListPage, GatewayError> {
        self.data
            .list_niffler_upstream_accounts(query)
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
}
