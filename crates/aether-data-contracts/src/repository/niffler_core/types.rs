use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerAccountStatus {
    Available,
    Disabled,
    Invalid,
    QuotaExhausted,
    CoolingDown,
}

impl NifflerAccountStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Disabled => "disabled",
            Self::Invalid => "invalid",
            Self::QuotaExhausted => "quota_exhausted",
            Self::CoolingDown => "cooling_down",
        }
    }

    pub fn allows_scheduling(self) -> bool {
        matches!(self, Self::Available)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerAccountProtectionAction {
    RecordOnly,
    PauseScheduling,
    DisableAccount,
}

impl NifflerAccountProtectionAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RecordOnly => "record_only",
            Self::PauseScheduling => "pause_scheduling",
            Self::DisableAccount => "disable_account",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerPauseDuration {
    TenMinutes,
    OneHour,
    TwentyFourHours,
    ManualRestore,
}

impl NifflerPauseDuration {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TenMinutes => "ten_minutes",
            Self::OneHour => "one_hour",
            Self::TwentyFourHours => "twenty_four_hours",
            Self::ManualRestore => "manual_restore",
        }
    }

    pub const fn seconds(self) -> Option<u64> {
        match self {
            Self::TenMinutes => Some(10 * 60),
            Self::OneHour => Some(60 * 60),
            Self::TwentyFourHours => Some(24 * 60 * 60),
            Self::ManualRestore => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerErrorResponseScope {
    Platform,
    Upstream,
}

impl NifflerErrorResponseScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Platform => "platform",
            Self::Upstream => "upstream",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum NifflerUpstreamErrorHandlingStep {
    RiskKeyword,
    ContactOrMarketingReplacement,
    StatusCodeMessage,
    DefaultUpstreamMessage,
}

impl NifflerUpstreamErrorHandlingStep {
    pub const fn priority(self) -> u8 {
        match self {
            Self::RiskKeyword => 10,
            Self::ContactOrMarketingReplacement => 20,
            Self::StatusCodeMessage => 30,
            Self::DefaultUpstreamMessage => 40,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerUserResponseMode {
    Replace,
    Append,
    Redact,
}

impl NifflerUserResponseMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Replace => "replace",
            Self::Append => "append",
            Self::Redact => "redact",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerPriceSourcePreference {
    Official,
    Upstream,
}

impl NifflerPriceSourcePreference {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Official => "official",
            Self::Upstream => "upstream",
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerUpstreamService {
    pub id: String,
    pub display_name: String,
    pub service_kind: String,
    pub default_api_format: Option<String>,
    pub base_url: Option<String>,
    pub cost_multiplier: f64,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerUpstreamService {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("upstream_services.id", &self.id)?;
        validate_required("upstream_services.display_name", &self.display_name)?;
        validate_required("upstream_services.service_kind", &self.service_kind)?;
        validate_multiplier("upstream_services.cost_multiplier", self.cost_multiplier)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerUpstreamAccount {
    pub id: String,
    pub upstream_service_id: String,
    pub display_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub auth_kind: String,
    pub status: NifflerAccountStatus,
    pub cost_multiplier: f64,
    pub priority: i32,
    pub cooldown_until_unix_ms: Option<u64>,
    pub last_tested_at_unix_ms: Option<u64>,
    pub last_test_error: Option<String>,
    pub config: Option<serde_json::Value>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerUpstreamAccount {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("upstream_accounts.id", &self.id)?;
        validate_required(
            "upstream_accounts.upstream_service_id",
            &self.upstream_service_id,
        )?;
        validate_required("upstream_accounts.display_name", &self.display_name)?;
        validate_required("upstream_accounts.auth_kind", &self.auth_kind)?;
        validate_multiplier("upstream_accounts.cost_multiplier", self.cost_multiplier)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerProductPlan {
    pub id: String,
    pub display_name: String,
    pub is_public: bool,
    pub is_active: bool,
    pub sales_multiplier: f64,
    pub description: Option<String>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerProductPlan {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("product_plans.id", &self.id)?;
        validate_required("product_plans.display_name", &self.display_name)?;
        validate_multiplier("product_plans.sales_multiplier", self.sales_multiplier)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerProductPlanModel {
    pub id: String,
    pub product_plan_id: String,
    pub model_name: String,
    pub is_enabled: bool,
    pub sales_multiplier_override: Option<f64>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerProductPlanModel {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("product_plan_models.id", &self.id)?;
        validate_required("product_plan_models.product_plan_id", &self.product_plan_id)?;
        validate_required("product_plan_models.model_name", &self.model_name)?;
        validate_optional_non_negative(
            "product_plan_models.sales_multiplier_override",
            self.sales_multiplier_override,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerModelBasePrice {
    pub id: String,
    pub model_name: String,
    pub input_price_per_million: f64,
    pub output_price_per_million: f64,
    pub cache_write_price_per_million: Option<f64>,
    pub cache_read_price_per_million: Option<f64>,
    pub source: String,
    pub effective_from_unix_ms: u64,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerModelBasePrice {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("model_base_prices.id", &self.id)?;
        validate_required("model_base_prices.model_name", &self.model_name)?;
        validate_required("model_base_prices.source", &self.source)?;
        validate_non_negative(
            "model_base_prices.input_price_per_million",
            self.input_price_per_million,
        )?;
        validate_non_negative(
            "model_base_prices.output_price_per_million",
            self.output_price_per_million,
        )?;
        if let Some(value) = self.cache_write_price_per_million {
            validate_non_negative("model_base_prices.cache_write_price_per_million", value)?;
        }
        if let Some(value) = self.cache_read_price_per_million {
            validate_non_negative("model_base_prices.cache_read_price_per_million", value)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerUpstreamModelPrice {
    pub id: String,
    pub upstream_service_id: String,
    pub model_name: String,
    pub upstream_input_price_per_million: Option<f64>,
    pub upstream_output_price_per_million: Option<f64>,
    pub upstream_cache_write_price_per_million: Option<f64>,
    pub upstream_cache_read_price_per_million: Option<f64>,
    pub price_source_preference: NifflerPriceSourcePreference,
    pub source: Option<String>,
    pub synced_at_unix_ms: Option<u64>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerUpstreamModelPrice {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("upstream_model_prices.id", &self.id)?;
        validate_required(
            "upstream_model_prices.upstream_service_id",
            &self.upstream_service_id,
        )?;
        validate_required("upstream_model_prices.model_name", &self.model_name)?;
        validate_optional_non_negative(
            "upstream_model_prices.upstream_input_price_per_million",
            self.upstream_input_price_per_million,
        )?;
        validate_optional_non_negative(
            "upstream_model_prices.upstream_output_price_per_million",
            self.upstream_output_price_per_million,
        )?;
        validate_optional_non_negative(
            "upstream_model_prices.upstream_cache_write_price_per_million",
            self.upstream_cache_write_price_per_million,
        )?;
        validate_optional_non_negative(
            "upstream_model_prices.upstream_cache_read_price_per_million",
            self.upstream_cache_read_price_per_million,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerAccountModelCapability {
    pub id: String,
    pub upstream_service_id: String,
    pub upstream_account_id: String,
    pub model_name: String,
    pub is_enabled: bool,
    pub source: Option<String>,
    pub last_checked_at_unix_ms: Option<u64>,
    pub last_error: Option<String>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerAccountModelCapability {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("account_model_capabilities.id", &self.id)?;
        validate_required(
            "account_model_capabilities.upstream_service_id",
            &self.upstream_service_id,
        )?;
        validate_required(
            "account_model_capabilities.upstream_account_id",
            &self.upstream_account_id,
        )?;
        validate_required("account_model_capabilities.model_name", &self.model_name)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerRouteAttempt {
    pub id: String,
    pub request_id: String,
    pub upstream_service_id: Option<String>,
    pub upstream_account_id: Option<String>,
    pub product_plan_id: Option<String>,
    pub model_name: String,
    pub attempt_index: u32,
    pub status: String,
    pub skip_reason: Option<String>,
    pub upstream_status_code: Option<u16>,
    pub latency_ms: Option<u64>,
    pub created_at_unix_ms: u64,
}

impl StoredNifflerRouteAttempt {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("route_attempts.id", &self.id)?;
        validate_required("route_attempts.request_id", &self.request_id)?;
        validate_required("route_attempts.model_name", &self.model_name)?;
        validate_required("route_attempts.status", &self.status)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerAccountRiskEvent {
    pub id: String,
    pub upstream_service_id: Option<String>,
    pub upstream_account_id: String,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub model_name: Option<String>,
    pub rule_id: Option<String>,
    pub matched_text: Option<String>,
    pub upstream_status_code: Option<u16>,
    pub action: NifflerAccountProtectionAction,
    pub created_at_unix_ms: u64,
}

impl StoredNifflerAccountRiskEvent {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("account_risk_events.id", &self.id)?;
        validate_required(
            "account_risk_events.upstream_account_id",
            &self.upstream_account_id,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerApiKeyPause {
    pub id: String,
    pub api_key_id: String,
    pub reason_code: String,
    pub user_message: String,
    pub paused_until_unix_ms: Option<u64>,
    pub manual_restore_required: bool,
    pub created_at_unix_ms: u64,
    pub restored_at_unix_ms: Option<u64>,
    pub restored_by: Option<String>,
}

impl StoredNifflerApiKeyPause {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("api_key_pauses.id", &self.id)?;
        validate_required("api_key_pauses.api_key_id", &self.api_key_id)?;
        validate_required("api_key_pauses.reason_code", &self.reason_code)?;
        validate_required("api_key_pauses.user_message", &self.user_message)?;
        if !self.manual_restore_required && self.paused_until_unix_ms.is_none() {
            return Err(crate::DataLayerError::InvalidInput(
                "api key pause must include paused_until_unix_ms unless manual restore is required"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerErrorReturnSetting {
    pub id: String,
    pub scope: NifflerErrorResponseScope,
    pub upstream_service_id: Option<String>,
    pub match_status_code: Option<u16>,
    pub match_text: Option<String>,
    pub handling_step: Option<NifflerUpstreamErrorHandlingStep>,
    pub response_mode: NifflerUserResponseMode,
    pub user_message: String,
    pub account_protection_action: NifflerAccountProtectionAction,
    pub pause_duration: Option<NifflerPauseDuration>,
    pub is_active: bool,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerErrorReturnSetting {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("error_return_settings.id", &self.id)?;
        validate_required("error_return_settings.user_message", &self.user_message)?;
        if self.scope == NifflerErrorResponseScope::Upstream && self.handling_step.is_none() {
            return Err(crate::DataLayerError::InvalidInput(
                "upstream error setting must include handling_step".to_string(),
            ));
        }
        if self.account_protection_action == NifflerAccountProtectionAction::PauseScheduling
            && self.pause_duration.is_none()
        {
            return Err(crate::DataLayerError::InvalidInput(
                "pause_scheduling action must include pause_duration".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NifflerDefaultErrorMessage {
    pub code: &'static str,
    pub title: &'static str,
    pub user_message: &'static str,
}

pub const NIFFLER_DEFAULT_PLATFORM_ERROR_MESSAGES: &[NifflerDefaultErrorMessage] = &[
    NifflerDefaultErrorMessage {
        code: "invalid_api_key",
        title: "API Key 无效",
        user_message: "API Key 无效，请检查后重新请求。",
    },
    NifflerDefaultErrorMessage {
        code: "api_key_paused",
        title: "API Key 已暂停",
        user_message: "当前 API Key 已暂停使用，请联系平台客服确认原因。",
    },
    NifflerDefaultErrorMessage {
        code: "insufficient_balance",
        title: "余额不足",
        user_message: "账户余额不足，请充值后再试。",
    },
    NifflerDefaultErrorMessage {
        code: "plan_quota_exhausted",
        title: "套餐额度不足",
        user_message: "当前套餐额度不足，请升级套餐或使用钱包余额。",
    },
    NifflerDefaultErrorMessage {
        code: "model_not_allowed",
        title: "模型不可用",
        user_message: "当前 API Key 无权使用这个模型，请检查绑定的产品策略。",
    },
    NifflerDefaultErrorMessage {
        code: "no_available_account",
        title: "暂无可用账号",
        user_message: "当前模型暂无可用上游账号，请稍后重试。",
    },
    NifflerDefaultErrorMessage {
        code: "rate_limited",
        title: "请求过于频繁",
        user_message: "请求过于频繁，请稍后再试。",
    },
    NifflerDefaultErrorMessage {
        code: "platform_maintenance",
        title: "平台维护中",
        user_message: "平台正在维护，请稍后再试。",
    },
];

pub fn default_platform_error_message(code: &str) -> Option<&'static NifflerDefaultErrorMessage> {
    NIFFLER_DEFAULT_PLATFORM_ERROR_MESSAGES
        .iter()
        .find(|message| message.code == code)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerReadinessSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerShadowTableStatus {
    pub database_driver: Option<String>,
    pub expected_tables: u64,
    pub existing_tables: u64,
    pub all_present: bool,
    pub tables: Vec<NifflerShadowTableItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerShadowTableItem {
    pub table_name: String,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerCoreReadinessSummary {
    pub providers_total: u64,
    pub providers_active: u64,
    pub provider_keys_total: u64,
    pub provider_keys_active: u64,
    pub routing_groups_total: u64,
    pub routing_groups_enabled: u64,
    pub global_models_total: u64,
    pub global_models_active: u64,
    pub recent_problem_usage_sample_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerCoreMappingSummary {
    pub legacy_count: u64,
    pub mapped_count: u64,
    pub blocked_count: u64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerDisabledProviderReference {
    pub routing_group_id: String,
    pub routing_group_name: String,
    pub provider_id: String,
    pub provider_name: String,
    pub source_field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerKeyScopeResidue {
    pub subject_kind: String,
    pub key_id: String,
    pub key_name: Option<String>,
    pub owner_label: Option<String>,
    pub residue_fields: Vec<String>,
    pub impact: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerGroupPolicyGap {
    pub routing_group_id: String,
    pub routing_group_name: String,
    pub gap_kind: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerPriceGap {
    pub scope: String,
    pub provider_id: Option<String>,
    pub provider_name: Option<String>,
    pub model_id: Option<String>,
    pub model_name: String,
    pub missing_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NifflerUsageAnomaly {
    pub usage_id: String,
    pub request_id: String,
    pub created_at_unix_ms: u64,
    pub provider_name: String,
    pub provider_id: Option<String>,
    pub provider_api_key_id: Option<String>,
    pub model: String,
    pub status: String,
    pub billing_status: String,
    pub status_code: Option<u16>,
    pub error_category: Option<String>,
    pub diagnosis: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerRouteSkipReasonSummary {
    pub reason: String,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerReadinessIssue {
    pub severity: NifflerReadinessSeverity,
    pub code: String,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NifflerCoreReadinessReport {
    pub schema_version: u32,
    pub generated_at_unix_secs: u64,
    pub recent_days: u32,
    pub shadow_tables: NifflerShadowTableStatus,
    pub summary: NifflerCoreReadinessSummary,
    pub provider_mapping: NifflerCoreMappingSummary,
    pub account_mapping: NifflerCoreMappingSummary,
    pub product_plan_mapping: NifflerCoreMappingSummary,
    pub provider_status_counts: BTreeMap<String, u64>,
    pub account_status_counts: BTreeMap<String, u64>,
    pub disabled_provider_references: Vec<NifflerDisabledProviderReference>,
    pub key_scope_residue: Vec<NifflerKeyScopeResidue>,
    pub group_policy_gaps: Vec<NifflerGroupPolicyGap>,
    pub price_gaps: Vec<NifflerPriceGap>,
    pub recent_usage_anomalies: Vec<NifflerUsageAnomaly>,
    pub route_skip_reasons: Vec<NifflerRouteSkipReasonSummary>,
    pub issues: Vec<NifflerReadinessIssue>,
}

fn validate_required(field: &str, value: &str) -> Result<(), crate::DataLayerError> {
    if value.trim().is_empty() {
        return Err(crate::DataLayerError::InvalidInput(format!(
            "{field} cannot be empty"
        )));
    }
    Ok(())
}

fn validate_multiplier(field: &str, value: f64) -> Result<(), crate::DataLayerError> {
    if !value.is_finite() || value < 0.0 {
        return Err(crate::DataLayerError::InvalidInput(format!(
            "{field} must be a non-negative finite number"
        )));
    }
    Ok(())
}

fn validate_non_negative(field: &str, value: f64) -> Result<(), crate::DataLayerError> {
    if !value.is_finite() || value < 0.0 {
        return Err(crate::DataLayerError::InvalidInput(format!(
            "{field} must be a non-negative finite number"
        )));
    }
    Ok(())
}

fn validate_optional_non_negative(
    field: &str,
    value: Option<f64>,
) -> Result<(), crate::DataLayerError> {
    if let Some(value) = value {
        validate_non_negative(field, value)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        default_platform_error_message, NifflerAccountProtectionAction, NifflerAccountStatus,
        NifflerPauseDuration, NifflerPriceSourcePreference, NifflerUpstreamErrorHandlingStep,
        StoredNifflerApiKeyPause, StoredNifflerErrorReturnSetting, StoredNifflerProductPlan,
        StoredNifflerProductPlanModel, StoredNifflerUpstreamModelPrice,
    };

    #[test]
    fn account_status_scheduling_policy_is_explicit() {
        assert!(NifflerAccountStatus::Available.allows_scheduling());
        assert!(!NifflerAccountStatus::CoolingDown.allows_scheduling());
        assert!(!NifflerAccountStatus::Disabled.allows_scheduling());
        assert!(!NifflerAccountStatus::Invalid.allows_scheduling());
        assert!(!NifflerAccountStatus::QuotaExhausted.allows_scheduling());
    }

    #[test]
    fn upstream_error_steps_keep_business_order() {
        assert!(
            NifflerUpstreamErrorHandlingStep::RiskKeyword.priority()
                < NifflerUpstreamErrorHandlingStep::ContactOrMarketingReplacement.priority()
        );
        assert!(
            NifflerUpstreamErrorHandlingStep::ContactOrMarketingReplacement.priority()
                < NifflerUpstreamErrorHandlingStep::StatusCodeMessage.priority()
        );
        assert!(
            NifflerUpstreamErrorHandlingStep::StatusCodeMessage.priority()
                < NifflerUpstreamErrorHandlingStep::DefaultUpstreamMessage.priority()
        );
    }

    #[test]
    fn pause_durations_are_fixed() {
        assert_eq!(NifflerPauseDuration::TenMinutes.seconds(), Some(600));
        assert_eq!(NifflerPauseDuration::OneHour.seconds(), Some(3600));
        assert_eq!(NifflerPauseDuration::TwentyFourHours.seconds(), Some(86400));
        assert_eq!(NifflerPauseDuration::ManualRestore.seconds(), None);
    }

    #[test]
    fn platform_error_messages_include_common_local_failures() {
        let message = default_platform_error_message("insufficient_balance")
            .expect("insufficient balance message");
        assert_eq!(message.title, "余额不足");
        assert!(default_platform_error_message("missing").is_none());
    }

    #[test]
    fn product_plan_rejects_invalid_multiplier() {
        let plan = StoredNifflerProductPlan {
            id: "plan-1".to_string(),
            display_name: "默认策略".to_string(),
            is_public: true,
            is_active: true,
            sales_multiplier: f64::NAN,
            description: None,
            created_at_unix_ms: 0,
            updated_at_unix_ms: 0,
        };
        assert!(plan.validate().is_err());
    }

    #[test]
    fn upstream_model_price_rejects_negative_synced_price() {
        let price = StoredNifflerUpstreamModelPrice {
            id: "price-1".to_string(),
            upstream_service_id: "service-1".to_string(),
            model_name: "model-1".to_string(),
            upstream_input_price_per_million: Some(-1.0),
            upstream_output_price_per_million: None,
            upstream_cache_write_price_per_million: None,
            upstream_cache_read_price_per_million: None,
            price_source_preference: NifflerPriceSourcePreference::Upstream,
            source: Some("sync".to_string()),
            synced_at_unix_ms: Some(1),
            created_at_unix_ms: 1,
            updated_at_unix_ms: 1,
        };
        assert!(price.validate().is_err());
    }

    #[test]
    fn product_plan_model_rejects_negative_sales_override() {
        let model = StoredNifflerProductPlanModel {
            id: "plan-model-1".to_string(),
            product_plan_id: "plan-1".to_string(),
            model_name: "model-1".to_string(),
            is_enabled: true,
            sales_multiplier_override: Some(-0.1),
            created_at_unix_ms: 1,
            updated_at_unix_ms: 1,
        };
        assert!(model.validate().is_err());
    }

    #[test]
    fn pause_action_requires_duration() {
        let setting = StoredNifflerErrorReturnSetting {
            id: "setting-1".to_string(),
            scope: super::NifflerErrorResponseScope::Platform,
            upstream_service_id: None,
            match_status_code: None,
            match_text: None,
            handling_step: None,
            response_mode: super::NifflerUserResponseMode::Replace,
            user_message: "请求已暂停".to_string(),
            account_protection_action: NifflerAccountProtectionAction::PauseScheduling,
            pause_duration: None,
            is_active: true,
            created_at_unix_ms: 0,
            updated_at_unix_ms: 0,
        };
        assert!(setting.validate().is_err());
    }

    #[test]
    fn api_key_pause_requires_expiry_or_manual_restore() {
        let pause = StoredNifflerApiKeyPause {
            id: "pause-1".to_string(),
            api_key_id: "key-1".to_string(),
            reason_code: "risk_keyword".to_string(),
            user_message: "当前 API Key 已暂停".to_string(),
            paused_until_unix_ms: None,
            manual_restore_required: false,
            created_at_unix_ms: 1,
            restored_at_unix_ms: None,
            restored_by: None,
        };
        assert!(pause.validate().is_err());
    }
}
