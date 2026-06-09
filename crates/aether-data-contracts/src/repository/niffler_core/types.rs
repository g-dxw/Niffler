use std::collections::BTreeMap;

use async_trait::async_trait;

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
    pub fn from_database(value: &str) -> Result<Self, crate::DataLayerError> {
        match value {
            "available" => Ok(Self::Available),
            "disabled" => Ok(Self::Disabled),
            "invalid" => Ok(Self::Invalid),
            "quota_exhausted" => Ok(Self::QuotaExhausted),
            "cooling_down" => Ok(Self::CoolingDown),
            _ => Err(crate::DataLayerError::UnexpectedValue(format!(
                "unknown niffler account status: {value}"
            ))),
        }
    }

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
    pub fn from_database(value: &str) -> Result<Self, crate::DataLayerError> {
        match value {
            "record_only" => Ok(Self::RecordOnly),
            "pause_scheduling" => Ok(Self::PauseScheduling),
            "disable_account" => Ok(Self::DisableAccount),
            _ => Err(crate::DataLayerError::UnexpectedValue(format!(
                "unknown niffler account protection action: {value}"
            ))),
        }
    }

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
    pub fn from_database(value: &str) -> Result<Self, crate::DataLayerError> {
        match value {
            "ten_minutes" => Ok(Self::TenMinutes),
            "one_hour" => Ok(Self::OneHour),
            "twenty_four_hours" => Ok(Self::TwentyFourHours),
            "manual_restore" => Ok(Self::ManualRestore),
            _ => Err(crate::DataLayerError::UnexpectedValue(format!(
                "unknown niffler pause duration: {value}"
            ))),
        }
    }

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
    pub fn from_database(value: &str) -> Result<Self, crate::DataLayerError> {
        match value {
            "platform" => Ok(Self::Platform),
            "upstream" => Ok(Self::Upstream),
            _ => Err(crate::DataLayerError::UnexpectedValue(format!(
                "unknown niffler error response scope: {value}"
            ))),
        }
    }

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
    pub fn from_database(value: &str) -> Result<Self, crate::DataLayerError> {
        match value {
            "risk_keyword" => Ok(Self::RiskKeyword),
            "contact_or_marketing_replacement" => Ok(Self::ContactOrMarketingReplacement),
            "status_code_message" => Ok(Self::StatusCodeMessage),
            "default_upstream_message" => Ok(Self::DefaultUpstreamMessage),
            _ => Err(crate::DataLayerError::UnexpectedValue(format!(
                "unknown niffler upstream error handling step: {value}"
            ))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RiskKeyword => "risk_keyword",
            Self::ContactOrMarketingReplacement => "contact_or_marketing_replacement",
            Self::StatusCodeMessage => "status_code_message",
            Self::DefaultUpstreamMessage => "default_upstream_message",
        }
    }

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
    pub fn from_database(value: &str) -> Result<Self, crate::DataLayerError> {
        match value {
            "replace" => Ok(Self::Replace),
            "append" => Ok(Self::Append),
            "redact" => Ok(Self::Redact),
            _ => Err(crate::DataLayerError::UnexpectedValue(format!(
                "unknown niffler user response mode: {value}"
            ))),
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerProtocolKind {
    Openai,
    Anthropic,
    Gemini,
    Codex,
    Custom,
}

impl NifflerProtocolKind {
    pub fn from_database(value: &str) -> Result<Self, crate::DataLayerError> {
        match value {
            "openai" => Ok(Self::Openai),
            "anthropic" => Ok(Self::Anthropic),
            "gemini" => Ok(Self::Gemini),
            "codex" => Ok(Self::Codex),
            "custom" => Ok(Self::Custom),
            _ => Err(crate::DataLayerError::UnexpectedValue(format!(
                "unknown niffler protocol kind: {value}"
            ))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Openai => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::Codex => "codex",
            Self::Custom => "custom",
        }
    }

    pub const fn supports_openai_responses_image_tool(self) -> bool {
        matches!(self, Self::Openai | Self::Codex)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerServiceCapabilityKind {
    Text,
    Streaming,
    ImagesEndpoint,
    OpenaiResponsesImageTool,
    ModelList,
    ModelTest,
}

impl NifflerServiceCapabilityKind {
    pub fn from_database(value: &str) -> Result<Self, crate::DataLayerError> {
        match value {
            "text" => Ok(Self::Text),
            "streaming" => Ok(Self::Streaming),
            "images_endpoint" => Ok(Self::ImagesEndpoint),
            "openai_responses_image_tool" => Ok(Self::OpenaiResponsesImageTool),
            "model_list" => Ok(Self::ModelList),
            "model_test" => Ok(Self::ModelTest),
            _ => Err(crate::DataLayerError::UnexpectedValue(format!(
                "unknown niffler service capability kind: {value}"
            ))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Streaming => "streaming",
            Self::ImagesEndpoint => "images_endpoint",
            Self::OpenaiResponsesImageTool => "openai_responses_image_tool",
            Self::ModelList => "model_list",
            Self::ModelTest => "model_test",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerRuntimeRolloutTargetScope {
    ApiKey,
    ProductPlan,
}

impl NifflerRuntimeRolloutTargetScope {
    pub fn from_database(value: &str) -> Result<Self, crate::DataLayerError> {
        match value {
            "api_key" => Ok(Self::ApiKey),
            "product_plan" => Ok(Self::ProductPlan),
            _ => Err(crate::DataLayerError::UnexpectedValue(format!(
                "unknown niffler runtime rollout target scope: {value}"
            ))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApiKey => "api_key",
            Self::ProductPlan => "product_plan",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerBillingReservationStatus {
    Active,
    Settled,
    Released,
    Expired,
    ManualReview,
}

impl NifflerBillingReservationStatus {
    pub fn from_database(value: &str) -> Result<Self, crate::DataLayerError> {
        match value {
            "active" => Ok(Self::Active),
            "settled" => Ok(Self::Settled),
            "released" => Ok(Self::Released),
            "expired" => Ok(Self::Expired),
            "manual_review" => Ok(Self::ManualReview),
            _ => Err(crate::DataLayerError::UnexpectedValue(format!(
                "unknown niffler billing reservation status: {value}"
            ))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Settled => "settled",
            Self::Released => "released",
            Self::Expired => "expired",
            Self::ManualReview => "manual_review",
        }
    }

    pub const fn is_open(self) -> bool {
        matches!(self, Self::Active | Self::ManualReview)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerBillingReservationEventKind {
    Reserved,
    Settled,
    Released,
    Expired,
    ManualReview,
}

impl NifflerBillingReservationEventKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reserved => "reserved",
            Self::Settled => "settled",
            Self::Released => "released",
            Self::Expired => "expired",
            Self::ManualReview => "manual_review",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerReferralRewardRuleStatus {
    Active,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerReferralRewardKind {
    FixedAmount,
    Percentage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerReferralRewardLedgerStatus {
    Pending,
    Paid,
    Failed,
    Cancelled,
}

impl NifflerReferralRewardLedgerStatus {
    pub fn from_database(value: &str) -> Result<Self, crate::DataLayerError> {
        match value {
            "pending" => Ok(Self::Pending),
            "paid" => Ok(Self::Paid),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(crate::DataLayerError::UnexpectedValue(format!(
                "unknown niffler referral reward ledger status: {value}"
            ))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Paid => "paid",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NifflerReferralRewardEventKind {
    Created,
    Paid,
    Failed,
    RetryScheduled,
    ManualRetry,
    ManualPaid,
    Cancelled,
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
pub struct StoredNifflerUpstreamServiceCapability {
    pub id: String,
    pub upstream_service_id: String,
    pub protocol_kind: NifflerProtocolKind,
    pub capability_kind: NifflerServiceCapabilityKind,
    pub is_enabled: bool,
    pub config: Option<serde_json::Value>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerUpstreamServiceCapability {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("upstream_service_capabilities.id", &self.id)?;
        validate_required(
            "upstream_service_capabilities.upstream_service_id",
            &self.upstream_service_id,
        )?;
        if self.is_enabled
            && self.capability_kind == NifflerServiceCapabilityKind::OpenaiResponsesImageTool
            && !self.protocol_kind.supports_openai_responses_image_tool()
        {
            return Err(crate::DataLayerError::InvalidInput(
                "openai_responses_image_tool can only be used by openai or codex protocol"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NifflerUpstreamServiceCapabilityListQuery {
    pub upstream_service_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerUpstreamServiceCapabilityListPage {
    pub items: Vec<StoredNifflerUpstreamServiceCapability>,
    pub total: usize,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerUpstreamServiceListQuery {
    pub include_inactive: bool,
    pub search: Option<String>,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerUpstreamServiceListPage {
    pub items: Vec<StoredNifflerUpstreamService>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerUpstreamAccountListQuery {
    pub upstream_service_id: Option<String>,
    pub status: Option<NifflerAccountStatus>,
    pub search: Option<String>,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerUpstreamAccountListPage {
    pub items: Vec<StoredNifflerUpstreamAccount>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNifflerUpstreamServiceRecord {
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

impl CreateNifflerUpstreamServiceRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("upstream_services.id", &self.id)?;
        validate_required("upstream_services.display_name", &self.display_name)?;
        validate_required("upstream_services.service_kind", &self.service_kind)?;
        validate_multiplier("upstream_services.cost_multiplier", self.cost_multiplier)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNifflerUpstreamAccountRecord {
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

impl CreateNifflerUpstreamAccountRecord {
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

#[derive(Debug, Clone, PartialEq)]
pub struct UpsertNifflerUpstreamServiceCapabilityRecord {
    pub id: String,
    pub upstream_service_id: String,
    pub protocol_kind: NifflerProtocolKind,
    pub capability_kind: NifflerServiceCapabilityKind,
    pub is_enabled: bool,
    pub config: Option<serde_json::Value>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl UpsertNifflerUpstreamServiceCapabilityRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        StoredNifflerUpstreamServiceCapability {
            id: self.id.clone(),
            upstream_service_id: self.upstream_service_id.clone(),
            protocol_kind: self.protocol_kind,
            capability_kind: self.capability_kind,
            is_enabled: self.is_enabled,
            config: self.config.clone(),
            created_at_unix_ms: self.created_at_unix_ms,
            updated_at_unix_ms: self.updated_at_unix_ms,
        }
        .validate()
    }
}

#[async_trait]
pub trait NifflerCoreReadRepository: Send + Sync {
    async fn list_upstream_services(
        &self,
        query: &NifflerUpstreamServiceListQuery,
    ) -> Result<StoredNifflerUpstreamServiceListPage, crate::DataLayerError>;

    async fn find_upstream_service_by_id(
        &self,
        upstream_service_id: &str,
    ) -> Result<Option<StoredNifflerUpstreamService>, crate::DataLayerError>;

    async fn list_upstream_service_capabilities(
        &self,
        query: &NifflerUpstreamServiceCapabilityListQuery,
    ) -> Result<StoredNifflerUpstreamServiceCapabilityListPage, crate::DataLayerError>;

    async fn list_upstream_accounts(
        &self,
        query: &NifflerUpstreamAccountListQuery,
    ) -> Result<StoredNifflerUpstreamAccountListPage, crate::DataLayerError>;

    async fn find_upstream_account_by_id(
        &self,
        upstream_account_id: &str,
    ) -> Result<Option<StoredNifflerUpstreamAccount>, crate::DataLayerError>;

    async fn list_account_model_capabilities(
        &self,
        query: &NifflerAccountModelCapabilityListQuery,
    ) -> Result<StoredNifflerAccountModelCapabilityListPage, crate::DataLayerError>;

    async fn list_runtime_account_model_access(
        &self,
        query: &NifflerRuntimeAccountModelAccessListQuery,
    ) -> Result<StoredNifflerRuntimeAccountModelAccessListPage, crate::DataLayerError>;

    async fn list_product_plans(
        &self,
        query: &NifflerProductPlanListQuery,
    ) -> Result<StoredNifflerProductPlanListPage, crate::DataLayerError>;

    async fn find_product_plan_by_id(
        &self,
        product_plan_id: &str,
    ) -> Result<Option<StoredNifflerProductPlan>, crate::DataLayerError>;

    async fn list_product_plan_models(
        &self,
        query: &NifflerProductPlanModelListQuery,
    ) -> Result<StoredNifflerProductPlanModelListPage, crate::DataLayerError>;

    async fn list_api_key_product_plan_bindings(
        &self,
        query: &NifflerApiKeyProductPlanBindingListQuery,
    ) -> Result<StoredNifflerApiKeyProductPlanBindingListPage, crate::DataLayerError>;

    async fn find_api_key_product_plan_binding_by_api_key_id(
        &self,
        api_key_id: &str,
    ) -> Result<Option<StoredNifflerApiKeyProductPlanBinding>, crate::DataLayerError>;

    async fn list_runtime_rollout_settings(
        &self,
        query: &NifflerRuntimeRolloutSettingListQuery,
    ) -> Result<StoredNifflerRuntimeRolloutSettingListPage, crate::DataLayerError>;

    async fn find_runtime_rollout_setting(
        &self,
        target_scope: NifflerRuntimeRolloutTargetScope,
        target_id: &str,
    ) -> Result<Option<StoredNifflerRuntimeRolloutSetting>, crate::DataLayerError>;

    async fn list_error_return_settings(
        &self,
        query: &NifflerErrorReturnSettingListQuery,
    ) -> Result<StoredNifflerErrorReturnSettingListPage, crate::DataLayerError>;

    async fn list_settlement_snapshots(
        &self,
        query: &NifflerSettlementSnapshotListQuery,
    ) -> Result<StoredNifflerSettlementSnapshotListPage, crate::DataLayerError>;

    async fn list_billing_reservations(
        &self,
        query: &NifflerBillingReservationListQuery,
    ) -> Result<StoredNifflerBillingReservationListPage, crate::DataLayerError>;

    async fn sum_active_billing_reservation_wallet_usd(
        &self,
        user_id: Option<&str>,
        api_key_id: Option<&str>,
        now_unix_ms: u64,
    ) -> Result<f64, crate::DataLayerError>;

    async fn list_billing_reservation_dry_runs(
        &self,
        query: &NifflerBillingReservationDryRunListQuery,
    ) -> Result<StoredNifflerBillingReservationDryRunListPage, crate::DataLayerError>;

    async fn list_referral_reward_ledger(
        &self,
        query: &NifflerReferralRewardLedgerListQuery,
    ) -> Result<StoredNifflerReferralRewardLedgerListPage, crate::DataLayerError>;

    async fn list_route_attempts(
        &self,
        query: &NifflerRouteAttemptListQuery,
    ) -> Result<StoredNifflerRouteAttemptListPage, crate::DataLayerError>;

    async fn list_consistency_checks(
        &self,
        query: &NifflerConsistencyCheckListQuery,
    ) -> Result<StoredNifflerConsistencyCheckListPage, crate::DataLayerError>;

    async fn list_stability_observations(
        &self,
        query: &NifflerStabilityObservationListQuery,
    ) -> Result<StoredNifflerStabilityObservationListPage, crate::DataLayerError>;
}

#[async_trait]
pub trait NifflerCoreWriteRepository: Send + Sync {
    async fn create_upstream_service(
        &self,
        record: CreateNifflerUpstreamServiceRecord,
    ) -> Result<StoredNifflerUpstreamService, crate::DataLayerError>;

    async fn create_upstream_account(
        &self,
        record: CreateNifflerUpstreamAccountRecord,
    ) -> Result<StoredNifflerUpstreamAccount, crate::DataLayerError>;

    async fn upsert_upstream_service_capability(
        &self,
        record: UpsertNifflerUpstreamServiceCapabilityRecord,
    ) -> Result<StoredNifflerUpstreamServiceCapability, crate::DataLayerError>;

    async fn create_product_plan(
        &self,
        record: CreateNifflerProductPlanRecord,
    ) -> Result<StoredNifflerProductPlan, crate::DataLayerError>;

    async fn upsert_product_plan_model(
        &self,
        record: UpsertNifflerProductPlanModelRecord,
    ) -> Result<StoredNifflerProductPlanModel, crate::DataLayerError>;

    async fn upsert_api_key_product_plan_binding(
        &self,
        record: UpsertNifflerApiKeyProductPlanBindingRecord,
    ) -> Result<StoredNifflerApiKeyProductPlanBinding, crate::DataLayerError>;

    async fn upsert_runtime_rollout_setting(
        &self,
        record: UpsertNifflerRuntimeRolloutSettingRecord,
    ) -> Result<StoredNifflerRuntimeRolloutSetting, crate::DataLayerError>;

    async fn create_error_return_setting(
        &self,
        record: CreateNifflerErrorReturnSettingRecord,
    ) -> Result<StoredNifflerErrorReturnSetting, crate::DataLayerError>;

    async fn create_account_risk_event(
        &self,
        record: CreateNifflerAccountRiskEventRecord,
    ) -> Result<StoredNifflerAccountRiskEvent, crate::DataLayerError>;

    async fn create_settlement_snapshot(
        &self,
        record: CreateNifflerSettlementSnapshotRecord,
    ) -> Result<StoredNifflerSettlementSnapshot, crate::DataLayerError>;

    async fn create_billing_reservation(
        &self,
        record: CreateNifflerBillingReservationRecord,
    ) -> Result<StoredNifflerBillingReservation, crate::DataLayerError>;

    async fn finalize_billing_reservation_by_request_id(
        &self,
        record: FinalizeNifflerBillingReservationRecord,
    ) -> Result<Option<StoredNifflerBillingReservation>, crate::DataLayerError>;

    async fn create_billing_reservation_dry_run(
        &self,
        record: CreateNifflerBillingReservationDryRunRecord,
    ) -> Result<StoredNifflerBillingReservationDryRun, crate::DataLayerError>;

    async fn create_referral_reward_ledger(
        &self,
        record: CreateNifflerReferralRewardLedgerRecord,
    ) -> Result<StoredNifflerReferralRewardLedger, crate::DataLayerError>;

    async fn create_route_attempt(
        &self,
        record: CreateNifflerRouteAttemptRecord,
    ) -> Result<StoredNifflerRouteAttempt, crate::DataLayerError>;

    async fn upsert_stability_observation(
        &self,
        record: UpsertNifflerStabilityObservationRecord,
    ) -> Result<StoredNifflerStabilityObservation, crate::DataLayerError>;
}

pub trait NifflerCoreRepository: NifflerCoreReadRepository + NifflerCoreWriteRepository {}

impl<T> NifflerCoreRepository for T where T: NifflerCoreReadRepository + NifflerCoreWriteRepository {}

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
pub struct StoredNifflerApiKeyProductPlanBinding {
    pub id: String,
    pub api_key_id: String,
    pub product_plan_id: String,
    pub config: Option<serde_json::Value>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerApiKeyProductPlanBinding {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("api_key_product_plan_bindings.id", &self.id)?;
        validate_required("api_key_product_plan_bindings.api_key_id", &self.api_key_id)?;
        validate_required(
            "api_key_product_plan_bindings.product_plan_id",
            &self.product_plan_id,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerRuntimeRolloutSetting {
    pub id: String,
    pub target_scope: NifflerRuntimeRolloutTargetScope,
    pub target_id: String,
    pub enable_new_routing: bool,
    pub enable_settlement_snapshot: bool,
    pub enable_error_return_rules: bool,
    pub enable_billing_reservation: bool,
    pub enable_referral_ledger: bool,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerRuntimeRolloutSetting {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("runtime_rollout_settings.id", &self.id)?;
        validate_required("runtime_rollout_settings.target_id", &self.target_id)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerProductPlanListQuery {
    pub include_inactive: bool,
    pub public_only: bool,
    pub search: Option<String>,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerProductPlanListPage {
    pub items: Vec<StoredNifflerProductPlan>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerProductPlanModelListQuery {
    pub product_plan_id: String,
    pub enabled_only: bool,
    pub search: Option<String>,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerProductPlanModelListPage {
    pub items: Vec<StoredNifflerProductPlanModel>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerApiKeyProductPlanBindingListQuery {
    pub product_plan_id: Option<String>,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerApiKeyProductPlanBindingListPage {
    pub items: Vec<StoredNifflerApiKeyProductPlanBinding>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerRuntimeRolloutSettingListQuery {
    pub target_scope: Option<NifflerRuntimeRolloutTargetScope>,
    pub include_inactive: bool,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerRuntimeRolloutSettingListPage {
    pub items: Vec<StoredNifflerRuntimeRolloutSetting>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerSettlementSnapshotListQuery {
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub product_plan_id: Option<String>,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerSettlementSnapshotListPage {
    pub items: Vec<StoredNifflerSettlementSnapshotListItem>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerSettlementSnapshotListItem {
    pub id: String,
    pub request_id: String,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub product_plan_id: Option<String>,
    pub product_plan_name: Option<String>,
    pub upstream_service_id: Option<String>,
    pub upstream_service_name: Option<String>,
    pub upstream_account_id: Option<String>,
    pub upstream_account_display_name: Option<String>,
    pub upstream_account_email: Option<String>,
    pub upstream_account_phone: Option<String>,
    pub requested_model_name: String,
    pub upstream_execution_model_name: Option<String>,
    pub image_tool_model_name: Option<String>,
    pub pricing_snapshot: serde_json::Value,
    pub wallet_charge_usd: f64,
    pub entitlement_charge_usd: f64,
    pub upstream_cost_usd: f64,
    pub gross_margin_usd: f64,
    pub created_at_unix_ms: u64,
    pub finalized_at_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerBillingReservationListQuery {
    pub status: Option<NifflerBillingReservationStatus>,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub request_id: Option<String>,
    pub expires_at_gte_unix_ms: Option<u64>,
    pub expires_at_lte_unix_ms: Option<u64>,
    pub expires_at_lt_unix_ms: Option<u64>,
    pub finalized_at_gte_unix_ms: Option<u64>,
    pub finalized_at_lt_unix_ms: Option<u64>,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerBillingReservationListPage {
    pub items: Vec<StoredNifflerBillingReservation>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerBillingReservationDryRunListQuery {
    pub status: Option<String>,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub product_plan_id: Option<String>,
    pub request_id: Option<String>,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerBillingReservationDryRunListPage {
    pub items: Vec<StoredNifflerBillingReservationDryRun>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerReferralRewardLedgerListQuery {
    pub status: Option<NifflerReferralRewardLedgerStatus>,
    pub inviter_user_id: Option<String>,
    pub invitee_user_id: Option<String>,
    pub order_id: Option<String>,
    pub updated_at_gte_unix_ms: Option<u64>,
    pub updated_at_lt_unix_ms: Option<u64>,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerReferralRewardLedgerListPage {
    pub items: Vec<StoredNifflerReferralRewardLedger>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerRouteAttemptListQuery {
    pub request_id: Option<String>,
    pub upstream_service_id: Option<String>,
    pub upstream_account_id: Option<String>,
    pub status: Option<String>,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerRouteAttemptListPage {
    pub items: Vec<StoredNifflerRouteAttemptListItem>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerRouteAttemptListItem {
    pub id: String,
    pub request_id: String,
    pub upstream_service_id: Option<String>,
    pub upstream_service_name: Option<String>,
    pub upstream_account_id: Option<String>,
    pub upstream_account_display_name: Option<String>,
    pub upstream_account_email: Option<String>,
    pub upstream_account_phone: Option<String>,
    pub product_plan_id: Option<String>,
    pub product_plan_name: Option<String>,
    pub model_name: String,
    pub attempt_index: u32,
    pub status: String,
    pub skip_reason: Option<String>,
    pub upstream_status_code: Option<u16>,
    pub latency_ms: Option<u64>,
    pub created_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerConsistencyCheckListQuery {
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub product_plan_id: Option<String>,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerConsistencyCheckListPage {
    pub items: Vec<StoredNifflerConsistencyCheckItem>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerConsistencyCheckItem {
    pub request_id: String,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub product_plan_id: Option<String>,
    pub product_plan_name: Option<String>,
    pub usage_status: Option<String>,
    pub usage_billing_status: Option<String>,
    pub usage_total_cost_usd: Option<f64>,
    pub legacy_wallet_charge_usd: Option<f64>,
    pub legacy_entitlement_charge_usd: f64,
    pub niffler_wallet_charge_usd: f64,
    pub niffler_entitlement_charge_usd: f64,
    pub niffler_total_charge_usd: f64,
    pub wallet_difference_usd: Option<f64>,
    pub entitlement_difference_usd: f64,
    pub total_difference_usd: Option<f64>,
    pub reservation_id: Option<String>,
    pub reservation_status: Option<NifflerBillingReservationStatus>,
    pub reservation_release_reason: Option<String>,
    pub route_attempt_count: u64,
    pub successful_route_attempt_count: u64,
    pub issue_codes: Vec<String>,
    pub consistency_status: String,
    pub created_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerStabilityObservationListQuery {
    pub status: Option<String>,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerStabilityObservationListPage {
    pub items: Vec<StoredNifflerStabilityObservation>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerStabilityObservation {
    pub id: String,
    pub window_start_unix_ms: u64,
    pub window_end_unix_ms: u64,
    pub status: String,
    pub rollback_drill_status: String,
    pub consistency_checked_count: u64,
    pub consistency_issue_count: u64,
    pub unknown_upstream_count: u64,
    pub legacy_write_call_count: u64,
    pub billing_reservation_exception_count: u64,
    pub referral_exception_count: u64,
    pub blocker_codes: Vec<String>,
    pub summary: Option<serde_json::Value>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerStabilityObservation {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("stability_observations.id", &self.id)?;
        validate_stability_status(&self.status)?;
        validate_rollback_drill_status(&self.rollback_drill_status)?;
        if self.window_end_unix_ms <= self.window_start_unix_ms {
            return Err(crate::DataLayerError::InvalidInput(
                "stability_observations.window_end_unix_ms must be greater than window_start_unix_ms"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpsertNifflerStabilityObservationRecord {
    pub id: String,
    pub window_start_unix_ms: u64,
    pub window_end_unix_ms: u64,
    pub status: String,
    pub rollback_drill_status: String,
    pub consistency_checked_count: u64,
    pub consistency_issue_count: u64,
    pub unknown_upstream_count: u64,
    pub legacy_write_call_count: u64,
    pub billing_reservation_exception_count: u64,
    pub referral_exception_count: u64,
    pub blocker_codes: Vec<String>,
    pub summary: Option<serde_json::Value>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl UpsertNifflerStabilityObservationRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("stability_observations.id", &self.id)?;
        validate_stability_status(&self.status)?;
        validate_rollback_drill_status(&self.rollback_drill_status)?;
        if self.window_end_unix_ms <= self.window_start_unix_ms {
            return Err(crate::DataLayerError::InvalidInput(
                "stability_observations.window_end_unix_ms must be greater than window_start_unix_ms"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerErrorReturnSettingListQuery {
    pub scope: Option<NifflerErrorResponseScope>,
    pub upstream_service_id: Option<String>,
    pub include_inactive: bool,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerErrorReturnSettingListPage {
    pub items: Vec<StoredNifflerErrorReturnSetting>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNifflerProductPlanRecord {
    pub id: String,
    pub display_name: String,
    pub is_public: bool,
    pub is_active: bool,
    pub sales_multiplier: f64,
    pub description: Option<String>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl CreateNifflerProductPlanRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        StoredNifflerProductPlan {
            id: self.id.clone(),
            display_name: self.display_name.clone(),
            is_public: self.is_public,
            is_active: self.is_active,
            sales_multiplier: self.sales_multiplier,
            description: self.description.clone(),
            created_at_unix_ms: self.created_at_unix_ms,
            updated_at_unix_ms: self.updated_at_unix_ms,
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpsertNifflerProductPlanModelRecord {
    pub id: String,
    pub product_plan_id: String,
    pub model_name: String,
    pub is_enabled: bool,
    pub sales_multiplier_override: Option<f64>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl UpsertNifflerProductPlanModelRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        StoredNifflerProductPlanModel {
            id: self.id.clone(),
            product_plan_id: self.product_plan_id.clone(),
            model_name: self.model_name.clone(),
            is_enabled: self.is_enabled,
            sales_multiplier_override: self.sales_multiplier_override,
            created_at_unix_ms: self.created_at_unix_ms,
            updated_at_unix_ms: self.updated_at_unix_ms,
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpsertNifflerApiKeyProductPlanBindingRecord {
    pub id: String,
    pub api_key_id: String,
    pub product_plan_id: String,
    pub config: Option<serde_json::Value>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl UpsertNifflerApiKeyProductPlanBindingRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        StoredNifflerApiKeyProductPlanBinding {
            id: self.id.clone(),
            api_key_id: self.api_key_id.clone(),
            product_plan_id: self.product_plan_id.clone(),
            config: self.config.clone(),
            created_at_unix_ms: self.created_at_unix_ms,
            updated_at_unix_ms: self.updated_at_unix_ms,
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpsertNifflerRuntimeRolloutSettingRecord {
    pub id: String,
    pub target_scope: NifflerRuntimeRolloutTargetScope,
    pub target_id: String,
    pub enable_new_routing: bool,
    pub enable_settlement_snapshot: bool,
    pub enable_error_return_rules: bool,
    pub enable_billing_reservation: bool,
    pub enable_referral_ledger: bool,
    pub is_active: bool,
    pub config: Option<serde_json::Value>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl UpsertNifflerRuntimeRolloutSettingRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        StoredNifflerRuntimeRolloutSetting {
            id: self.id.clone(),
            target_scope: self.target_scope,
            target_id: self.target_id.clone(),
            enable_new_routing: self.enable_new_routing,
            enable_settlement_snapshot: self.enable_settlement_snapshot,
            enable_error_return_rules: self.enable_error_return_rules,
            enable_billing_reservation: self.enable_billing_reservation,
            enable_referral_ledger: self.enable_referral_ledger,
            is_active: self.is_active,
            config: self.config.clone(),
            created_at_unix_ms: self.created_at_unix_ms,
            updated_at_unix_ms: self.updated_at_unix_ms,
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNifflerErrorReturnSettingRecord {
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

impl CreateNifflerErrorReturnSettingRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        StoredNifflerErrorReturnSetting {
            id: self.id.clone(),
            scope: self.scope,
            upstream_service_id: self.upstream_service_id.clone(),
            match_status_code: self.match_status_code,
            match_text: self.match_text.clone(),
            handling_step: self.handling_step,
            response_mode: self.response_mode,
            user_message: self.user_message.clone(),
            account_protection_action: self.account_protection_action,
            pause_duration: self.pause_duration,
            is_active: self.is_active,
            created_at_unix_ms: self.created_at_unix_ms,
            updated_at_unix_ms: self.updated_at_unix_ms,
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNifflerAccountRiskEventRecord {
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

impl CreateNifflerAccountRiskEventRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        StoredNifflerAccountRiskEvent {
            id: self.id.clone(),
            upstream_service_id: self.upstream_service_id.clone(),
            upstream_account_id: self.upstream_account_id.clone(),
            request_id: self.request_id.clone(),
            user_id: self.user_id.clone(),
            api_key_id: self.api_key_id.clone(),
            model_name: self.model_name.clone(),
            rule_id: self.rule_id.clone(),
            matched_text: self.matched_text.clone(),
            upstream_status_code: self.upstream_status_code,
            action: self.action,
            created_at_unix_ms: self.created_at_unix_ms,
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNifflerRouteAttemptRecord {
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

impl CreateNifflerRouteAttemptRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        StoredNifflerRouteAttempt {
            id: self.id.clone(),
            request_id: self.request_id.clone(),
            upstream_service_id: self.upstream_service_id.clone(),
            upstream_account_id: self.upstream_account_id.clone(),
            product_plan_id: self.product_plan_id.clone(),
            model_name: self.model_name.clone(),
            attempt_index: self.attempt_index,
            status: self.status.clone(),
            skip_reason: self.skip_reason.clone(),
            upstream_status_code: self.upstream_status_code,
            latency_ms: self.latency_ms,
            created_at_unix_ms: self.created_at_unix_ms,
        }
        .validate()
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerAccountModelCapabilityListQuery {
    pub upstream_service_id: Option<String>,
    pub upstream_account_id: Option<String>,
    pub model_name: Option<String>,
    pub enabled_only: bool,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerAccountModelCapabilityListPage {
    pub items: Vec<StoredNifflerAccountModelCapability>,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NifflerRuntimeAccountModelAccessListQuery {
    pub model_name: String,
    pub now_unix_ms: u64,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerRuntimeAccountModelAccess {
    pub upstream_service_id: String,
    pub upstream_account_id: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerRuntimeAccountModelAccessListPage {
    pub items: Vec<StoredNifflerRuntimeAccountModelAccess>,
    pub total: usize,
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
pub struct StoredNifflerSettlementSnapshot {
    pub id: String,
    pub request_id: String,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub product_plan_id: Option<String>,
    pub upstream_service_id: Option<String>,
    pub upstream_account_id: Option<String>,
    pub requested_model_name: String,
    pub upstream_execution_model_name: Option<String>,
    pub image_tool_model_name: Option<String>,
    pub pricing_snapshot: serde_json::Value,
    pub wallet_charge_usd: f64,
    pub entitlement_charge_usd: f64,
    pub upstream_cost_usd: f64,
    pub gross_margin_usd: f64,
    pub created_at_unix_ms: u64,
    pub finalized_at_unix_ms: Option<u64>,
}

impl StoredNifflerSettlementSnapshot {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("settlement_snapshots.id", &self.id)?;
        validate_required("settlement_snapshots.request_id", &self.request_id)?;
        validate_required(
            "settlement_snapshots.requested_model_name",
            &self.requested_model_name,
        )?;
        validate_non_negative(
            "settlement_snapshots.wallet_charge_usd",
            self.wallet_charge_usd,
        )?;
        validate_non_negative(
            "settlement_snapshots.entitlement_charge_usd",
            self.entitlement_charge_usd,
        )?;
        validate_non_negative(
            "settlement_snapshots.upstream_cost_usd",
            self.upstream_cost_usd,
        )?;
        validate_finite(
            "settlement_snapshots.gross_margin_usd",
            self.gross_margin_usd,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNifflerSettlementSnapshotRecord {
    pub id: String,
    pub request_id: String,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub product_plan_id: Option<String>,
    pub upstream_service_id: Option<String>,
    pub upstream_account_id: Option<String>,
    pub requested_model_name: String,
    pub upstream_execution_model_name: Option<String>,
    pub image_tool_model_name: Option<String>,
    pub pricing_snapshot: serde_json::Value,
    pub wallet_charge_usd: f64,
    pub entitlement_charge_usd: f64,
    pub upstream_cost_usd: f64,
    pub gross_margin_usd: f64,
    pub created_at_unix_ms: u64,
    pub finalized_at_unix_ms: Option<u64>,
}

impl CreateNifflerSettlementSnapshotRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        StoredNifflerSettlementSnapshot {
            id: self.id.clone(),
            request_id: self.request_id.clone(),
            user_id: self.user_id.clone(),
            api_key_id: self.api_key_id.clone(),
            product_plan_id: self.product_plan_id.clone(),
            upstream_service_id: self.upstream_service_id.clone(),
            upstream_account_id: self.upstream_account_id.clone(),
            requested_model_name: self.requested_model_name.clone(),
            upstream_execution_model_name: self.upstream_execution_model_name.clone(),
            image_tool_model_name: self.image_tool_model_name.clone(),
            pricing_snapshot: self.pricing_snapshot.clone(),
            wallet_charge_usd: self.wallet_charge_usd,
            entitlement_charge_usd: self.entitlement_charge_usd,
            upstream_cost_usd: self.upstream_cost_usd,
            gross_margin_usd: self.gross_margin_usd,
            created_at_unix_ms: self.created_at_unix_ms,
            finalized_at_unix_ms: self.finalized_at_unix_ms,
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNifflerBillingReservationRecord {
    pub id: String,
    pub request_id: String,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub product_plan_id: Option<String>,
    pub reserved_total_usd: f64,
    pub wallet_reserved_usd: f64,
    pub entitlement_reserved_usd: f64,
    pub reserved_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub idempotency_key: String,
    pub event_id: String,
    pub event_idempotency_key: String,
    pub actor_id: Option<String>,
}

impl CreateNifflerBillingReservationRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        StoredNifflerBillingReservation {
            id: self.id.clone(),
            request_id: self.request_id.clone(),
            user_id: self.user_id.clone(),
            api_key_id: self.api_key_id.clone(),
            product_plan_id: self.product_plan_id.clone(),
            status: NifflerBillingReservationStatus::Active,
            reserved_total_usd: self.reserved_total_usd,
            wallet_reserved_usd: self.wallet_reserved_usd,
            entitlement_reserved_usd: self.entitlement_reserved_usd,
            reserved_at_unix_ms: self.reserved_at_unix_ms,
            expires_at_unix_ms: self.expires_at_unix_ms,
            finalized_at_unix_ms: None,
            settlement_snapshot_id: None,
            release_reason: None,
            idempotency_key: self.idempotency_key.clone(),
        }
        .validate()?;
        StoredNifflerBillingReservationEvent {
            id: self.event_id.clone(),
            reservation_id: self.id.clone(),
            event_kind: NifflerBillingReservationEventKind::Reserved,
            amount_usd: self.reserved_total_usd,
            reason: None,
            idempotency_key: self.event_idempotency_key.clone(),
            actor_id: self.actor_id.clone(),
            created_at_unix_ms: self.reserved_at_unix_ms,
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FinalizeNifflerBillingReservationRecord {
    pub request_id: String,
    pub status: NifflerBillingReservationStatus,
    pub finalized_at_unix_ms: u64,
    pub settlement_snapshot_id: Option<String>,
    pub release_reason: Option<String>,
    pub event_id: String,
    pub event_idempotency_key: String,
    pub actor_id: Option<String>,
}

impl FinalizeNifflerBillingReservationRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("billing_reservations.request_id", &self.request_id)?;
        validate_required("billing_reservation_events.id", &self.event_id)?;
        validate_required(
            "billing_reservation_events.idempotency_key",
            &self.event_idempotency_key,
        )?;
        if self.status == NifflerBillingReservationStatus::Active {
            return Err(crate::DataLayerError::InvalidInput(
                "billing reservation final status cannot be active".to_string(),
            ));
        }
        if self.status == NifflerBillingReservationStatus::Settled
            && self
                .settlement_snapshot_id
                .as_deref()
                .is_none_or(str::is_empty)
        {
            return Err(crate::DataLayerError::InvalidInput(
                "settled billing reservation must include settlement_snapshot_id".to_string(),
            ));
        }
        if matches!(
            self.status,
            NifflerBillingReservationStatus::Released
                | NifflerBillingReservationStatus::Expired
                | NifflerBillingReservationStatus::ManualReview
        ) && self.release_reason.as_deref().is_none_or(str::is_empty)
        {
            return Err(crate::DataLayerError::InvalidInput(
                "released, expired, or manual review billing reservation must include release_reason"
                    .to_string(),
            ));
        }
        Ok(())
    }

    pub fn event_kind(&self) -> NifflerBillingReservationEventKind {
        match self.status {
            NifflerBillingReservationStatus::Active => NifflerBillingReservationEventKind::Reserved,
            NifflerBillingReservationStatus::Settled => NifflerBillingReservationEventKind::Settled,
            NifflerBillingReservationStatus::Released => {
                NifflerBillingReservationEventKind::Released
            }
            NifflerBillingReservationStatus::Expired => NifflerBillingReservationEventKind::Expired,
            NifflerBillingReservationStatus::ManualReview => {
                NifflerBillingReservationEventKind::ManualReview
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNifflerBillingReservationDryRunRecord {
    pub id: String,
    pub request_id: String,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub product_plan_id: Option<String>,
    pub requested_model_name: String,
    pub estimated_reservation_usd: f64,
    pub legacy_final_charge_usd: f64,
    pub difference_usd: f64,
    pub estimation_source: String,
    pub status: String,
    pub created_at_unix_ms: u64,
    pub finalized_at_unix_ms: Option<u64>,
}

impl CreateNifflerBillingReservationDryRunRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        StoredNifflerBillingReservationDryRun {
            id: self.id.clone(),
            request_id: self.request_id.clone(),
            user_id: self.user_id.clone(),
            api_key_id: self.api_key_id.clone(),
            product_plan_id: self.product_plan_id.clone(),
            requested_model_name: self.requested_model_name.clone(),
            estimated_reservation_usd: self.estimated_reservation_usd,
            legacy_final_charge_usd: self.legacy_final_charge_usd,
            difference_usd: self.difference_usd,
            estimation_source: self.estimation_source.clone(),
            status: self.status.clone(),
            created_at_unix_ms: self.created_at_unix_ms,
            finalized_at_unix_ms: self.finalized_at_unix_ms,
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerBillingReservation {
    pub id: String,
    pub request_id: String,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub product_plan_id: Option<String>,
    pub status: NifflerBillingReservationStatus,
    pub reserved_total_usd: f64,
    pub wallet_reserved_usd: f64,
    pub entitlement_reserved_usd: f64,
    pub reserved_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub finalized_at_unix_ms: Option<u64>,
    pub settlement_snapshot_id: Option<String>,
    pub release_reason: Option<String>,
    pub idempotency_key: String,
}

impl StoredNifflerBillingReservation {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("billing_reservations.id", &self.id)?;
        validate_required("billing_reservations.request_id", &self.request_id)?;
        validate_required(
            "billing_reservations.idempotency_key",
            &self.idempotency_key,
        )?;
        validate_non_negative(
            "billing_reservations.reserved_total_usd",
            self.reserved_total_usd,
        )?;
        validate_non_negative(
            "billing_reservations.wallet_reserved_usd",
            self.wallet_reserved_usd,
        )?;
        validate_non_negative(
            "billing_reservations.entitlement_reserved_usd",
            self.entitlement_reserved_usd,
        )?;
        if self.expires_at_unix_ms <= self.reserved_at_unix_ms {
            return Err(crate::DataLayerError::InvalidInput(
                "billing reservation expires_at_unix_ms must be after reserved_at_unix_ms"
                    .to_string(),
            ));
        }
        if self.status == NifflerBillingReservationStatus::Settled
            && self.settlement_snapshot_id.is_none()
        {
            return Err(crate::DataLayerError::InvalidInput(
                "settled billing reservation must include settlement_snapshot_id".to_string(),
            ));
        }
        if matches!(
            self.status,
            NifflerBillingReservationStatus::Released
                | NifflerBillingReservationStatus::Expired
                | NifflerBillingReservationStatus::ManualReview
        ) && self.release_reason.as_deref().is_none_or(str::is_empty)
        {
            return Err(crate::DataLayerError::InvalidInput(
                "released, expired, or manual review billing reservation must include release_reason"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerBillingReservationDryRun {
    pub id: String,
    pub request_id: String,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub product_plan_id: Option<String>,
    pub requested_model_name: String,
    pub estimated_reservation_usd: f64,
    pub legacy_final_charge_usd: f64,
    pub difference_usd: f64,
    pub estimation_source: String,
    pub status: String,
    pub created_at_unix_ms: u64,
    pub finalized_at_unix_ms: Option<u64>,
}

impl StoredNifflerBillingReservationDryRun {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("billing_reservation_dry_runs.id", &self.id)?;
        validate_required("billing_reservation_dry_runs.request_id", &self.request_id)?;
        validate_required(
            "billing_reservation_dry_runs.requested_model_name",
            &self.requested_model_name,
        )?;
        validate_required(
            "billing_reservation_dry_runs.estimation_source",
            &self.estimation_source,
        )?;
        validate_required("billing_reservation_dry_runs.status", &self.status)?;
        validate_non_negative(
            "billing_reservation_dry_runs.estimated_reservation_usd",
            self.estimated_reservation_usd,
        )?;
        validate_non_negative(
            "billing_reservation_dry_runs.legacy_final_charge_usd",
            self.legacy_final_charge_usd,
        )?;
        validate_finite(
            "billing_reservation_dry_runs.difference_usd",
            self.difference_usd,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerBillingReservationEvent {
    pub id: String,
    pub reservation_id: String,
    pub event_kind: NifflerBillingReservationEventKind,
    pub amount_usd: f64,
    pub reason: Option<String>,
    pub idempotency_key: String,
    pub actor_id: Option<String>,
    pub created_at_unix_ms: u64,
}

impl StoredNifflerBillingReservationEvent {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("billing_reservation_events.id", &self.id)?;
        validate_required(
            "billing_reservation_events.reservation_id",
            &self.reservation_id,
        )?;
        validate_required(
            "billing_reservation_events.idempotency_key",
            &self.idempotency_key,
        )?;
        validate_non_negative("billing_reservation_events.amount_usd", self.amount_usd)?;
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
pub struct StoredNifflerReferralRewardRule {
    pub id: String,
    pub display_name: String,
    pub status: NifflerReferralRewardRuleStatus,
    pub reward_kind: NifflerReferralRewardKind,
    pub reward_value: f64,
    pub applies_to_order_kind: Option<String>,
    pub max_reward_usd: Option<f64>,
    pub effective_from_unix_ms: u64,
    pub effective_until_unix_ms: Option<u64>,
    pub config: Option<serde_json::Value>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerReferralRewardRule {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("referral_reward_rules.id", &self.id)?;
        validate_required("referral_reward_rules.display_name", &self.display_name)?;
        validate_non_negative("referral_reward_rules.reward_value", self.reward_value)?;
        validate_optional_non_negative(
            "referral_reward_rules.max_reward_usd",
            self.max_reward_usd,
        )?;
        if matches!(self.reward_kind, NifflerReferralRewardKind::Percentage)
            && self.reward_value > 1.0
        {
            return Err(crate::DataLayerError::InvalidInput(
                "percentage referral reward_value must be between 0 and 1".to_string(),
            ));
        }
        if self
            .effective_until_unix_ms
            .is_some_and(|until| until <= self.effective_from_unix_ms)
        {
            return Err(crate::DataLayerError::InvalidInput(
                "referral reward effective_until_unix_ms must be after effective_from_unix_ms"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerReferralRewardLedger {
    pub id: String,
    pub order_id: String,
    pub idempotency_key: String,
    pub inviter_user_id: String,
    pub invitee_user_id: String,
    pub rule_id: Option<String>,
    pub reward_amount_usd: f64,
    pub rule_snapshot: serde_json::Value,
    pub status: NifflerReferralRewardLedgerStatus,
    pub failure_reason: Option<String>,
    pub retry_count: u32,
    pub paid_at_unix_ms: Option<u64>,
    pub cancelled_at_unix_ms: Option<u64>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl StoredNifflerReferralRewardLedger {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("referral_reward_ledger.id", &self.id)?;
        validate_required("referral_reward_ledger.order_id", &self.order_id)?;
        validate_required(
            "referral_reward_ledger.idempotency_key",
            &self.idempotency_key,
        )?;
        validate_required(
            "referral_reward_ledger.inviter_user_id",
            &self.inviter_user_id,
        )?;
        validate_required(
            "referral_reward_ledger.invitee_user_id",
            &self.invitee_user_id,
        )?;
        validate_non_negative(
            "referral_reward_ledger.reward_amount_usd",
            self.reward_amount_usd,
        )?;
        if self.status == NifflerReferralRewardLedgerStatus::Failed
            && self.failure_reason.as_deref().is_none_or(str::is_empty)
        {
            return Err(crate::DataLayerError::InvalidInput(
                "failed referral reward must include failure_reason".to_string(),
            ));
        }
        if self.status == NifflerReferralRewardLedgerStatus::Paid && self.paid_at_unix_ms.is_none()
        {
            return Err(crate::DataLayerError::InvalidInput(
                "paid referral reward must include paid_at_unix_ms".to_string(),
            ));
        }
        if self.status == NifflerReferralRewardLedgerStatus::Cancelled
            && self.cancelled_at_unix_ms.is_none()
        {
            return Err(crate::DataLayerError::InvalidInput(
                "cancelled referral reward must include cancelled_at_unix_ms".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNifflerReferralRewardLedgerRecord {
    pub id: String,
    pub order_id: String,
    pub idempotency_key: String,
    pub inviter_user_id: String,
    pub invitee_user_id: String,
    pub rule_id: Option<String>,
    pub reward_amount_usd: f64,
    pub rule_snapshot: serde_json::Value,
    pub status: NifflerReferralRewardLedgerStatus,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

impl CreateNifflerReferralRewardLedgerRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        StoredNifflerReferralRewardLedger {
            id: self.id.clone(),
            order_id: self.order_id.clone(),
            idempotency_key: self.idempotency_key.clone(),
            inviter_user_id: self.inviter_user_id.clone(),
            invitee_user_id: self.invitee_user_id.clone(),
            rule_id: self.rule_id.clone(),
            reward_amount_usd: self.reward_amount_usd,
            rule_snapshot: self.rule_snapshot.clone(),
            status: self.status,
            failure_reason: None,
            retry_count: 0,
            paid_at_unix_ms: None,
            cancelled_at_unix_ms: None,
            created_at_unix_ms: self.created_at_unix_ms,
            updated_at_unix_ms: self.updated_at_unix_ms,
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StoredNifflerReferralRewardEvent {
    pub id: String,
    pub reward_ledger_id: String,
    pub event_kind: NifflerReferralRewardEventKind,
    pub reason: Option<String>,
    pub actor_id: Option<String>,
    pub event_snapshot: Option<serde_json::Value>,
    pub created_at_unix_ms: u64,
}

impl StoredNifflerReferralRewardEvent {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_required("referral_reward_events.id", &self.id)?;
        validate_required(
            "referral_reward_events.reward_ledger_id",
            &self.reward_ledger_id,
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
    pub product_plans_total: u64,
    pub product_plans_public: u64,
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
    pub product_plan_id: String,
    pub product_plan_name: String,
    pub provider_id: String,
    pub provider_name: String,
    pub source_field: String,
    pub source_field_label: String,
    pub reason: String,
    pub impact: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerKeyScopeResidue {
    pub subject_kind: String,
    pub key_id: String,
    pub key_name: Option<String>,
    pub owner_label: Option<String>,
    pub display_name: String,
    pub provider_id: Option<String>,
    pub provider_name: Option<String>,
    pub account_label: Option<String>,
    pub residue_fields: Vec<String>,
    pub field_labels: Vec<String>,
    pub reason: String,
    pub impact: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerGroupPolicyGap {
    pub product_plan_id: String,
    pub product_plan_name: String,
    pub gap_kind: String,
    pub gap_label: String,
    pub message: String,
    pub impact: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerPriceGap {
    pub scope: String,
    pub scope_label: String,
    pub provider_id: Option<String>,
    pub provider_name: Option<String>,
    pub model_id: Option<String>,
    pub model_name: String,
    pub missing_fields: Vec<String>,
    pub reason: String,
    pub impact: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NifflerUsageAnomaly {
    pub usage_id: String,
    pub request_id: String,
    pub created_at_unix_secs: u64,
    pub provider_name: String,
    pub provider_id: Option<String>,
    pub provider_api_key_id: Option<String>,
    pub provider_display_name: String,
    pub provider_api_key_name: Option<String>,
    pub provider_account_label: Option<String>,
    pub model: String,
    pub status: String,
    pub billing_status: String,
    pub status_code: Option<u16>,
    pub error_category: Option<String>,
    pub anomaly_kind: String,
    pub anomaly_label: String,
    pub diagnosis: String,
    pub impact: String,
    pub recommended_action: String,
    pub total_cost_usd: f64,
    pub actual_total_cost_usd: f64,
    pub package_debit_usd: Option<f64>,
    pub wallet_debit_usd: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerRouteSkipReasonSummary {
    pub reason: String,
    pub label: String,
    pub category: String,
    pub count: u64,
    pub impact: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NifflerRouteSkipSample {
    pub request_id: String,
    pub created_at_unix_secs: u64,
    pub provider_id: Option<String>,
    pub provider_name: Option<String>,
    pub key_id: Option<String>,
    pub key_name: Option<String>,
    pub account_label: Option<String>,
    pub reason: String,
    pub label: String,
    pub impact: String,
    pub recommended_action: String,
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
    pub route_skip_samples: Vec<NifflerRouteSkipSample>,
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
    validate_finite(field, value)?;
    if value < 0.0 {
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

fn validate_finite(field: &str, value: f64) -> Result<(), crate::DataLayerError> {
    if !value.is_finite() {
        return Err(crate::DataLayerError::InvalidInput(format!(
            "{field} must be a finite number"
        )));
    }
    Ok(())
}

fn validate_stability_status(value: &str) -> Result<(), crate::DataLayerError> {
    match value {
        "pass" | "pending" | "reset_required" => Ok(()),
        _ => Err(crate::DataLayerError::InvalidInput(format!(
            "stability_observations.status is invalid: {value}"
        ))),
    }
}

fn validate_rollback_drill_status(value: &str) -> Result<(), crate::DataLayerError> {
    match value {
        "passed" | "failed" | "not_recorded" => Ok(()),
        _ => Err(crate::DataLayerError::InvalidInput(format!(
            "stability_observations.rollback_drill_status is invalid: {value}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        default_platform_error_message, CreateNifflerReferralRewardLedgerRecord,
        NifflerAccountProtectionAction, NifflerAccountStatus, NifflerBillingReservationStatus,
        NifflerPauseDuration, NifflerPriceSourcePreference, NifflerProtocolKind,
        NifflerReferralRewardKind, NifflerReferralRewardLedgerStatus,
        NifflerReferralRewardRuleStatus, NifflerServiceCapabilityKind,
        NifflerUpstreamErrorHandlingStep, StoredNifflerApiKeyPause,
        StoredNifflerBillingReservation, StoredNifflerBillingReservationEvent,
        StoredNifflerErrorReturnSetting, StoredNifflerProductPlan, StoredNifflerProductPlanModel,
        StoredNifflerReferralRewardLedger, StoredNifflerReferralRewardRule,
        StoredNifflerUpstreamModelPrice, StoredNifflerUpstreamServiceCapability,
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

    #[test]
    fn openai_responses_image_tool_is_protocol_scoped() {
        let enabled_capability = StoredNifflerUpstreamServiceCapability {
            id: "capability-1".to_string(),
            upstream_service_id: "service-1".to_string(),
            protocol_kind: NifflerProtocolKind::Gemini,
            capability_kind: NifflerServiceCapabilityKind::OpenaiResponsesImageTool,
            is_enabled: true,
            config: None,
            created_at_unix_ms: 1,
            updated_at_unix_ms: 1,
        };
        assert!(enabled_capability.validate().is_err());

        let disabled_capability = StoredNifflerUpstreamServiceCapability {
            is_enabled: false,
            ..enabled_capability
        };
        assert!(disabled_capability.validate().is_ok());
    }

    #[test]
    fn settled_reservation_requires_snapshot() {
        let reservation = StoredNifflerBillingReservation {
            id: "reservation-1".to_string(),
            request_id: "request-1".to_string(),
            user_id: Some("user-1".to_string()),
            api_key_id: Some("key-1".to_string()),
            product_plan_id: Some("plan-1".to_string()),
            status: NifflerBillingReservationStatus::Settled,
            reserved_total_usd: 1.0,
            wallet_reserved_usd: 1.0,
            entitlement_reserved_usd: 0.0,
            reserved_at_unix_ms: 1,
            expires_at_unix_ms: 2,
            finalized_at_unix_ms: Some(2),
            settlement_snapshot_id: None,
            release_reason: None,
            idempotency_key: "reservation-request-1".to_string(),
        };
        assert!(reservation.validate().is_err());
    }

    #[test]
    fn reservation_event_requires_idempotency_key() {
        let event = StoredNifflerBillingReservationEvent {
            id: "reservation-event-1".to_string(),
            reservation_id: "reservation-1".to_string(),
            event_kind: super::NifflerBillingReservationEventKind::Released,
            amount_usd: 1.0,
            reason: Some("request_cancelled".to_string()),
            idempotency_key: "   ".to_string(),
            actor_id: None,
            created_at_unix_ms: 1,
        };
        assert!(event.validate().is_err());
    }

    #[test]
    fn referral_percentage_is_fraction() {
        let rule = StoredNifflerReferralRewardRule {
            id: "rule-1".to_string(),
            display_name: "邀请返利".to_string(),
            status: NifflerReferralRewardRuleStatus::Active,
            reward_kind: NifflerReferralRewardKind::Percentage,
            reward_value: 30.0,
            applies_to_order_kind: Some("wallet_recharge".to_string()),
            max_reward_usd: None,
            effective_from_unix_ms: 1,
            effective_until_unix_ms: None,
            config: None,
            created_at_unix_ms: 1,
            updated_at_unix_ms: 1,
        };
        assert!(rule.validate().is_err());
    }

    #[test]
    fn failed_referral_reward_requires_reason() {
        let ledger = StoredNifflerReferralRewardLedger {
            id: "reward-1".to_string(),
            order_id: "order-1".to_string(),
            idempotency_key: "order-1".to_string(),
            inviter_user_id: "user-a".to_string(),
            invitee_user_id: "user-b".to_string(),
            rule_id: None,
            reward_amount_usd: 1.0,
            rule_snapshot: serde_json::json!({ "reward_kind": "fixed_amount" }),
            status: NifflerReferralRewardLedgerStatus::Failed,
            failure_reason: None,
            retry_count: 0,
            paid_at_unix_ms: None,
            cancelled_at_unix_ms: None,
            created_at_unix_ms: 1,
            updated_at_unix_ms: 1,
        };
        assert!(ledger.validate().is_err());
    }

    #[test]
    fn pending_referral_reward_create_record_is_valid() {
        let record = CreateNifflerReferralRewardLedgerRecord {
            id: "reward-1".to_string(),
            order_id: "order-1".to_string(),
            idempotency_key: "niffler-referral-ledger:order:order-1".to_string(),
            inviter_user_id: "user-a".to_string(),
            invitee_user_id: "user-b".to_string(),
            rule_id: None,
            reward_amount_usd: 1.0,
            rule_snapshot: serde_json::json!({ "source": "legacy_referral_rewards" }),
            status: NifflerReferralRewardLedgerStatus::Pending,
            created_at_unix_ms: 1,
            updated_at_unix_ms: 1,
        };
        assert!(record.validate().is_ok());

        let paid_without_paid_time = CreateNifflerReferralRewardLedgerRecord {
            status: NifflerReferralRewardLedgerStatus::Paid,
            ..record
        };
        assert!(paid_without_paid_time.validate().is_err());
    }
}
