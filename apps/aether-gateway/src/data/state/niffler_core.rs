use super::{
    CreateNifflerErrorReturnSettingRecord, CreateNifflerProductPlanRecord,
    CreateNifflerUpstreamAccountRecord, CreateNifflerUpstreamServiceRecord, DataLayerError,
    GatewayDataState, NifflerApiKeyProductPlanBindingListQuery, NifflerErrorReturnSettingListQuery,
    NifflerProductPlanListQuery, NifflerProductPlanModelListQuery,
    NifflerRuntimeRolloutSettingListQuery, NifflerRuntimeRolloutTargetScope,
    NifflerUpstreamAccountListQuery, NifflerUpstreamServiceCapabilityListQuery,
    NifflerUpstreamServiceListQuery, StoredNifflerApiKeyProductPlanBinding,
    StoredNifflerApiKeyProductPlanBindingListPage, StoredNifflerErrorReturnSetting,
    StoredNifflerErrorReturnSettingListPage, StoredNifflerProductPlan,
    StoredNifflerProductPlanListPage, StoredNifflerProductPlanModel,
    StoredNifflerProductPlanModelListPage, StoredNifflerRuntimeRolloutSetting,
    StoredNifflerRuntimeRolloutSettingListPage, StoredNifflerUpstreamAccount,
    StoredNifflerUpstreamAccountListPage, StoredNifflerUpstreamService,
    StoredNifflerUpstreamServiceCapability, StoredNifflerUpstreamServiceCapabilityListPage,
    StoredNifflerUpstreamServiceListPage, UpsertNifflerApiKeyProductPlanBindingRecord,
    UpsertNifflerProductPlanModelRecord, UpsertNifflerRuntimeRolloutSettingRecord,
    UpsertNifflerUpstreamServiceCapabilityRecord,
};

impl GatewayDataState {
    pub(crate) fn has_niffler_core_reader(&self) -> bool {
        self.backends
            .as_ref()
            .is_some_and(|backends| backends.read().niffler_core().is_some())
    }

    pub(crate) fn has_niffler_core_writer(&self) -> bool {
        self.backends
            .as_ref()
            .is_some_and(|backends| backends.write().niffler_core().is_some())
    }

