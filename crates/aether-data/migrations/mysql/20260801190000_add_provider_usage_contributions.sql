CREATE TABLE IF NOT EXISTS provider_api_key_usage_contributions (
    request_id VARCHAR(128) NOT NULL,
    provider_api_key_id VARCHAR(64),
    request_count BIGINT NOT NULL DEFAULT 0,
    success_count BIGINT NOT NULL DEFAULT 0,
    error_count BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    total_cost_usd DECIMAL(20,8) NOT NULL DEFAULT 0,
    total_response_time_ms BIGINT NOT NULL DEFAULT 0,
    last_used_at_unix_secs BIGINT,
    usage_created_at_unix_secs BIGINT,
    window_request_count BIGINT NOT NULL DEFAULT 0,
    window_total_tokens BIGINT NOT NULL DEFAULT 0,
    window_total_cost_usd DECIMAL(20,8) NOT NULL DEFAULT 0,
    revision BIGINT NOT NULL DEFAULT 0,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (request_id),
    CONSTRAINT provider_api_key_usage_contributions_key_fkey
        FOREIGN KEY (provider_api_key_id)
        REFERENCES provider_api_keys(id)
        ON DELETE CASCADE
);

CREATE INDEX ix_provider_api_key_usage_contributions_key
    ON provider_api_key_usage_contributions (provider_api_key_id, usage_created_at_unix_secs);

CREATE TABLE IF NOT EXISTS provider_api_key_usage_contribution_backfill_state (
    id INT NOT NULL DEFAULT 1,
    high_water_created_at BIGINT,
    initialized_at BIGINT,
    completed_at BIGINT,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS provider_api_key_usage_contribution_backfills (
    provider_api_key_id VARCHAR(64) NOT NULL,
    high_water_created_at BIGINT,
    cursor_created_at BIGINT,
    cursor_request_id VARCHAR(128),
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    backfilled_at BIGINT,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (provider_api_key_id),
    KEY ix_provider_api_key_usage_contribution_backfills_ready (status, updated_at),
    CONSTRAINT provider_api_key_usage_contribution_backfills_key_fkey
        FOREIGN KEY (provider_api_key_id)
        REFERENCES provider_api_keys(id)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS provider_api_key_usage_projection_repairs (
    provider_api_key_id VARCHAR(64) NOT NULL,
    main_requested BOOLEAN NOT NULL DEFAULT FALSE,
    window_requested BOOLEAN NOT NULL DEFAULT FALSE,
    available_at BIGINT NOT NULL,
    attempts INT NOT NULL DEFAULT 0,
    last_error LONGTEXT,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (provider_api_key_id),
    KEY ix_provider_api_key_usage_projection_repairs_ready (available_at, updated_at),
    CONSTRAINT provider_api_key_usage_projection_repairs_key_fkey
        FOREIGN KEY (provider_api_key_id)
        REFERENCES provider_api_keys(id)
        ON DELETE CASCADE
);
