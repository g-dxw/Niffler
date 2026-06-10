-- no-transaction
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_usage_provider_api_key_created_at
    ON public.usage USING btree (provider_api_key_id, created_at);
