use super::{AppState, GatewayError};
use aether_data_contracts::repository::niffler_core::{
    CreateNifflerProductPlanRecord, CreateNifflerUpstreamAccountRecord,
    CreateNifflerUpstreamServiceRecord, NifflerProductPlanListQuery,
    NifflerProductPlanModelListQuery, NifflerUpstreamAccountListQuery,
    NifflerUpstreamServiceListQuery, StoredNifflerProductPlan, StoredNifflerProductPlanListPage,
    StoredNifflerProductPlanModel, StoredNifflerProductPlanModelListPage,
    StoredNifflerUpstreamAccount, StoredNifflerUpstreamAccountListPage,
    StoredNifflerUpstreamService, StoredNifflerUpstreamServiceCapability,
    StoredNifflerUpstreamServiceListPage, UpsertNifflerProductPlanModelRecord,
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
}
