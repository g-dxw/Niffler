use async_trait::async_trait;
use serde_json::Value;
use sqlx::{postgres::PgRow, PgPool, Row};

use super::{
    ContentModerationEvidenceReadRepository, ContentModerationEvidenceWriteRepository,
    InsertContentModerationEvidenceRecord, StoredContentModerationEvidence,
};
use crate::error::SqlxResultExt;
use crate::DataLayerError;

const EVIDENCE_COLUMNS: &str = r#"
SELECT
  id,
  request_id,
  user_id,
  api_key_id,
  provider_id,
  upstream_service_id,
  upstream_account_id,
  moderation_model,
  input_sha256,
  input_text,
  categories,
  category_scores,
  flagged,
  created_at_unix_secs,
  expires_at_unix_secs,
  redacted_at_unix_secs
FROM content_moderation_evidence
"#;

#[derive(Debug, Clone)]
pub struct SqlxContentModerationEvidenceRepository {
    pool: PgPool,
}

impl SqlxContentModerationEvidenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContentModerationEvidenceReadRepository for SqlxContentModerationEvidenceRepository {
    async fn find_by_id(
        &self,
        evidence_id: &str,
    ) -> Result<Option<StoredContentModerationEvidence>, DataLayerError> {
        let row = sqlx::query(&format!("{EVIDENCE_COLUMNS} WHERE id = $1 LIMIT 1"))
            .bind(evidence_id)
            .fetch_optional(&self.pool)
            .await
            .map_postgres_err()?;
        row.as_ref().map(map_row).transpose()
    }
}

#[async_trait]
impl ContentModerationEvidenceWriteRepository for SqlxContentModerationEvidenceRepository {
    async fn insert(
        &self,
        record: InsertContentModerationEvidenceRecord,
    ) -> Result<StoredContentModerationEvidence, DataLayerError> {
        record.validate()?;
        sqlx::query(
            r#"
INSERT INTO content_moderation_evidence (
  id,
  request_id,
  user_id,
  api_key_id,
  provider_id,
  upstream_service_id,
  upstream_account_id,
  moderation_model,
  input_sha256,
  input_text,
  categories,
  category_scores,
  flagged,
  created_at_unix_secs,
  expires_at_unix_secs,
  redacted_at_unix_secs
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
  $11::jsonb, $12::jsonb, $13, $14, $15, NULL
)
ON CONFLICT (id) DO UPDATE SET
  request_id = EXCLUDED.request_id,
  user_id = EXCLUDED.user_id,
  api_key_id = EXCLUDED.api_key_id,
  provider_id = EXCLUDED.provider_id,
  upstream_service_id = EXCLUDED.upstream_service_id,
  upstream_account_id = EXCLUDED.upstream_account_id,
  moderation_model = EXCLUDED.moderation_model,
  input_sha256 = EXCLUDED.input_sha256,
  input_text = EXCLUDED.input_text,
  categories = EXCLUDED.categories,
  category_scores = EXCLUDED.category_scores,
  flagged = EXCLUDED.flagged,
  created_at_unix_secs = EXCLUDED.created_at_unix_secs,
  expires_at_unix_secs = EXCLUDED.expires_at_unix_secs,
  redacted_at_unix_secs = NULL
"#,
        )
        .bind(&record.id)
        .bind(&record.request_id)
        .bind(&record.user_id)
        .bind(&record.api_key_id)
        .bind(&record.provider_id)
        .bind(&record.upstream_service_id)
        .bind(&record.upstream_account_id)
        .bind(&record.moderation_model)
        .bind(&record.input_sha256)
        .bind(&record.input_text)
        .bind(record.categories.clone())
        .bind(record.category_scores.clone())
        .bind(record.flagged)
        .bind(u64_to_i64(
            record.created_at_unix_secs,
            "content_moderation_evidence.created_at_unix_secs",
        )?)
        .bind(u64_to_i64(
            record.expires_at_unix_secs,
            "content_moderation_evidence.expires_at_unix_secs",
        )?)
        .execute(&self.pool)
        .await
        .map_postgres_err()?;

        self.find_by_id(&record.id).await?.ok_or_else(|| {
            DataLayerError::UnexpectedValue(
                "content moderation evidence missing after insert".to_string(),
            )
        })
    }

    async fn redact_expired_input_text(
        &self,
        now_unix_secs: u64,
        limit: usize,
    ) -> Result<usize, DataLayerError> {
        if limit == 0 {
            return Ok(0);
        }
        let rows_affected = sqlx::query(
            r#"
UPDATE content_moderation_evidence
SET input_text = NULL,
    redacted_at_unix_secs = $1
WHERE id IN (
  SELECT id
  FROM content_moderation_evidence
  WHERE input_text IS NOT NULL
    AND expires_at_unix_secs <= $2
  ORDER BY expires_at_unix_secs ASC, id ASC
  LIMIT $3
)
"#,
        )
        .bind(u64_to_i64(
            now_unix_secs,
            "content_moderation_evidence.redacted_at_unix_secs",
        )?)
        .bind(u64_to_i64(
            now_unix_secs,
            "content_moderation_evidence.expires_at_unix_secs",
        )?)
        .bind(i64_from_usize(
            limit,
            "content moderation evidence cleanup limit",
        )?)
        .execute(&self.pool)
        .await
        .map_postgres_err()?
        .rows_affected();
        Ok(usize::try_from(rows_affected).unwrap_or_default())
    }
}

fn map_row(row: &PgRow) -> Result<StoredContentModerationEvidence, DataLayerError> {
    let created_at: i64 = row.try_get("created_at_unix_secs").map_postgres_err()?;
    let expires_at: i64 = row.try_get("expires_at_unix_secs").map_postgres_err()?;
    let redacted_at: Option<i64> = row.try_get("redacted_at_unix_secs").map_postgres_err()?;
    Ok(StoredContentModerationEvidence {
        id: row.try_get("id").map_postgres_err()?,
        request_id: row.try_get("request_id").map_postgres_err()?,
        user_id: row.try_get("user_id").map_postgres_err()?,
        api_key_id: row.try_get("api_key_id").map_postgres_err()?,
        provider_id: row.try_get("provider_id").map_postgres_err()?,
        upstream_service_id: row.try_get("upstream_service_id").map_postgres_err()?,
        upstream_account_id: row.try_get("upstream_account_id").map_postgres_err()?,
        moderation_model: row.try_get("moderation_model").map_postgres_err()?,
        input_sha256: row.try_get("input_sha256").map_postgres_err()?,
        input_text: row.try_get("input_text").map_postgres_err()?,
        categories: row.try_get::<Value, _>("categories").map_postgres_err()?,
        category_scores: row
            .try_get::<Value, _>("category_scores")
            .map_postgres_err()?,
        flagged: row.try_get("flagged").map_postgres_err()?,
        created_at_unix_secs: u64::try_from(created_at).unwrap_or_default(),
        expires_at_unix_secs: u64::try_from(expires_at).unwrap_or_default(),
        redacted_at_unix_secs: redacted_at.and_then(|value| u64::try_from(value).ok()),
    })
}

fn u64_to_i64(value: u64, label: &str) -> Result<i64, DataLayerError> {
    i64::try_from(value)
        .map_err(|_| DataLayerError::UnexpectedValue(format!("{label} overflow: {value}")))
}

fn i64_from_usize(value: usize, label: &str) -> Result<i64, DataLayerError> {
    i64::try_from(value)
        .map_err(|_| DataLayerError::UnexpectedValue(format!("{label} overflow: {value}")))
}