    pub(crate) async fn list_niffler_upstream_services(
        &self,
        query: &NifflerUpstreamServiceListQuery,
    ) -> Result<StoredNifflerUpstreamServiceListPage, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.read().niffler_core())
        {
            Some(repository) => repository.list_upstream_services(query).await,
            None => Ok(StoredNifflerUpstreamServiceListPage {
                items: Vec::new(),
                total: 0,
            }),
        }
    }

    pub(crate) async fn find_niffler_upstream_service_by_id(
        &self,
        upstream_service_id: &str,
    ) -> Result<Option<StoredNifflerUpstreamService>, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.read().niffler_core())
        {
            Some(repository) => {
                repository
                    .find_upstream_service_by_id(upstream_service_id)
                    .await
            }
            None => Ok(None),
        }
    }

    pub(crate) async fn list_niffler_upstream_service_capabilities(
        &self,
        query: &NifflerUpstreamServiceCapabilityListQuery,
    ) -> Result<StoredNifflerUpstreamServiceCapabilityListPage, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.read().niffler_core())
        {
            Some(repository) => repository.list_upstream_service_capabilities(query).await,
            None => Ok(StoredNifflerUpstreamServiceCapabilityListPage {
                items: Vec::new(),
                total: 0,
            }),
        }
    }

    pub(crate) async fn list_niffler_upstream_accounts(
        &self,
        query: &NifflerUpstreamAccountListQuery,
    ) -> Result<StoredNifflerUpstreamAccountListPage, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.read().niffler_core())
        {
            Some(repository) => repository.list_upstream_accounts(query).await,
            None => Ok(StoredNifflerUpstreamAccountListPage {
                items: Vec::new(),
                total: 0,
            }),
        }
    }

    pub(crate) async fn list_niffler_product_plans(
        &self,
        query: &NifflerProductPlanListQuery,
    ) -> Result<StoredNifflerProductPlanListPage, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.read().niffler_core())
        {
            Some(repository) => repository.list_product_plans(query).await,
            None => Ok(StoredNifflerProductPlanListPage {
                items: Vec::new(),
                total: 0,
            }),
        }
    }

    pub(crate) async fn find_niffler_product_plan_by_id(
        &self,
        product_plan_id: &str,
    ) -> Result<Option<StoredNifflerProductPlan>, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.read().niffler_core())
        {
            Some(repository) => repository.find_product_plan_by_id(product_plan_id).await,
            None => Ok(None),
        }
    }

    pub(crate) async fn list_niffler_product_plan_models(
        &self,
        query: &NifflerProductPlanModelListQuery,
    ) -> Result<StoredNifflerProductPlanModelListPage, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.read().niffler_core())
        {
            Some(repository) => repository.list_product_plan_models(query).await,
            None => Ok(StoredNifflerProductPlanModelListPage {
                items: Vec::new(),
                total: 0,
            }),
        }
    }

    pub(crate) async fn list_niffler_api_key_product_plan_bindings(
        &self,
        query: &NifflerApiKeyProductPlanBindingListQuery,
    ) -> Result<StoredNifflerApiKeyProductPlanBindingListPage, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.read().niffler_core())
        {
            Some(repository) => repository.list_api_key_product_plan_bindings(query).await,
            None => Ok(StoredNifflerApiKeyProductPlanBindingListPage {
                items: Vec::new(),
                total: 0,
            }),
        }
    }

    pub(crate) async fn find_niffler_api_key_product_plan_binding_by_api_key_id(
        &self,
        api_key_id: &str,
    ) -> Result<Option<StoredNifflerApiKeyProductPlanBinding>, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.read().niffler_core())
        {
            Some(repository) => {
                repository
                    .find_api_key_product_plan_binding_by_api_key_id(api_key_id)
                    .await
            }
            None => Ok(None),
        }
    }

    pub(crate) async fn list_niffler_runtime_rollout_settings(
        &self,
        query: &NifflerRuntimeRolloutSettingListQuery,
    ) -> Result<StoredNifflerRuntimeRolloutSettingListPage, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.read().niffler_core())
        {
            Some(repository) => repository.list_runtime_rollout_settings(query).await,
            None => Ok(StoredNifflerRuntimeRolloutSettingListPage {
                items: Vec::new(),
                total: 0,
            }),
        }
    }

    pub(crate) async fn find_niffler_runtime_rollout_setting(
        &self,
        target_scope: NifflerRuntimeRolloutTargetScope,
        target_id: &str,
    ) -> Result<Option<StoredNifflerRuntimeRolloutSetting>, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.read().niffler_core())
        {
            Some(repository) => {
                repository
                    .find_runtime_rollout_setting(target_scope, target_id)
                    .await
            }
            None => Ok(None),
        }
    }

    pub(crate) async fn list_niffler_error_return_settings(
        &self,
        query: &NifflerErrorReturnSettingListQuery,
    ) -> Result<StoredNifflerErrorReturnSettingListPage, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.read().niffler_core())
        {
            Some(repository) => repository.list_error_return_settings(query).await,
            None => Ok(StoredNifflerErrorReturnSettingListPage {
                items: Vec::new(),
                total: 0,
            }),
        }
    }

    pub(crate) async fn create_niffler_upstream_service(
        &self,
        record: CreateNifflerUpstreamServiceRecord,
    ) -> Result<Option<StoredNifflerUpstreamService>, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.write().niffler_core())
        {
            Some(repository) => repository.create_upstream_service(record).await.map(Some),
            None => Ok(None),
        }
    }

    pub(crate) async fn create_niffler_upstream_account(
        &self,
        record: CreateNifflerUpstreamAccountRecord,
    ) -> Result<Option<StoredNifflerUpstreamAccount>, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.write().niffler_core())
        {
            Some(repository) => repository.create_upstream_account(record).await.map(Some),
            None => Ok(None),
        }
    }

    pub(crate) async fn upsert_niffler_upstream_service_capability(
        &self,
        record: UpsertNifflerUpstreamServiceCapabilityRecord,
    ) -> Result<Option<StoredNifflerUpstreamServiceCapability>, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.write().niffler_core())
        {
            Some(repository) => repository
                .upsert_upstream_service_capability(record)
                .await
                .map(Some),
            None => Ok(None),
        }
    }

    pub(crate) async fn create_niffler_product_plan(
        &self,
        record: CreateNifflerProductPlanRecord,
    ) -> Result<Option<StoredNifflerProductPlan>, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.write().niffler_core())
        {
            Some(repository) => repository.create_product_plan(record).await.map(Some),
            None => Ok(None),
        }
    }

    pub(crate) async fn upsert_niffler_product_plan_model(
        &self,
        record: UpsertNifflerProductPlanModelRecord,
    ) -> Result<Option<StoredNifflerProductPlanModel>, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.write().niffler_core())
        {
            Some(repository) => repository.upsert_product_plan_model(record).await.map(Some),
            None => Ok(None),
        }
    }

    pub(crate) async fn upsert_niffler_api_key_product_plan_binding(
        &self,
        record: UpsertNifflerApiKeyProductPlanBindingRecord,
    ) -> Result<Option<StoredNifflerApiKeyProductPlanBinding>, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.write().niffler_core())
        {
            Some(repository) => repository
                .upsert_api_key_product_plan_binding(record)
                .await
                .map(Some),
            None => Ok(None),
        }
    }

    pub(crate) async fn upsert_niffler_runtime_rollout_setting(
        &self,
        record: UpsertNifflerRuntimeRolloutSettingRecord,
    ) -> Result<Option<StoredNifflerRuntimeRolloutSetting>, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.write().niffler_core())
        {
            Some(repository) => repository
                .upsert_runtime_rollout_setting(record)
                .await
                .map(Some),
            None => Ok(None),
        }
    }

    pub(crate) async fn create_niffler_error_return_setting(
        &self,
        record: CreateNifflerErrorReturnSettingRecord,
    ) -> Result<Option<StoredNifflerErrorReturnSetting>, DataLayerError> {
        match self
            .backends
            .as_ref()
            .and_then(|backends| backends.write().niffler_core())
        {
            Some(repository) => repository
                .create_error_return_setting(record)
                .await
                .map(Some),
            None => Ok(None),
        }
    }
}
