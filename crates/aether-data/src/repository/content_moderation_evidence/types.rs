use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct StoredContentModerationEvidence {
    pub id: String,
    pub request_id: String,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub provider_id: Option<String>,
    pub upstream_service_id: Option<String>,
    pub upstream_account_id: Option<String>,
    pub moderation_model: String,
    pub input_sha256: String,
    pub input_text: Option<String>,
    pub categories: Value,
    pub category_scores: Value,
    pub flagged: bool,
    pub created_at_unix_secs: u64,
    pub expires_at_unix_secs: u64,
    pub redacted_at_unix_secs: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertContentModerationEvidenceRecord {
    pub id: String,
    pub request_id: String,
    pub user_id: Option<String>,
    pub api_key_id: Option<String>,
    pub provider_id: Option<String>,
    pub upstream_service_id: Option<String>,
    pub upstream_account_id: Option<String>,
    pub moderation_model: String,
    pub input_sha256: String,
    pub input_text: Option<String>,
    pub categories: Value,
    pub category_scores: Value,
    pub flagged: bool,
    pub created_at_unix_secs: u64,
    pub expires_at_unix_secs: u64,
}

impl InsertContentModerationEvidenceRecord {
    pub fn validate(&self) -> Result<(), crate::DataLayerError> {
        validate_non_empty(&self.id, "content_moderation_evidence.id")?;
        validate_non_empty(&self.request_id, "content_moderation_evidence.request_id")?;
        validate_non_empty(
            &self.moderation_model,
            "content_moderation_evidence.moderation_model",
        )?;
        validate_non_empty(
            &self.input_sha256,
            "content_moderation_evidence.input_sha256",
        )?;
        if self.created_at_unix_secs == 0 {
            return Err(crate::DataLayerError::InvalidInput(
                "content_moderation_evidence.created_at is empty".to_string(),
            ));
        }
        if self.expires_at_unix_secs == 0 {
            return Err(crate::DataLayerError::InvalidInput(
                "content_moderation_evidence.expires_at is empty".to_string(),
            ));
        }
        Ok(())
    }

    pub fn into_stored(self) -> StoredContentModerationEvidence {
        StoredContentModerationEvidence {
            id: self.id,
            request_id: self.request_id,
            user_id: self.user_id,
            api_key_id: self.api_key_id,
            provider_id: self.provider_id,
            upstream_service_id: self.upstream_service_id,
            upstream_account_id: self.upstream_account_id,
            moderation_model: self.moderation_model,
            input_sha256: self.input_sha256,
            input_text: self.input_text,
            categories: self.categories,
            category_scores: self.category_scores,
            flagged: self.flagged,
            created_at_unix_secs: self.created_at_unix_secs,
            expires_at_unix_secs: self.expires_at_unix_secs,
            redacted_at_unix_secs: None,
        }
    }
}

#[async_trait]
pub trait ContentModerationEvidenceReadRepository: Send + Sync {
    async fn find_by_id(
        &self,
        evidence_id: &str,
    ) -> Result<Option<StoredContentModerationEvidence>, crate::DataLayerError>;
}

#[async_trait]
pub trait ContentModerationEvidenceWriteRepository: Send + Sync {
    async fn insert(
        &self,
        record: InsertContentModerationEvidenceRecord,
    ) -> Result<StoredContentModerationEvidence, crate::DataLayerError>;

    async fn redact_expired_input_text(
        &self,
        now_unix_secs: u64,
        limit: usize,
    ) -> Result<usize, crate::DataLayerError>;
}

pub trait ContentModerationEvidenceRepository:
    ContentModerationEvidenceReadRepository + ContentModerationEvidenceWriteRepository
{
}

impl<T> ContentModerationEvidenceRepository for T where
    T: ContentModerationEvidenceReadRepository + ContentModerationEvidenceWriteRepository
{
}

fn validate_non_empty(value: &str, field_name: &str) -> Result<(), crate::DataLayerError> {
    if value.trim().is_empty() {
        return Err(crate::DataLayerError::InvalidInput(format!(
            "{field_name} is empty"
        )));
    }
    Ok(())
}
