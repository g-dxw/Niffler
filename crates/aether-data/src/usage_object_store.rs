use std::fmt;
use std::sync::Arc;

use bytes::Bytes;
use object_store::aws::AmazonS3Builder;
use object_store::path::Path;
use object_store::{MultipartUpload, ObjectStore, PutMode, PutOptions};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use crate::DataLayerError;

const DEFAULT_USAGE_OBJECT_PREFIX: &str = "usage";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageObjectStoreConfig {
    pub url: String,
    #[serde(default = "default_usage_object_prefix")]
    pub prefix: String,
}

fn default_usage_object_prefix() -> String {
    DEFAULT_USAGE_OBJECT_PREFIX.to_string()
}

impl UsageObjectStoreConfig {
    pub fn new(url: impl Into<String>, prefix: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            prefix: prefix.into(),
        }
    }

    pub fn validate(&self) -> Result<(), DataLayerError> {
        let url = self.url.trim();
        if url.is_empty() {
            return Err(DataLayerError::InvalidInput(
                "usage object store URL cannot be empty".to_string(),
            ));
        }
        let parsed = Url::parse(url).map_err(|err| {
            DataLayerError::InvalidInput(format!("invalid usage object store URL: {err}"))
        })?;
        match parsed.scheme() {
            "file" | "s3" | "s3a" | "https" => {}
            scheme => {
                return Err(DataLayerError::InvalidInput(format!(
                    "unsupported usage object store URL scheme: {scheme}"
                )))
            }
        }
        Path::parse(self.prefix.trim().trim_matches('/')).map_err(|err| {
            DataLayerError::InvalidInput(format!("invalid usage object store prefix: {err}"))
        })?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredUsageObject {
    pub object_key: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub content_type: String,
    pub content_encoding: Option<String>,
    pub payload_format: String,
}

#[derive(Clone)]
pub struct UsageObjectStore {
    store: Arc<dyn ObjectStore>,
    prefix: Path,
}

impl fmt::Debug for UsageObjectStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UsageObjectStore")
            .field("store", &format!("{:?}", self.store))
            .field("prefix", &self.prefix.as_ref())
            .finish()
    }
}

impl UsageObjectStore {
    pub fn from_config(config: &UsageObjectStoreConfig) -> Result<Self, DataLayerError> {
        config.validate()?;
        let url = Url::parse(config.url.trim()).map_err(|err| {
            DataLayerError::InvalidInput(format!("invalid usage object store URL: {err}"))
        })?;

        let (store, url_prefix): (Arc<dyn ObjectStore>, Path) = match url.scheme() {
            "file" => {
                let (store, prefix) = object_store::parse_url(&url).map_err(object_store_error)?;
                (Arc::from(store), prefix)
            }
            "s3" | "s3a" | "https" => {
                let (_, prefix) =
                    object_store::ObjectStoreScheme::parse(&url).map_err(object_store_error)?;
                let store = AmazonS3Builder::from_env()
                    .with_url(url.as_str())
                    .build()
                    .map_err(object_store_error)?;
                (Arc::new(store), prefix)
            }
            scheme => {
                return Err(DataLayerError::InvalidInput(format!(
                    "unsupported usage object store URL scheme: {scheme}"
                )))
            }
        };

        let prefix = join_paths(&url_prefix, config.prefix.trim().trim_matches('/'))?;
        Ok(Self { store, prefix })
    }

    pub fn object_key(&self, request_id: &str, body_field: &str, extension: &str) -> String {
        let directory = self.object_directory(request_id, body_field);
        let version = uuid::Uuid::new_v4().simple().to_string();
        join_paths(&directory, &format!("{version}.{extension}"))
            .expect("versioned usage object key must be valid")
            .to_string()
    }

    pub fn content_object_key(
        &self,
        request_id: &str,
        body_field: &str,
        sha256: &str,
        extension: &str,
    ) -> String {
        let directory = self.object_directory(request_id, body_field);
        join_paths(&directory, &format!("{sha256}.{extension}"))
            .expect("content-addressed usage object key must be valid")
            .to_string()
    }

