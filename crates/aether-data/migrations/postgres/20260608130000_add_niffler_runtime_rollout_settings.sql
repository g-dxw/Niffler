CREATE TABLE IF NOT EXISTS public.niffler_runtime_rollout_settings (
    id character varying(36) PRIMARY KEY,
    target_scope character varying(32) NOT NULL,
    target_id character varying(64) NOT NULL,
    enable_new_routing boolean NOT NULL DEFAULT false,
    enable_settlement_snapshot boolean NOT NULL DEFAULT false,
    enable_error_return_rules boolean NOT NULL DEFAULT false,
    enable_billing_reservation boolean NOT NULL DEFAULT false,
    enable_referral_ledger boolean NOT NULL DEFAULT false,
    is_active boolean NOT NULL DEFAULT true,
    config jsonb,
    created_at_unix_ms bigint NOT NULL,
    updated_at_unix_ms bigint NOT NULL,
    CONSTRAINT ck_niffler_runtime_rollout_settings_scope
        CHECK (target_scope IN ('api_key', 'product_plan')),
    CONSTRAINT uq_niffler_runtime_rollout_settings_target
        UNIQUE (target_scope, target_id)
);

CREATE INDEX IF NOT EXISTS idx_niffler_runtime_rollout_settings_active
    ON public.niffler_runtime_rollout_settings (is_active);
