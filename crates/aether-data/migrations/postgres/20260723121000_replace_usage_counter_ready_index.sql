-- no-transaction
CREATE INDEX CONCURRENTLY IF NOT EXISTS ix_usage_counter_deltas_ready
    ON public.usage_counter_deltas (available_at, created_at, id)
    WHERE processed_at IS NULL;

DROP INDEX CONCURRENTLY IF EXISTS public.ix_usage_counter_deltas_unprocessed;
