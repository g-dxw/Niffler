CREATE INDEX IF NOT EXISTS idx_niffler_settlement_snapshots_created_at
    ON public.niffler_settlement_snapshots (created_at_unix_ms DESC, request_id);

CREATE INDEX IF NOT EXISTS idx_niffler_settlement_snapshots_key_time
    ON public.niffler_settlement_snapshots (api_key_id, created_at_unix_ms DESC);

CREATE INDEX IF NOT EXISTS idx_niffler_settlement_snapshots_plan_time
    ON public.niffler_settlement_snapshots (product_plan_id, created_at_unix_ms DESC);
