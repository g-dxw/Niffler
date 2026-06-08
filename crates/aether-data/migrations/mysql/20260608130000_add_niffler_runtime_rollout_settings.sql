CREATE TABLE IF NOT EXISTS niffler_runtime_rollout_settings (
    id VARCHAR(36) NOT NULL,
    target_scope VARCHAR(32) NOT NULL,
    target_id VARCHAR(64) NOT NULL,
    enable_new_routing BOOLEAN NOT NULL DEFAULT FALSE,
    enable_settlement_snapshot BOOLEAN NOT NULL DEFAULT FALSE,
    enable_error_return_rules BOOLEAN NOT NULL DEFAULT FALSE,
    enable_billing_reservation BOOLEAN NOT NULL DEFAULT FALSE,
    enable_referral_ledger BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    config JSON,
    created_at_unix_ms BIGINT NOT NULL,
    updated_at_unix_ms BIGINT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_niffler_runtime_rollout_settings_target (target_scope, target_id),
    INDEX idx_niffler_runtime_rollout_settings_active (is_active),
    CONSTRAINT ck_niffler_runtime_rollout_settings_scope
        CHECK (target_scope IN ('api_key', 'product_plan'))
);
