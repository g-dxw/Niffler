CREATE TABLE IF NOT EXISTS niffler_runtime_rollout_settings (
    id TEXT PRIMARY KEY,
    target_scope TEXT NOT NULL CHECK (target_scope IN ('api_key', 'product_plan')),
    target_id TEXT NOT NULL,
    enable_new_routing INTEGER NOT NULL DEFAULT 0,
    enable_settlement_snapshot INTEGER NOT NULL DEFAULT 0,
    enable_error_return_rules INTEGER NOT NULL DEFAULT 0,
    enable_billing_reservation INTEGER NOT NULL DEFAULT 0,
    enable_referral_ledger INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    config TEXT,
    created_at_unix_ms INTEGER NOT NULL,
    updated_at_unix_ms INTEGER NOT NULL,
    UNIQUE (target_scope, target_id)
);

CREATE INDEX IF NOT EXISTS idx_niffler_runtime_rollout_settings_active
    ON niffler_runtime_rollout_settings (is_active);
