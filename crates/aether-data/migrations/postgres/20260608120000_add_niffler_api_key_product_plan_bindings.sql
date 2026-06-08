CREATE TABLE IF NOT EXISTS public.niffler_api_key_product_plan_bindings (
    id character varying(36) PRIMARY KEY,
    api_key_id character varying(36) NOT NULL,
    product_plan_id character varying(36) NOT NULL
        REFERENCES public.niffler_product_plans (id),
    config jsonb,
    created_at_unix_ms bigint NOT NULL,
    updated_at_unix_ms bigint NOT NULL,
    CONSTRAINT uq_niffler_api_key_product_plan_bindings_key
        UNIQUE (api_key_id)
);

CREATE INDEX IF NOT EXISTS idx_niffler_api_key_product_plan_bindings_plan
    ON public.niffler_api_key_product_plan_bindings (product_plan_id);
