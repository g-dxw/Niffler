use super::{
    CreateNifflerUpstreamAccountRecord, CreateNifflerUpstreamServiceRecord, DataLayerError,
    GatewayDataState, NifflerUpstreamAccountListQuery, NifflerUpstreamServiceListQuery,
    StoredNifflerUpstreamAccount, StoredNifflerUpstreamAccountListPage,
    StoredNifflerUpstreamService, StoredNifflerUpstreamServiceCapability,
    StoredNifflerUpstreamServiceListPage, UpsertNifflerUpstreamServiceCapabilityRecord,
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
}
