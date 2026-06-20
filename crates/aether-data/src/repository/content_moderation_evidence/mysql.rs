use async_trait::async_trait;
use serde_json::Value;
use sqlx::{mysql::MySqlRow, Row};

use super::{
    ContentModerationEvidenceReadRepository, ContentModerationEvidenceWriteRepository,
    InsertContentModerationEvidenceRecord, StoredContentModerationEvidence,
};
use crate::driver::mysql::MysqlPool;
use crate::error::SqlResultExt;
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

const INSERT_EVIDENCE_SQL: &str = r#"
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
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
ON DUPLICATE KEY UPDATE
  request_id = VALUES(request_id),
  user_id = VALUES(user_id),
  api_key_id = VALUES(api_key_id),
  provider_id = VALUES(provider_id),
  upstream_service_id = VALUES(upstream_service_id),
  upstream_account_id = VALUES(upstream_account_id),
  moderation_model = VALUES(moderation_model),
  input_sha256 = VALUES(input_sha256),
  input_text = VALUES(input_text),
  categories = VALUES(categories),
  category_scores = VALUES(category_scores),
  flagged = VALUES(flagged),
  created_at_unix_secs = VALUES(created_at_unix_secs),
  expires_at_unix_secs = VALUES(expires_at_unix_secs),
  redacted_at_unix_secs = NULL
"#;

#[derive(Debug, Clone)]
pub struct MysqlContentModerationEvidenceRepository {
    pool: MysqlPool,
}

impl MysqlContentModerationEvidenceRepository {
    pub fn new(pool: MysqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContentModerationEvidenceReadRepository for MysqlContentModerationEvidenceRepository {
    async fn find_by_id(
        &self,
        evidence_id: &str,
    ) -> Result<Option<StoredContentModerationEvidence>, DataLayerError> {
        let row = sqlx::query(&format!("{EVIDENCE_COLUMNS} WHERE id = ? LIMIT 1"))
            .bind(evidence_id)
            .fetch_optional(&self.pool)
            .await
            .map_sql_err()?;
        row.as_ref().map(map_row).transpose()
    }
}

#[async_trait]
impl ContentModerationEvidenceWriteRepository for MysqlContentModerationEvidenceRepository {
    async fn insert(
        &self,
        record: InsertContentModerationEvidenceRecord,
    ) -> Result<StoredContentModerationEvidence, DataLayerError> {
        record.validate()?;
        sqlx::query(INSERT_EVIDENCE_SQL)
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
            .bind(json_to_string(
                &record.categories,
                "content_moderation_evidence.categories",
            )?)
            .bind(json_to_string(
                &record.category_scores,
                "content_moderation_evidence.category_scores",
            )?)
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
            .map_sql_err()?;

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
    redacted_at_unix_secs = ?
WHERE input_text IS NOT NULL
  AND expires_at_unix_secs <= ?
ORDER BY expires_at_unix_secs ASC, id ASC
LIMIT ?
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
        .map_sql_err()?
        .rows_affected();
        Ok(usize::try_from(rows_affected).unwrap_or_default())
    }
}

fn map_row(row: &MySqlRow) -> Result<StoredContentModerationEvidence, DataLayerError> {
    let created_at: i64 = row.try_get("created_at_unix_secs").map_sql_err()?;
    let expires_at: i64 = row.try_get("expires_at_unix_secs").map_sql_err()?;
    let redacted_at: Option<i64> = row.try_get("redacted_at_unix_secs").map_sql_err()?;
    Ok(StoredContentModerationEvidence {
        id: row.try_get("id").map_sql_err()?,
        request_id: row.try_get("request_id").map_sql_err()?,
        user_id: row.try_get("user_id").map_sql_err()?,
        api_key_id: row.try_get("api_key_id").map_sql_err()?,
        provider_id: row.try_get("provider_id").map_sql_err()?,
        upstream_service_id: row.try_get("upstream_service_id").map_sql_err()?,
        upstream_account_id: row.try_get("upstream_account_id").map_sql_err()?,
        moderation_model: row.try_get("moderation_model").map_sql_err()?,
        input_sha256: row.try_get("input_sha256").map_sql_err()?,
        input_text: row.try_get("input_text").map_sql_err()?,
        categories: json_from_string(
            row.try_get("categories").map_sql_err()?,
            "content_moderation_evidence.categories",
        )?,
        category_scores: json_from_string(
            row.try_get("category_scores").map_sql_err()?,
            "content_moderation_evidence.category_scores",
        )?,
        flagged: row.try_get("flagged").map_sql_err()?,
        created_at_unix_secs: u64::try_from(created_at).unwrap_or_default(),
        expires_at_unix_secs: u64::try_from(expires_at).unwrap_or_default(),
        redacted_at_unix_secs: redacted_at.and_then(|value| u64::try_from(value).ok()),
    })
}

fn json_to_string(value: &Value, field_name: &str) -> Result<String, DataLayerError> {
    serde_json::to_string(value)
        .map_err(|err| DataLayerError::UnexpectedValue(format!("{field_name} invalid json: {err}")))
}

fn json_from_string(value: String, field_name: &str) -> Result<Value, DataLayerError> {
    serde_json::from_str(&value)
        .map_err(|err| DataLayerError::UnexpectedValue(format!("{field_name} invalid json: {err}")))
}

fn u64_to_i64(value: u64, label: &str) -> Result<i64, DataLayerError> {
    i64::try_from(value)
        .map_err(|_| DataLayerError::UnexpectedValue(format!("{label} overflow: {value}")))
}

fn i64_from_usize(value: usize, label: &str) -> Result<i64, DataLayerError> {
    i64::try_from(value)
        .map_err(|_| DataLayerError::UnexpectedValue(format!("{label} overflow: {value}")))
}

#[cfg(test)]
mod tests {
    use super::INSERT_EVIDENCE_SQL;

    #[test]
    fn insert_sql_binds_json_strings_without_mysql_only_casts() {
        assert!(INSERT_EVIDENCE_SQL.contains("?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?"));
        assert!(!INSERT_EVIDENCE_SQL.contains("CAST(? AS JSON)"));
    }
}
