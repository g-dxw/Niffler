use std::time::Duration;

use aether_data_contracts::repository::niffler_core::{
    NifflerRuntimeRolloutTargetScope, StoredNifflerRuntimeRolloutSetting,
};

use crate::{AppState, GatewayError};

const RUNTIME_ROLLOUT_DECISION_CACHE_TTL: Duration = Duration::from_secs(30);
const RUNTIME_ROLLOUT_DECISION_CACHE_MAX_ENTRIES: usize = 2_048;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NifflerRuntimeRolloutDecisionSource {
    ApiKey,
    ProductPlan,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NifflerRuntimeRolloutDecision {
    pub(crate) api_key_id: String,
    pub(crate) product_plan_id: Option<String>,
    pub(crate) source: Option<NifflerRuntimeRolloutDecisionSource>,
    pub(crate) enable_new_routing: bool,
    pub(crate) enable_settlement_snapshot: bool,
    pub(crate) enable_error_return_rules: bool,
    pub(crate) enable_billing_reservation: bool,
    pub(crate) enable_referral_ledger: bool,
}

impl NifflerRuntimeRolloutDecision {
    fn disabled(api_key_id: String, product_plan_id: Option<String>) -> Self {
        Self {
            api_key_id,
            product_plan_id,
            source: None,
            enable_new_routing: false,
            enable_settlement_snapshot: false,
            enable_error_return_rules: false,
            enable_billing_reservation: false,
            enable_referral_ledger: false,
        }
    }

    fn from_setting(
        api_key_id: String,
        product_plan_id: Option<String>,
        source: NifflerRuntimeRolloutDecisionSource,
        setting: &StoredNifflerRuntimeRolloutSetting,
    ) -> Self {
        Self {
            api_key_id,
            product_plan_id,
            source: Some(source),
            enable_new_routing: setting.enable_new_routing,
            enable_settlement_snapshot: setting.enable_settlement_snapshot,
            enable_error_return_rules: setting.enable_error_return_rules,
            enable_billing_reservation: setting.enable_billing_reservation,
            enable_referral_ledger: setting.enable_referral_ledger,
        }
    }
}

pub(crate) async fn resolve_niffler_runtime_rollout_decision(
    state: &AppState,
    api_key_id: &str,
) -> Result<NifflerRuntimeRolloutDecision, GatewayError> {
    let api_key_id = api_key_id.trim();
    if api_key_id.is_empty() {
        return Ok(NifflerRuntimeRolloutDecision::disabled(String::new(), None));
    }

    if let Some(cached) = state
        .niffler_runtime_rollout_decision_cache
        .get_fresh(api_key_id, RUNTIME_ROLLOUT_DECISION_CACHE_TTL)
    {
        return Ok(cached);
    }

    let decision = resolve_niffler_runtime_rollout_decision_uncached(state, api_key_id).await?;
    state.niffler_runtime_rollout_decision_cache.insert(
        api_key_id.to_string(),
        decision.clone(),
        RUNTIME_ROLLOUT_DECISION_CACHE_TTL,
        RUNTIME_ROLLOUT_DECISION_CACHE_MAX_ENTRIES,
    );
    Ok(decision)
}

async fn resolve_niffler_runtime_rollout_decision_uncached(
    state: &AppState,
    api_key_id: &str,
) -> Result<NifflerRuntimeRolloutDecision, GatewayError> {
    let api_key_id_owned = api_key_id.to_string();
    let key_setting = state
        .find_niffler_runtime_rollout_setting(NifflerRuntimeRolloutTargetScope::ApiKey, api_key_id)
        .await?;
    if let Some(setting) = key_setting.as_ref().filter(|setting| setting.is_active) {
        return Ok(NifflerRuntimeRolloutDecision::from_setting(
            api_key_id_owned,
            None,
            NifflerRuntimeRolloutDecisionSource::ApiKey,
            setting,
        ));
    }

    let Some(binding) = state
        .find_niffler_api_key_product_plan_binding_by_api_key_id(api_key_id)
        .await?
    else {
        return Ok(NifflerRuntimeRolloutDecision::disabled(
            api_key_id_owned,
            None,
        ));
    };

    let product_plan_id = binding.product_plan_id.clone();
    let Some(product_plan) = state
        .find_niffler_product_plan_by_id(&product_plan_id)
        .await?
    else {
        return Ok(NifflerRuntimeRolloutDecision::disabled(
            api_key_id_owned,
            Some(product_plan_id),
        ));
    };
    if !product_plan.is_active {
        return Ok(NifflerRuntimeRolloutDecision::disabled(
            api_key_id_owned,
            Some(product_plan_id),
        ));
    }

    let product_plan_setting = state
        .find_niffler_runtime_rollout_setting(
            NifflerRuntimeRolloutTargetScope::ProductPlan,
            &product_plan_id,
        )
        .await?;
    let Some(setting) = product_plan_setting
        .as_ref()
        .filter(|setting| setting.is_active)
    else {
        return Ok(NifflerRuntimeRolloutDecision::disabled(
            api_key_id_owned,
            Some(product_plan_id),
        ));
    };

    Ok(NifflerRuntimeRolloutDecision::from_setting(
        api_key_id_owned,
        Some(product_plan_id),
        NifflerRuntimeRolloutDecisionSource::ProductPlan,
        setting,
    ))
}
