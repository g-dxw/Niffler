use crate::data::state::{
    ReferralRelationshipListQuery, ReferralRelationshipRecord, ReferralRewardConfig,
    ReferralRewardListQuery, ReferralRewardRecord, ReferralUserDashboard,
};
use crate::niffler_runtime::{
    resolve_niffler_runtime_rollout_decision, NifflerRuntimeRolloutDecisionSource,
};
use crate::{AppState, GatewayError};
use aether_data_contracts::repository::niffler_core::{
    CreateNifflerReferralRewardLedgerRecord, NifflerReferralRewardLedgerStatus,
};
use axum::http::StatusCode;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use tracing::warn;

fn referral_data_error(err: aether_data::DataLayerError) -> GatewayError {
    match err {
        aether_data::DataLayerError::InvalidInput(detail) => GatewayError::Client {
            status: StatusCode::BAD_REQUEST,
            message: detail,
        },
        other => GatewayError::Internal(other.to_string()),
    }
}

fn config_bool(value: Option<&serde_json::Value>, default: bool) -> bool {
    match value {
        Some(serde_json::Value::Bool(value)) => *value,
        Some(serde_json::Value::String(value)) => {
            match value.trim().to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => true,
                "false" | "0" | "no" | "off" => false,
                _ => default,
            }
        }
        Some(serde_json::Value::Number(value)) => {
            value.as_i64().map(|value| value != 0).unwrap_or(default)
        }
        _ => default,
    }
}

fn config_string(value: Option<&serde_json::Value>) -> Option<String> {
    match value {
        Some(serde_json::Value::String(value)) => {
            let value = value.trim();
            (!value.is_empty()).then_some(value.to_string())
        }
        Some(value) => Some(value.to_string()),
        None => None,
    }
}

fn config_f64(value: Option<&serde_json::Value>, default: f64) -> f64 {
    match value {
        Some(serde_json::Value::Number(value)) => value.as_f64().unwrap_or(default),
        Some(serde_json::Value::String(value)) => value.trim().parse::<f64>().unwrap_or(default),
        _ => default,
    }
}

#[derive(Debug, Clone)]
struct ReferralLedgerRolloutMatch {
    source: NifflerRuntimeRolloutDecisionSource,
    api_key_id: Option<String>,
    product_plan_id: Option<String>,
    matched_key_count: usize,
}

impl AppState {
    pub(crate) fn has_referral_data_backend(&self) -> bool {
        self.data.has_referral_data_backend()
    }