    fn object_directory(&self, request_id: &str, body_field: &str) -> Path {
        let digest = format!("{:x}", Sha256::digest(request_id.as_bytes()));
        let shard = &digest[..2];
        join_paths(&self.prefix, &format!("{shard}/{digest}/{body_field}"))
            .expect("hashed usage object directory must be valid")
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn put(
        &self,
        object_key: &str,
        payload: Vec<u8>,
        source_size_bytes: u64,
        source_sha256: String,
        content_type: &str,
        content_encoding: Option<&str>,
        payload_format: &str,
    ) -> Result<StoredUsageObject, DataLayerError> {
        let path = Path::parse(object_key).map_err(object_store_error)?;
        match self
            .store
            .put_opts(
                &path,
                Bytes::from(payload).into(),
                PutOptions {
                    mode: PutMode::Create,
                    ..PutOptions::default()
                },
            )
            .await
        {
            Ok(_) | Err(object_store::Error::AlreadyExists { .. }) => {}
            Err(err) => return Err(object_store_error(err)),
        }
        Ok(StoredUsageObject {
            object_key: object_key.to_string(),
            size_bytes: source_size_bytes,
            sha256: source_sha256,
            content_type: content_type.to_string(),
            content_encoding: content_encoding.map(ToOwned::to_owned),
            payload_format: payload_format.to_string(),
        })
    }

    pub async fn get(&self, object_key: &str) -> Result<Bytes, DataLayerError> {
        let path = Path::parse(object_key).map_err(object_store_error)?;
        self.store
            .get(&path)
            .await
            .map_err(object_store_error)?
            .bytes()
            .await
            .map_err(object_store_error)
    }

    pub async fn delete(&self, object_key: &str) -> Result<(), DataLayerError> {
        let path = Path::parse(object_key).map_err(object_store_error)?;
        match self.store.delete(&path).await {
            Ok(()) | Err(object_store::Error::NotFound { .. }) => Ok(()),
            Err(err) => Err(object_store_error(err)),
        }
    }

    pub async fn start_multipart(
        &self,
        object_key: &str,
    ) -> Result<UsageObjectStoreWriter, DataLayerError> {
        let path = Path::parse(object_key).map_err(object_store_error)?;
        let upload = self
            .store
            .put_multipart(&path)
            .await
            .map_err(object_store_error)?;
        Ok(UsageObjectStoreWriter {
            object_key: object_key.to_string(),
            upload: Some(upload),
            pending: Vec::new(),
            uploaded_bytes: 0,
            sha256: Sha256::new(),
        })
    }
}

pub struct UsageObjectStoreWriter {
    object_key: String,
    upload: Option<Box<dyn MultipartUpload>>,
    pending: Vec<u8>,
    uploaded_bytes: u64,
    sha256: Sha256,
}

impl fmt::Debug for UsageObjectStoreWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UsageObjectStoreWriter")
            .field("object_key", &self.object_key)
            .field("pending_bytes", &self.pending.len())
            .field("uploaded_bytes", &self.uploaded_bytes)
            .finish()
    }
}

impl UsageObjectStoreWriter {
    pub async fn write(&mut self, bytes: &[u8]) -> Result<(), DataLayerError> {
        const PART_BYTES: usize = 5 * 1024 * 1024;
        if bytes.is_empty() {
            return Ok(());
        }
        self.sha256.update(bytes);
        self.uploaded_bytes = self.uploaded_bytes.saturating_add(bytes.len() as u64);
        self.pending.extend_from_slice(bytes);

        while self.pending.len() >= PART_BYTES {
            let remaining = self.pending.split_off(PART_BYTES);
            let part = std::mem::replace(&mut self.pending, remaining);
            self.put_part(part).await?;
        }
        Ok(())
    }

    pub async fn complete(
        mut self,
        content_type: &str,
        payload_format: &str,
    ) -> Result<StoredUsageObject, DataLayerError> {
        if !self.pending.is_empty() {
            let part = std::mem::take(&mut self.pending);
            if let Err(err) = self.put_part(part).await {
                let _ = self.abort_active_upload().await;
                return Err(err);
            }
        }
        let mut upload = self.upload.take().ok_or_else(|| {
            DataLayerError::UnexpectedValue("usage object upload is no longer active".to_string())
        })?;
        if let Err(err) = upload.complete().await {
            let _ = upload.abort().await;
            return Err(object_store_error(err));
        }
        Ok(StoredUsageObject {
            object_key: self.object_key,
            size_bytes: self.uploaded_bytes,
            sha256: format!("{:x}", self.sha256.finalize()),
            content_type: content_type.to_string(),
            content_encoding: None,
            payload_format: payload_format.to_string(),
        })
    }

    pub async fn abort(mut self) -> Result<(), DataLayerError> {
        self.abort_active_upload().await
    }

    async fn put_part(&mut self, part: Vec<u8>) -> Result<(), DataLayerError> {
        let upload = self.upload.as_mut().ok_or_else(|| {
            DataLayerError::UnexpectedValue("usage object upload is no longer active".to_string())
        })?;
        upload
            .put_part(Bytes::from(part).into())
            .await
            .map_err(object_store_error)
    }

