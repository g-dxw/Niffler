use std::time::Duration;

use aether_cache::ExpiringMap;

use crate::niffler_runtime::NifflerRuntimeRolloutDecision;

#[derive(Debug, Default)]
pub(crate) struct NifflerRuntimeRolloutDecisionCache {
    entries: ExpiringMap<String, NifflerRuntimeRolloutDecision>,
}

impl NifflerRuntimeRolloutDecisionCache {
    pub(crate) fn get_fresh(
        &self,
        api_key_id: &str,
        ttl: Duration,
    ) -> Option<NifflerRuntimeRolloutDecision> {
        self.entries.get_fresh(&api_key_id.to_string(), ttl)
    }

    pub(crate) fn insert(
        &self,
        api_key_id: String,
        decision: NifflerRuntimeRolloutDecision,
        ttl: Duration,
        max_entries: usize,
    ) {
        self.entries.insert(api_key_id, decision, ttl, max_entries);
    }

    pub(crate) fn clear(&self) {
        self.entries.clear();
    }
}