    pub(crate) async fn record_user_privacy_policy_acceptance(
        &self,
        user_id: &str,
        version: &str,
    ) -> Result<bool, GatewayError> {
        self.data
            .record_user_privacy_policy_acceptance(user_id, version)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn referral_reward_config(
        &self,
    ) -> Result<Option<ReferralRewardConfig>, GatewayError> {
        let enabled = self
            .read_system_config_json_value("referral_enabled")
            .await?;
        if !config_bool(enabled.as_ref(), false) {
            return Ok(None);
        }
        let mode = self
            .read_system_config_json_value("referral_reward_mode")
            .await?;
        let mode = config_string(mode.as_ref()).unwrap_or_else(|| "percent".to_string());
        let percent = self
            .read_system_config_json_value("referral_recharge_percent")
            .await?;
        let headcount_amount = self
            .read_system_config_json_value("referral_headcount_amount_usd")
            .await?;
        let headcount_trigger = self
            .read_system_config_json_value("referral_headcount_trigger")
            .await?;
        let headcount_trigger =
            config_string(headcount_trigger.as_ref()).unwrap_or_else(|| "registration".to_string());
        Ok(Some(ReferralRewardConfig {
            percent_enabled: matches!(mode.as_str(), "percent" | "both"),
            percent_rate: config_f64(percent.as_ref(), 0.0),
            headcount_enabled: matches!(mode.as_str(), "headcount" | "both"),
            headcount_amount_usd: config_f64(headcount_amount.as_ref(), 0.0),
            headcount_trigger,
        }))
    }

    pub(crate) async fn bind_referral_invite_after_registration(
        &self,
        user_id: &str,
        email_verified: bool,
        invite_code: Option<&str>,
        source: Option<serde_json::Value>,
    ) -> Result<(), GatewayError> {
        let Some(config) = self.referral_reward_config().await? else {
            return Ok(());
        };
        let relationship = self
            .data
            .bind_referral_invite_code(user_id, invite_code, source)
            .await
            .map_err(referral_data_error)?;
        let trigger_matches = config.headcount_trigger == "registration"
            || (config.headcount_trigger == "email_verified" && email_verified);
        if relationship.is_some()
            && config.headcount_enabled
            && trigger_matches
            && config.headcount_amount_usd > 0.0
        {
            self.data
                .apply_registration_referral_reward(
                    user_id,
                    config.headcount_amount_usd,
                    &config.headcount_trigger,
                )
                .await
                .map_err(|err| GatewayError::Internal(err.to_string()))?;
        }
        Ok(())
    }

    pub(crate) async fn referral_dashboard(
        &self,
        user_id: &str,
    ) -> Result<Option<ReferralUserDashboard>, GatewayError> {
        self.data
            .referral_dashboard(user_id)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_admin_referral_relationships(
        &self,
        query: ReferralRelationshipListQuery,
    ) -> Result<
        Option<(
            Vec<ReferralRelationshipRecord>,
            u64,
            crate::data::state::ReferralAdminStats,
        )>,
        GatewayError,
    > {
        self.data
            .list_admin_referral_relationships(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn list_admin_referral_rewards(
        &self,
        query: ReferralRewardListQuery,
    ) -> Result<
        Option<(
            Vec<ReferralRewardRecord>,
            u64,
            crate::data::state::ReferralAdminStats,
        )>,
        GatewayError,
    > {
        self.data
            .list_admin_referral_rewards(query)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }

    pub(crate) async fn retry_referral_reward(
        &self,
        reward_id: &str,
        operator_id: Option<&str>,
        note: Option<&str>,
    ) -> Result<Option<ReferralRewardRecord>, GatewayError> {
        self.data
            .retry_referral_reward(reward_id, operator_id, note)
            .await
            .map_err(referral_data_error)
    }

    pub(crate) async fn void_referral_reward(
        &self,
        reward_id: &str,
        operator_id: Option<&str>,
        note: Option<&str>,
    ) -> Result<Option<ReferralRewardRecord>, GatewayError> {
        self.data
            .void_referral_reward(reward_id, operator_id, note)
            .await
            .map_err(referral_data_error)
    }

    pub(crate) async fn apply_referral_rewards_for_paid_order(
        &self,
        order: &aether_data::repository::wallet::StoredAdminPaymentOrder,
    ) -> Result<Vec<ReferralRewardRecord>, GatewayError> {
        let Some(config) = self.referral_reward_config().await? else {
            return Ok(Vec::new());
        };
        let rewards = self
            .data
            .apply_paid_order_referral_rewards(&order.id, config.clone())
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))?;
        self.record_niffler_referral_ledger_shadow(&order.id, &rewards, &config)
            .await;
        Ok(rewards)
    }

    pub(crate) async fn apply_referral_rewards_for_payment_order_id(
        &self,
        order_id: &str,
    ) -> Result<Vec<ReferralRewardRecord>, GatewayError> {
        let Some(config) = self.referral_reward_config().await? else {
            return Ok(Vec::new());
        };
        let rewards = self
            .data
            .apply_paid_order_referral_rewards(order_id, config.clone())
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))?;
        self.record_niffler_referral_ledger_shadow(order_id, &rewards, &config)
            .await;
        Ok(rewards)
    }

    async fn record_niffler_referral_ledger_shadow(
        &self,
        order_id: &str,
        rewards: &[ReferralRewardRecord],
        config: &ReferralRewardConfig,
    ) {
        if let Err(err) = self
            .record_niffler_referral_ledger_shadow_inner(order_id, rewards, config)
            .await
        {
            warn!(
                error = ?err,
                order_id = %order_id,
                "failed to write niffler referral reward ledger shadow"
            );
        }
    }

