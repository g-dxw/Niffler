CREATE TABLE IF NOT EXISTS niffler_stability_observations (
    id TEXT PRIMARY KEY NOT NULL,
    window_start_unix_ms INTEGER NOT NULL,
    window_end_unix_ms INTEGER NOT NULL,
    status TEXT NOT NULL,
    rollback_drill_status TEXT NOT NULL DEFAULT 'not_recorded',
    consistency_checked_count INTEGER NOT NULL DEFAULT 0,
    consistency_issue_count INTEGER NOT NULL DEFAULT 0,
    unknown_upstream_count INTEGER NOT NULL DEFAULT 0,
    legacy_write_call_count INTEGER NOT NULL DEFAULT 0,
    billing_reservation_exception_count INTEGER NOT NULL DEFAULT 0,
    referral_exception_count INTEGER NOT NULL DEFAULT 0,
    blocker_codes TEXT NOT NULL,
    summary TEXT,
    created_at_unix_ms INTEGER NOT NULL,
    updated_at_unix_ms INTEGER NOT NULL,
    UNIQUE (window_start_unix_ms, window_end_unix_ms)
);

CREATE INDEX IF NOT EXISTS idx_niffler_stability_observations_status_time ON niffler_stability_observations (status, window_end_unix_ms);
CREATE INDEX IF NOT EXISTS idx_niffler_stability_observations_window ON niffler_stability_observations (window_end_unix_ms);
