CREATE TABLE IF NOT EXISTS provider_api_key_usage_contributions (
    request_id TEXT NOT NULL PRIMARY KEY,
    provider_api_key_id TEXT,
    request_count INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    total_cost_usd REAL NOT NULL DEFAULT 0,
    total_response_time_ms INTEGER NOT NULL DEFAULT 0,
    last_used_at_unix_secs INTEGER,
    usage_created_at_unix_secs INTEGER,
    window_request_count INTEGER NOT NULL DEFAULT 0,
    window_total_tokens INTEGER NOT NULL DEFAULT 0,
    window_total_cost_usd REAL NOT NULL DEFAULT 0,
    revision INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (provider_api_key_id)
        REFERENCES provider_api_keys(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_provider_api_key_usage_contributions_key
    ON provider_api_key_usage_contributions (provider_api_key_id, usage_created_at_unix_secs);

CREATE TABLE IF NOT EXISTS provider_api_key_usage_contribution_backfill_state (
    id INTEGER NOT NULL PRIMARY KEY DEFAULT 1,
    high_water_created_at INTEGER,
    initialized_at INTEGER,
    completed_at INTEGER,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS provider_api_key_usage_contribution_backfills (
    provider_api_key_id TEXT NOT NULL PRIMARY KEY,
    high_water_created_at INTEGER,
    cursor_created_at INTEGER,
    cursor_request_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    backfilled_at INTEGER,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (provider_api_key_id)
        REFERENCES provider_api_keys(id)
        ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS ix_provider_api_key_usage_contribution_backfills_ready
    ON provider_api_key_usage_contribution_backfills (status, updated_at);

CREATE TABLE IF NOT EXISTS provider_api_key_usage_projection_repairs (
    provider_api_key_id TEXT NOT NULL PRIMARY KEY,
    main_requested INTEGER NOT NULL DEFAULT 0,
    window_requested INTEGER NOT NULL DEFAULT 0,
    available_at INTEGER NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (provider_api_key_id)
        REFERENCES provider_api_keys(id)
        ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS ix_provider_api_key_usage_projection_repairs_ready
    ON provider_api_key_usage_projection_repairs (available_at, updated_at);
