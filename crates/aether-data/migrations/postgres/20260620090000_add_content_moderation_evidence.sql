CREATE TABLE IF NOT EXISTS content_moderation_evidence (
  id TEXT PRIMARY KEY,
  request_id TEXT NOT NULL,
  user_id TEXT NULL,
  api_key_id TEXT NULL,
  provider_id TEXT NULL,
  upstream_service_id TEXT NULL,
  upstream_account_id TEXT NULL,
  moderation_model TEXT NOT NULL,
  input_sha256 TEXT NOT NULL,
  input_text TEXT NULL,
  categories JSONB NOT NULL DEFAULT '{}'::jsonb,
  category_scores JSONB NOT NULL DEFAULT '{}'::jsonb,
  flagged BOOLEAN NOT NULL DEFAULT FALSE,
  created_at_unix_secs BIGINT NOT NULL,
  expires_at_unix_secs BIGINT NOT NULL,
  redacted_at_unix_secs BIGINT NULL
);

CREATE INDEX IF NOT EXISTS idx_content_moderation_evidence_request
  ON content_moderation_evidence (request_id);

CREATE INDEX IF NOT EXISTS idx_content_moderation_evidence_expiry
  ON content_moderation_evidence (expires_at_unix_secs, id)
  WHERE input_text IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_content_moderation_evidence_actor
  ON content_moderation_evidence (user_id, api_key_id, created_at_unix_secs);
