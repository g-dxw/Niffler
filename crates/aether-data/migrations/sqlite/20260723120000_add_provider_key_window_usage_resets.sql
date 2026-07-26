CREATE TABLE IF NOT EXISTS provider_api_key_window_usage_resets (
    provider_api_key_id TEXT NOT NULL,
    window_scope TEXT NOT NULL DEFAULT 'account',
    window_start_unix_secs INTEGER NOT NULL,
    window_end_unix_secs INTEGER NOT NULL,
    usage_reset_at_unix_secs INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (
        provider_api_key_id,
        window_scope,
        window_start_unix_secs,
        window_end_unix_secs
    ),
    FOREIGN KEY (provider_api_key_id)
        REFERENCES provider_api_keys(id)
        ON DELETE CASCADE
);
