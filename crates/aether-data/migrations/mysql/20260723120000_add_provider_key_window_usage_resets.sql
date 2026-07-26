CREATE TABLE IF NOT EXISTS provider_api_key_window_usage_resets (
    provider_api_key_id VARCHAR(64) NOT NULL,
    window_scope VARCHAR(64) NOT NULL DEFAULT 'account',
    window_start_unix_secs BIGINT NOT NULL,
    window_end_unix_secs BIGINT NOT NULL,
    usage_reset_at_unix_secs BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (
        provider_api_key_id,
        window_scope,
        window_start_unix_secs,
        window_end_unix_secs
    ),
    CONSTRAINT provider_api_key_window_usage_resets_key_fkey
        FOREIGN KEY (provider_api_key_id)
        REFERENCES provider_api_keys(id)
        ON DELETE CASCADE
);
