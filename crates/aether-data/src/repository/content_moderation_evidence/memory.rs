use std::collections::BTreeMap;
use std::sync::RwLock;

use async_trait::async_trait;

use super::{
    ContentModerationEvidenceReadRepository, ContentModerationEvidenceWriteRepository,
    InsertContentModerationEvidenceRecord, StoredContentModerationEvidence,
};
use crate::DataLayerError;

#[derive(Debug, Default)]
pub struct InMemoryContentModerationEvidenceRepository {
    records: RwLock<BTreeMap<String, StoredContentModerationEvidence>>,
}

impl InMemoryContentModerationEvidenceRepository {
    pub fn seed<I>(records: I) -> Self
    where
        I: IntoIterator<Item = StoredContentModerationEvidence>,
    {
        Self {
            records: RwLock::new(
                records
                    .into_iter()
                    .map(|record| (record.id.clone(), record))
                    .collect(),
            ),
        }
    }
}

#[async_trait]
impl ContentModerationEvidenceReadRepository for InMemoryContentModerationEvidenceRepository {
    async fn find_by_id(
        &self,
        evidence_id: &str,
    ) -> Result<Option<StoredContentModerationEvidence>, DataLayerError> {
        Ok(self
            .records
            .read()
            .expect("content moderation evidence repository lock")
            .get(evidence_id)
            .cloned())
    }
}

#[async_trait]
impl ContentModerationEvidenceWriteRepository for InMemoryContentModerationEvidenceRepository {
    async fn insert(
        &self,
        record: InsertContentModerationEvidenceRecord,
    ) -> Result<StoredContentModerationEvidence, DataLayerError> {
        record.validate()?;
        let stored = record.into_stored();
        self.records
            .write()
            .expect("content moderation evidence repository lock")
            .insert(stored.id.clone(), stored.clone());
        Ok(stored)
    }

    async fn redact_expired_input_text(
        &self,
        now_unix_secs: u64,
        limit: usize,
    ) -> Result<usize, DataLayerError> {
        if limit == 0 {
            return Ok(0);
        }
        let mut records = self
            .records
            .write()
            .expect("content moderation evidence repository lock");
        let mut ids = records
            .values()
            .filter(|record| {
                record.input_text.is_some() && record.expires_at_unix_secs <= now_unix_secs
            })
            .map(|record| (record.expires_at_unix_secs, record.id.clone()))
            .collect::<Vec<_>>();
        ids.sort();
        let ids = ids
            .into_iter()
            .take(limit)
            .map(|(_, id)| id)
            .collect::<Vec<_>>();
        let mut redacted = 0usize;
        for id in ids {
            if let Some(record) = records.get_mut(&id) {
                if record.input_text.take().is_some() {
                    record.redacted_at_unix_secs = Some(now_unix_secs);
                    redacted += 1;
                }
            }
        }
        Ok(redacted)
    }
}
