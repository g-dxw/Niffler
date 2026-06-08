use super::*;
use crate::GatewayError;

impl<'a> AdminAppState<'a> {
    pub(crate) fn has_niffler_core_reader(&self) -> bool {
        self.app.has_niffler_core_reader()
    }

    pub(crate) fn has_niffler_core_writer(&self) -> bool {
        self.app.has_niffler_core_writer()
    }

    pub(crate) async fn list_niffler_upstream_services(
        &self,
        query: &aether_data_contracts::repository::niffler_core::NifflerUpstreamServiceListQuery,
    ) -> Result<
        aether_data_contracts::repository::niffler_core::StoredNifflerUpstreamServiceListPage,
        GatewayError,
    > {
        self.app.list_niffler_upstream_services(query).await
    }

    pub(crate) async fn find_niffler_upstream_service_by_id(
        &self,
        upstream_service_id: &str,
    ) -> Result<
        Option<aether_data_contracts::repository::niffler_core::StoredNifflerUpstreamService>,
        GatewayError,
    > {
        self.app
            .find_niffler_upstream_service_by_id(upstream_service_id)
            .await
    }

    pub(crate) async fn list_niffler_upstream_accounts(
        &self,
        query: &aether_data_contracts::repository::niffler_core::NifflerUpstreamAccountListQuery,
    ) -> Result<
        aether_data_contracts::repository::niffler_core::StoredNifflerUpstreamAccountListPage,
        GatewayError,
    > {
        self.app.list_niffler_upstream_accounts(query).await
    }

    pub(crate) async fn list_niffler_upstream_service_capabilities(
        &self,
        query: &aether_data_contracts::repository::niffler_core::NifflerUpstreamServiceCapabilityListQuery,
    ) -> Result<
        aether_data_contracts::repository::niffler_core::StoredNifflerUpstreamServiceCapabilityListPage,
        GatewayError,
    >{
        self.app
            .list_niffler_upstream_service_capabilities(query)
            .await
    }

    pub(crate) async fn list_niffler_product_plans(
        &self,
        query: &aether_data_contracts::repository::niffler_core::NifflerProductPlanListQuery,
    ) -> Result<
        aether_data_contracts::repository::niffler_core::StoredNifflerProductPlanListPage,
        GatewayError,
    > {
        self.app.list_niffler_product_plans(query).await
    }

    pub(crate) async fn find_niffler_product_plan_by_id(
        &self,
        product_plan_id: &str,
    ) -> Result<
        Option<aether_data_contracts::repository::niffler_core::StoredNifflerProductPlan>,
        GatewayError,
    > {
        self.app
            .find_niffler_product_plan_by_id(product_plan_id)
            .await
    }

    pub(crate) async fn list_niffler_product_plan_models(
        &self,
        query: &aether_data_contracts::repository::niffler_core::NifflerProductPlanModelListQuery,
    ) -> Result<
        aether_data_contracts::repository::niffler_core::StoredNifflerProductPlanModelListPage,
        GatewayError,
    > {
        self.app.list_niffler_product_plan_models(query).await
    }

    pub(crate) async fn list_niffler_api_key_product_plan_bindings(
        &self,
        query: &aether_data_contracts::repository::niffler_core::NifflerApiKeyProductPlanBindingListQuery,
    ) -> Result<
        aether_data_contracts::repository::niffler_core::StoredNifflerApiKeyProductPlanBindingListPage,
        GatewayError,
    >{
        self.app
            .list_niffler_api_key_product_plan_bindings(query)
            .await
    }

    pub(crate) async fn find_niffler_api_key_product_plan_binding_by_api_key_id(
        &self,
        api_key_id: &str,
    ) -> Result<
        Option<
            aether_data_contracts::repository::niffler_core::StoredNifflerApiKeyProductPlanBinding,
        >,
        GatewayError,
    > {
        self.app
            .find_niffler_api_key_product_plan_binding_by_api_key_id(api_key_id)
            .await
    }

    pub(crate) async fn list_niffler_error_return_settings(
        &self,
        query: &aether_data_contracts::repository::niffler_core::NifflerErrorReturnSettingListQuery,
    ) -> Result<
        aether_data_contracts::repository::niffler_core::StoredNifflerErrorReturnSettingListPage,
        GatewayError,
    > {
        self.app.list_niffler_error_return_settings(query).await
    }

    pub(crate) async fn create_niffler_upstream_service(
        &self,
        record: aether_data_contracts::repository::niffler_core::CreateNifflerUpstreamServiceRecord,
    ) -> Result<
        Option<aether_data_contracts::repository::niffler_core::StoredNifflerUpstreamService>,
        GatewayError,
    > {
        self.app.create_niffler_upstream_service(record).await
    }

    pub(crate) async fn create_niffler_upstream_account(
        &self,
        record: aether_data_contracts::repository::niffler_core::CreateNifflerUpstreamAccountRecord,
    ) -> Result<
        Option<aether_data_contracts::repository::niffler_core::StoredNifflerUpstreamAccount>,
        GatewayError,
    > {
        self.app.create_niffler_upstream_account(record).await
    }

    pub(crate) async fn upsert_niffler_upstream_service_capability(
        &self,
        record: aether_data_contracts::repository::niffler_core::UpsertNifflerUpstreamServiceCapabilityRecord,
    ) -> Result<
        Option<
            aether_data_contracts::repository::niffler_core::StoredNifflerUpstreamServiceCapability,
        >,
        GatewayError,
    > {
        self.app
            .upsert_niffler_upstream_service_capability(record)
            .await
    }

    pub(crate) async fn create_niffler_product_plan(
        &self,
        record: aether_data_contracts::repository::niffler_core::CreateNifflerProductPlanRecord,
    ) -> Result<
        Option<aether_data_contracts::repository::niffler_core::StoredNifflerProductPlan>,
        GatewayError,
    > {
        self.app.create_niffler_product_plan(record).await
    }

    pub(crate) async fn upsert_niffler_product_plan_model(
        &self,
        record: aether_data_contracts::repository::niffler_core::UpsertNifflerProductPlanModelRecord,
    ) -> Result<
        Option<aether_data_contracts::repository::niffler_core::StoredNifflerProductPlanModel>,
        GatewayError,
    > {
        self.app.upsert_niffler_product_plan_model(record).await
    }

    pub(crate) async fn upsert_niffler_api_key_product_plan_binding(
        &self,
        record: aether_data_contracts::repository::niffler_core::UpsertNifflerApiKeyProductPlanBindingRecord,
    ) -> Result<
        Option<
            aether_data_contracts::repository::niffler_core::StoredNifflerApiKeyProductPlanBinding,
        >,
        GatewayError,
    > {
        self.app
            .upsert_niffler_api_key_product_plan_binding(record)
            .await
    }

    pub(crate) async fn create_niffler_error_return_setting(
        &self,
        record: aether_data_contracts::repository::niffler_core::CreateNifflerErrorReturnSettingRecord,
    ) -> Result<
        Option<aether_data_contracts::repository::niffler_core::StoredNifflerErrorReturnSetting>,
        GatewayError,
    > {
        self.app.create_niffler_error_return_setting(record).await
    }
}
