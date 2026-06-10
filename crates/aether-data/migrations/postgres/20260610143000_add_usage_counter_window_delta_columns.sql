ALTER TABLE IF EXISTS public.usage_counter_deltas
  ADD COLUMN IF NOT EXISTS window_request_count_delta bigint,
  ADD COLUMN IF NOT EXISTS window_total_tokens_delta bigint,
  ADD COLUMN IF NOT EXISTS window_total_cost_usd_delta double precision;
