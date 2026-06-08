CREATE TABLE IF NOT EXISTS niffler_billing_reservation_dry_runs (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    user_id TEXT,
    api_key_id TEXT,
    product_plan_id TEXT,
    requested_model_name TEXT NOT NULL,
    estimated_reservation_usd REAL NOT NULL DEFAULT 0,
    legacy_final_charge_usd REAL NOT NULL DEFAULT 0,
    difference_usd REAL NOT NULL DEFAULT 0,
    estimation_source TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at_unix_ms INTEGER NOT NULL,
    finalized_at_unix_ms INTEGER,
    UNIQUE (request_id),
    CHECK (status IN ('matched', 'over_reserved', 'under_reserved')),
    CHECK (
        estimated_reservation_usd >= 0
        AND legacy_final_charge_usd >= 0
    )
);

CREATE INDEX IF NOT EXISTS idx_niffler_billing_reservation_dry_runs_status_time
    ON niffler_billing_reservation_dry_runs (status, created_at_unix_ms DESC);

CREATE INDEX IF NOT EXISTS idx_niffler_billing_reservation_dry_runs_user_time
    ON niffler_billing_reservation_dry_runs (user_id, created_at_unix_ms DESC);

CREATE INDEX IF NOT EXISTS idx_niffler_billing_reservation_dry_runs_key_time
    ON niffler_billing_reservation_dry_runs (api_key_id, created_at_unix_ms DESC);