    async fn abort_active_upload(&mut self) -> Result<(), DataLayerError> {
        if let Some(mut upload) = self.upload.take() {
            upload.abort().await.map_err(object_store_error)?;
        }
        Ok(())
    }
}

fn join_paths(prefix: &Path, suffix: &str) -> Result<Path, DataLayerError> {
    let prefix = prefix.as_ref().trim_matches('/');
    let suffix = suffix.trim_matches('/');
    let joined = match (prefix.is_empty(), suffix.is_empty()) {
        (true, true) => String::new(),
        (true, false) => suffix.to_string(),
        (false, true) => prefix.to_string(),
        (false, false) => format!("{prefix}/{suffix}"),
    };
    Path::parse(joined).map_err(object_store_error)
}

fn object_store_error(err: impl fmt::Display) -> DataLayerError {
    DataLayerError::UnexpectedValue(format!("usage object store operation failed: {err}"))
}

#[cfg(test)]
mod tests {
    use super::{UsageObjectStore, UsageObjectStoreConfig};
    use sha2::Digest as _;

    #[tokio::test]
    async fn file_store_round_trips_object() {
        let root = std::env::temp_dir().join(format!(
            "aether-usage-object-store-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).expect("create test object store");
        let config =
            UsageObjectStoreConfig::new(format!("file://{}", root.display()), "usage-test");
        let store = UsageObjectStore::from_config(&config).expect("build file store");
        let key = store.object_key("request-1", "response_body", "json");
        store
            .put(
                &key,
                br#"{"ok":true}"#.to_vec(),
                11,
                "hash".to_string(),
                "application/json",
                None,
                "json",
            )
            .await
            .expect("write object");
        let bytes = store.get(&key).await.expect("read object");
        assert_eq!(bytes.as_ref(), br#"{"ok":true}"#);
        store.delete(&key).await.expect("delete object");
        store
            .delete(&key)
            .await
            .expect("repeated delete should be idempotent");
        std::fs::remove_dir_all(root).expect("remove test object store");
    }

    #[tokio::test]
    async fn file_store_round_trips_multipart_object() {
        let root = std::env::temp_dir().join(format!(
            "aether-usage-object-store-multipart-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).expect("create multipart test object store");
        let config =
            UsageObjectStoreConfig::new(format!("file://{}", root.display()), "usage-test");
        let store = UsageObjectStore::from_config(&config).expect("build file store");
        let key = store.object_key("request-large-1", "response_body", "raw");
        let first = vec![b'a'; 5 * 1024 * 1024];
        let second = vec![b'b'; 1024 * 1024 + 17];
        let mut writer = store
            .start_multipart(&key)
            .await
            .expect("start multipart upload");
        writer.write(&first).await.expect("write first part");
        writer.write(&second).await.expect("write second part");
        let stored = writer
            .complete("application/octet-stream", "raw")
            .await
            .expect("complete multipart upload");

        let mut expected = first;
        expected.extend_from_slice(&second);
        assert_eq!(stored.size_bytes, expected.len() as u64);
        assert_eq!(
            stored.sha256,
            format!("{:x}", sha2::Sha256::digest(&expected))
        );
        assert_eq!(
            store
                .get(&key)
                .await
                .expect("read multipart object")
                .as_ref(),
            expected.as_slice()
        );

        store.delete(&key).await.expect("delete multipart object");
        std::fs::remove_dir_all(root).expect("remove multipart test object store");
    }

    #[test]
    fn repeated_request_uses_immutable_object_versions() {
        let root = std::env::temp_dir().join(format!(
            "aether-usage-object-store-keys-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).expect("create object key test store");
        let config =
            UsageObjectStoreConfig::new(format!("file://{}", root.display()), "usage-test");
        let store = UsageObjectStore::from_config(&config).expect("build file store");

        let first = store.object_key("request-repeat-1", "response_body", "json.gz");
        let second = store.object_key("request-repeat-1", "response_body", "json.gz");
        let content_first = store.content_object_key(
            "request-repeat-1",
            "response_body",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "json.gz",
        );
        let content_second = store.content_object_key(
            "request-repeat-1",
            "response_body",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "json.gz",
        );

        assert_ne!(first, second);
        assert!(first.contains("/response_body/"));
        assert!(second.contains("/response_body/"));
        assert_eq!(content_first, content_second);
        assert!(content_first.ends_with(
            "/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.json.gz"
        ));
        std::fs::remove_dir_all(root).expect("remove object key test store");
    }

    #[test]
    fn rejects_unsupported_scheme() {
        let config = UsageObjectStoreConfig::new("ftp://example.com/bucket", "usage");
        assert!(config.validate().is_err());
    }
}
