CREATE TABLE IF NOT EXISTS public.niffler_billing_reservation_dry_runs (
    id character varying(36) PRIMARY KEY,
    request_id character varying(100) NOT NULL,
    user_id character varying(36),
    api_key_id character varying(36),
    product_plan_id character varying(36),
    requested_model_name character varying(200) NOT NULL,
    estimated_reservation_usd numeric(20, 8) DEFAULT 0 NOT NULL,
    legacy_final_charge_usd numeric(20, 8) DEFAULT 0 NOT NULL,
    difference_usd numeric(20, 8) DEFAULT 0 NOT NULL,
    estimation_source character varying(64) NOT NULL,
    status character varying(32) NOT NULL,
    created_at_unix_ms bigint NOT NULL,
    finalized_at_unix_ms bigint,
    CONSTRAINT uq_niffler_billing_reservation_dry_runs_request
        UNIQUE (request_id),
    CONSTRAINT ck_niffler_billing_reservation_dry_runs_status
        CHECK (status IN ('matched', 'over_reserved', 'under_reserved')),
    CONSTRAINT ck_niffler_billing_reservation_dry_runs_non_negative
        CHECK (
            estimated_reservation_usd >= 0
            AND legacy_final_charge_usd >= 0
        )
);

CREATE INDEX IF NOT EXISTS idx_niffler_billing_reservation_dry_runs_status_time
    ON public.niffler_billing_reservation_dry_runs (status, created_at_unix_ms DESC);

CREATE INDEX IF NOT EXISTS idx_niffler_billing_reservation_dry_runs_user_time
    ON public.niffler_billing_reservation_dry_runs (user_id, created_at_unix_ms DESC);

CREATE INDEX IF NOT EXISTS idx_niffler_billing_reservation_dry_runs_key_time
    ON public.niffler_billing_reservation_dry_runs (api_key_id, created_at_unix_ms DESC);
