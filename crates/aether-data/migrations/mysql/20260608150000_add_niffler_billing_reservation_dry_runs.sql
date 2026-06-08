CREATE TABLE IF NOT EXISTS niffler_billing_reservation_dry_runs (
    id VARCHAR(36) NOT NULL,
    request_id VARCHAR(100) NOT NULL,
    user_id VARCHAR(36),
    api_key_id VARCHAR(36),
    product_plan_id VARCHAR(36),
    requested_model_name VARCHAR(200) NOT NULL,
    estimated_reservation_usd DECIMAL(20, 8) NOT NULL DEFAULT 0,
    legacy_final_charge_usd DECIMAL(20, 8) NOT NULL DEFAULT 0,
    difference_usd DECIMAL(20, 8) NOT NULL DEFAULT 0,
    estimation_source VARCHAR(64) NOT NULL,
    status VARCHAR(32) NOT NULL,
    created_at_unix_ms BIGINT NOT NULL,
    finalized_at_unix_ms BIGINT,
    PRIMARY KEY (id),
    UNIQUE KEY uq_niffler_billing_reservation_dry_runs_request (request_id),
    INDEX idx_niffler_billing_reservation_dry_runs_status_time (status, created_at_unix_ms),
    INDEX idx_niffler_billing_reservation_dry_runs_user_time (user_id, created_at_unix_ms),
    INDEX idx_niffler_billing_reservation_dry_runs_key_time (api_key_id, created_at_unix_ms),
    CHECK (status IN ('matched', 'over_reserved', 'under_reserved')),
    CHECK (
        estimated_reservation_usd >= 0
        AND legacy_final_charge_usd >= 0
    )
);
