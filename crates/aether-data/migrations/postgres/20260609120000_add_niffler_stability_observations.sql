CREATE TABLE IF NOT EXISTS public.niffler_stability_observations (
    id character varying(36) NOT NULL,
    window_start_unix_ms bigint NOT NULL,
    window_end_unix_ms bigint NOT NULL,
    status character varying(32) NOT NULL,
    rollback_drill_status character varying(32) DEFAULT 'not_recorded' NOT NULL,
    consistency_checked_count bigint DEFAULT 0 NOT NULL,
    consistency_issue_count bigint DEFAULT 0 NOT NULL,
    unknown_upstream_count bigint DEFAULT 0 NOT NULL,
    legacy_write_call_count bigint DEFAULT 0 NOT NULL,
    billing_reservation_exception_count bigint DEFAULT 0 NOT NULL,
    referral_exception_count bigint DEFAULT 0 NOT NULL,
    blocker_codes jsonb NOT NULL,
    summary jsonb,
    created_at_unix_ms bigint NOT NULL,
    updated_at_unix_ms bigint NOT NULL
);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'niffler_stability_observations_pkey'
    ) THEN
        ALTER TABLE ONLY public.niffler_stability_observations
            ADD CONSTRAINT niffler_stability_observations_pkey PRIMARY KEY (id);
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'uq_niffler_stability_observations_window'
    ) THEN
        ALTER TABLE ONLY public.niffler_stability_observations
            ADD CONSTRAINT uq_niffler_stability_observations_window UNIQUE (window_start_unix_ms, window_end_unix_ms);
    END IF;
END $$;
CREATE INDEX IF NOT EXISTS idx_niffler_stability_observations_status_time ON public.niffler_stability_observations USING btree (status, window_end_unix_ms);
CREATE INDEX IF NOT EXISTS idx_niffler_stability_observations_window ON public.niffler_stability_observations USING btree (window_end_unix_ms);
