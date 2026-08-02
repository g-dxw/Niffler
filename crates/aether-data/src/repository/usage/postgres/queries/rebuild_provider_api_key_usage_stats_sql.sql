WITH aggregated AS (
  SELECT
    provider_api_key_id,
    COALESCE(SUM(request_count), 0)::BIGINT AS request_count,
    COALESCE(SUM(success_count), 0)::BIGINT AS success_count,
    COALESCE(SUM(error_count), 0)::BIGINT AS error_count,
    COALESCE(SUM(total_tokens), 0)::BIGINT AS total_tokens,
    COALESCE(SUM(total_cost_usd), 0)::NUMERIC(20,8) AS total_cost_usd,
    COALESCE(SUM(total_response_time_ms), 0)::BIGINT AS total_response_time_ms,
    MAX(last_used_at_unix_secs) AS last_used_at_unix_secs
  FROM public.provider_api_key_usage_contributions
  WHERE provider_api_key_id IS NOT NULL
    AND BTRIM(provider_api_key_id) <> ''
  GROUP BY provider_api_key_id
)
UPDATE public.provider_api_keys AS keys
SET
  request_count = COALESCE(aggregated.request_count, 0),
  success_count = COALESCE(aggregated.success_count, 0),
  error_count = COALESCE(aggregated.error_count, 0),
  total_tokens = COALESCE(aggregated.total_tokens, 0),
  total_cost_usd = COALESCE(aggregated.total_cost_usd, 0),
  total_response_time_ms = COALESCE(aggregated.total_response_time_ms, 0),
  last_used_at = CASE
    WHEN aggregated.last_used_at_unix_secs IS NULL THEN NULL
    ELSE TO_TIMESTAMP(aggregated.last_used_at_unix_secs::DOUBLE PRECISION)
  END,
  updated_at = NOW()
FROM aggregated
WHERE keys.id = aggregated.provider_api_key_id
