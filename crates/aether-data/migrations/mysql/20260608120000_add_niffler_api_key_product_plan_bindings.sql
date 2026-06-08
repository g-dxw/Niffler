CREATE TABLE IF NOT EXISTS niffler_api_key_product_plan_bindings (
    id VARCHAR(36) NOT NULL,
    api_key_id VARCHAR(36) NOT NULL,
    product_plan_id VARCHAR(36) NOT NULL,
    config JSON,
    created_at_unix_ms BIGINT NOT NULL,
    updated_at_unix_ms BIGINT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_niffler_api_key_product_plan_bindings_key (api_key_id),
    INDEX idx_niffler_api_key_product_plan_bindings_plan (product_plan_id),
    CONSTRAINT fk_niffler_api_key_product_plan_bindings_plan
        FOREIGN KEY (product_plan_id) REFERENCES niffler_product_plans (id)
);
