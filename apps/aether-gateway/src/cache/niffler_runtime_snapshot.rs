use std::time::Duration;

use aether_cache::ExpiringMap;

use crate::niffler_runtime::{NifflerRuntimeModelAccessSnapshot, NifflerRuntimePolicySnapshot};

#[derive(Debug, Default)]
pub(crate) struct NifflerRuntimeSnapshotCache {
    policy_entries: ExpiringMap<String, NifflerRuntimePolicySnapshot>,
    model_access_entries: ExpiringMap<String, NifflerRuntimeModelAccessSnapshot>,
}

impl NifflerRuntimeSnapshotCache {
    pub(crate) fn get_policy_fresh(
        &self,
        api_key_id: &str,
        ttl: Duration,
    ) -> Option<NifflerRuntimePolicySnapshot> {
        self.policy_entries.get_fresh(&api_key_id.to_string(), ttl)
    }

    pub(crate) fn insert_policy(
        &self,
        api_key_id: String,
        snapshot: NifflerRuntimePolicySnapshot,
        ttl: Duration,
        max_entries: usize,
    ) {
        self.policy_entries
            .insert(api_key_id, snapshot, ttl, max_entries);
    }

    pub(crate) fn get_model_access_fresh(
        &self,
        cache_key: &str,
        ttl: Duration,
    ) -> Option<NifflerRuntimeModelAccessSnapshot> {
        self.model_access_entries
            .get_fresh(&cache_key.to_string(), ttl)
    }

    pub(crate) fn insert_model_access(
        &self,
        cache_key: String,
        snapshot: NifflerRuntimeModelAccessSnapshot,
        ttl: Duration,
        max_entries: usize,
    ) {
        self.model_access_entries
            .insert(cache_key, snapshot, ttl, max_entries);
    }

    pub(crate) fn clear(&self) {
        self.policy_entries.clear();
        self.model_access_entries.clear();
    }
}
