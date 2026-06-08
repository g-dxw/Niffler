CREATE TABLE IF NOT EXISTS niffler_api_key_product_plan_bindings (
    id TEXT PRIMARY KEY,
    api_key_id TEXT NOT NULL,
    product_plan_id TEXT NOT NULL REFERENCES niffler_product_plans (id),
    config TEXT,
    created_at_unix_ms INTEGER NOT NULL,
    updated_at_unix_ms INTEGER NOT NULL,
    UNIQUE (api_key_id)
);

CREATE INDEX IF NOT EXISTS idx_niffler_api_key_product_plan_bindings_plan
    ON niffler_api_key_product_plan_bindings (product_plan_id);
