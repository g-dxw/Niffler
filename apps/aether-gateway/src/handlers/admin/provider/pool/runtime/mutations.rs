use super::keys::{pool_cooldown_index_key, pool_cooldown_key, pool_model_cooldown_index_key};
use crate::handlers::admin::request::AdminAppState;
use aether_runtime_state::RuntimeState;

pub(crate) async fn clear_admin_provider_pool_cooldown(
    state: &AdminAppState<'_>,
    provider_id: &str,
    key_id: &str,
) {
    clear_provider_pool_cooldown(state.runtime_state(), provider_id, key_id).await;
}

async fn clear_provider_pool_cooldown(runtime: &RuntimeState, provider_id: &str, key_id: &str) {
    let _ = runtime
        .kv_delete(&pool_cooldown_key(provider_id, key_id))
        .await;
    let _ = runtime
        .set_remove(&pool_cooldown_index_key(provider_id), key_id)
        .await;

    let model_index_key = pool_model_cooldown_index_key(provider_id, key_id);
    if let Ok(model_cooldown_keys) = runtime.set_members(&model_index_key).await {
        if runtime.kv_delete_many(&model_cooldown_keys).await.is_ok() {
            for model_cooldown_key in model_cooldown_keys {
                let _ = runtime
                    .set_remove(&model_index_key, &model_cooldown_key)
                    .await;
            }
        }
    }
}

pub(crate) async fn reset_admin_provider_pool_cost(
    state: &AdminAppState<'_>,
    provider_id: &str,
    key_id: &str,
) {
    let _ = state
        .runtime_state()
        .score_remove_by_score(&format!("ap:{provider_id}:cost:{key_id}"), f64::INFINITY)
        .await;
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use aether_runtime_state::{MemoryRuntimeStateConfig, RuntimeState};

    use super::super::keys::{
        pool_cooldown_index_key, pool_cooldown_key, pool_model_cooldown_index_key,
        pool_model_cooldown_key,
    };
    use super::super::writes::record_admin_provider_pool_model_cooldown;
    use super::clear_provider_pool_cooldown;

    #[tokio::test]
    async fn clear_cooldown_removes_account_and_all_model_cooldowns() {
        let runtime = RuntimeState::memory(MemoryRuntimeStateConfig::default());
        let provider_id = "provider";
        let key_id = "key";
        runtime
            .kv_set(
                &pool_cooldown_key(provider_id, key_id),
                "rate_limited".to_string(),
                Some(Duration::from_secs(60)),
            )
            .await
            .expect("account cooldown should be written");
        runtime
            .set_add(&pool_cooldown_index_key(provider_id), key_id)
            .await
            .expect("account cooldown should be indexed");
        for model_name in ["gpt-5.6-sol", "gpt-5.6-terra"] {
            record_admin_provider_pool_model_cooldown(
                &runtime,
                provider_id,
                key_id,
                model_name,
                "capacity",
                60,
            )
            .await;
        }

        clear_provider_pool_cooldown(&runtime, provider_id, key_id).await;

        assert!(runtime
            .kv_get(&pool_cooldown_key(provider_id, key_id))
            .await
            .expect("account cooldown read should succeed")
            .is_none());
        for model_name in ["gpt-5.6-sol", "gpt-5.6-terra"] {
            assert!(runtime
                .kv_get(&pool_model_cooldown_key(provider_id, key_id, model_name,))
                .await
                .expect("model cooldown read should succeed")
                .is_none());
        }
        assert!(runtime
            .set_members(&pool_model_cooldown_index_key(provider_id, key_id))
            .await
            .expect("model cooldown index read should succeed")
            .is_empty());
    }
}
