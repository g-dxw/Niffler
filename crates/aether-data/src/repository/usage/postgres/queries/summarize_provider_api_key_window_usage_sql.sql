WITH requested AS (
  SELECT
    request_row.provider_api_key_id,
    lower(BTRIM(request_row.window_scope)) AS window_scope,
    lower(BTRIM(request_row.window_code)) AS window_code,
    request_row.start_unix_secs,
    request_row.end_unix_secs,
    request_row.ordinality
  FROM UNNEST(
    $1::TEXT[],
    $2::TEXT[],
    $3::TEXT[],
    $4::BIGINT[],
    $5::BIGINT[]
  ) WITH ORDINALITY AS request_row(
    provider_api_key_id,
    window_scope,
    window_code,
    start_unix_secs,
    end_unix_secs,
    ordinality
  )
)
SELECT
  requested.provider_api_key_id,
  requested.window_scope,
  requested.window_code,
  requested.start_unix_secs,
  requested.end_unix_secs,
  counters.request_count,
  counters.total_tokens,
  CAST(counters.total_cost_usd AS DOUBLE PRECISION) AS total_cost_usd
FROM requested
JOIN provider_api_key_window_usage_counters AS counters
 ON counters.provider_api_key_id = requested.provider_api_key_id
 AND counters.window_scope = requested.window_scope
 AND counters.window_code = requested.window_code
 AND counters.window_start_unix_secs = requested.start_unix_secs
 AND counters.window_end_unix_secs = requested.end_unix_secs
 AND counters.rebuilt_at IS NOT NULL
ORDER BY requested.ordinality ASC