    async fn record_niffler_referral_ledger_shadow_inner(
        &self,
        order_id: &str,
        rewards: &[ReferralRewardRecord],
        config: &ReferralRewardConfig,
    ) -> Result<(), GatewayError> {
        if rewards.is_empty() {
            return Ok(());
        }
        let Some(first_reward) = rewards.first() else {
            return Ok(());
        };
        let inviter_user_id = first_reward.inviter_user_id.clone();
        let invitee_user_id = first_reward.invitee_user_id.clone();
        if rewards.iter().any(|reward| {
            reward.inviter_user_id != inviter_user_id || reward.invitee_user_id != invitee_user_id
        }) {
            warn!(
                order_id = %order_id,
                "skip niffler referral ledger shadow because legacy rewards disagree on users"
            );
            return Ok(());
        }
        let rollout = match self
            .resolve_referral_ledger_rollout_for_user(&invitee_user_id)
            .await?
        {
            Some(value) => value,
            None => return Ok(()),
        };
        let reward_amount_usd: f64 = rewards.iter().map(|reward| reward.amount_usd).sum();
        if reward_amount_usd <= 0.0 {
            return Ok(());
        }
        let now_unix_ms = chrono::Utc::now().timestamp_millis().max(0) as u64;
        let source = match rollout.source {
            NifflerRuntimeRolloutDecisionSource::ApiKey => "api_key",
            NifflerRuntimeRolloutDecisionSource::ProductPlan => "product_plan",
        };
        let rule_snapshot = json!({
            "source": "legacy_referral_rewards",
            "legacy_config": {
                "percent_enabled": config.percent_enabled,
                "percent_rate": config.percent_rate,
                "headcount_enabled": config.headcount_enabled,
                "headcount_amount_usd": config.headcount_amount_usd,
                "headcount_trigger": config.headcount_trigger,
            },
            "rollout": {
                "source": source,
                "api_key_id": rollout.api_key_id,
                "product_plan_id": rollout.product_plan_id,
                "matched_key_count": rollout.matched_key_count,
            },
            "legacy_rewards": rewards.iter().map(|reward| json!({
                "id": reward.id,
                "reward_type": reward.reward_type,
                "trigger_point": reward.trigger_point,
                "amount_usd": reward.amount_usd,
                "status": reward.status,
                "wallet_transaction_id": reward.wallet_transaction_id,
                "idempotency_key": reward.idempotency_key,
            })).collect::<Vec<_>>(),
            "shadow_only": true,
        });
        let record = CreateNifflerReferralRewardLedgerRecord {
            id: uuid::Uuid::new_v4().to_string(),
            order_id: order_id.to_string(),
            idempotency_key: format!("niffler-referral-ledger:order:{order_id}"),
            inviter_user_id,
            invitee_user_id,
            rule_id: None,
            reward_amount_usd,
            rule_snapshot,
            status: NifflerReferralRewardLedgerStatus::Pending,
            created_at_unix_ms: now_unix_ms,
            updated_at_unix_ms: now_unix_ms,
        };
        let _ = self.create_niffler_referral_reward_ledger(record).await?;
        Ok(())
    }

    async fn resolve_referral_ledger_rollout_for_user(
        &self,
        user_id: &str,
    ) -> Result<Option<ReferralLedgerRolloutMatch>, GatewayError> {
        let now_unix_secs = chrono::Utc::now().timestamp().max(0) as u64;
        let keys = self
            .list_auth_api_key_export_records_by_user_ids(&[user_id.to_string()])
            .await?;
        let active_keys = keys
            .into_iter()
            .filter(|key| key.is_active)
            .filter(|key| {
                key.expires_at_unix_secs
                    .is_none_or(|expires_at| expires_at > now_unix_secs)
            })
            .collect::<Vec<_>>();

        let mut key_matches = Vec::new();
        let mut plan_matches: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for key in active_keys {
            let decision = resolve_niffler_runtime_rollout_decision(self, &key.api_key_id).await?;
            if !decision.enable_referral_ledger {
                continue;
            }
            match decision.source {
                Some(NifflerRuntimeRolloutDecisionSource::ApiKey) => {
                    key_matches.push(key.api_key_id);
                }
                Some(NifflerRuntimeRolloutDecisionSource::ProductPlan) => {
                    if let Some(product_plan_id) = decision.product_plan_id {
                        plan_matches
                            .entry(product_plan_id)
                            .or_default()
                            .insert(key.api_key_id);
                    }
                }
                None => {}
            }
        }

        if key_matches.len() == 1 {
            return Ok(Some(ReferralLedgerRolloutMatch {
                source: NifflerRuntimeRolloutDecisionSource::ApiKey,
                api_key_id: key_matches.into_iter().next(),
                product_plan_id: None,
                matched_key_count: 1,
            }));
        }
        if key_matches.len() > 1 {
            warn!(
                user_id = %user_id,
                matched_key_count = key_matches.len(),
                "skip niffler referral ledger shadow because multiple api keys enable referral ledger"
            );
            return Ok(None);
        }

        if plan_matches.len() == 1 {
            let (product_plan_id, api_key_ids) = plan_matches.into_iter().next().expect("len is 1");
            return Ok(Some(ReferralLedgerRolloutMatch {
                source: NifflerRuntimeRolloutDecisionSource::ProductPlan,
                api_key_id: (api_key_ids.len() == 1)
                    .then(|| api_key_ids.iter().next().cloned())
                    .flatten(),
                product_plan_id: Some(product_plan_id),
                matched_key_count: api_key_ids.len(),
            }));
        }
        if plan_matches.len() > 1 {
            warn!(
                user_id = %user_id,
                matched_product_plan_count = plan_matches.len(),
                "skip niffler referral ledger shadow because multiple product plans enable referral ledger"
            );
        }
        Ok(None)
    }

    pub(crate) async fn reverse_referral_rewards_for_order(
        &self,
        order_id: &str,
        amount_usd: f64,
    ) -> Result<Vec<ReferralRewardRecord>, GatewayError> {
        self.data
            .reverse_referral_rewards_for_order(order_id, amount_usd)
            .await
            .map_err(|err| GatewayError::Internal(err.to_string()))
    }
}
