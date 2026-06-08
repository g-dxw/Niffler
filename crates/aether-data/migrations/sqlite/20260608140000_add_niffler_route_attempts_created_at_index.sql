CREATE INDEX IF NOT EXISTS idx_niffler_route_attempts_created_at
    ON niffler_route_attempts (created_at_unix_ms DESC);
