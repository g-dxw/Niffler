WITH target AS (
  SELECT
    keys.id,
    COALESCE(keys.status_snapshot::jsonb, '{}'::jsonb) AS status_snapshot
  FROM provider_api_keys AS keys
  JOIN providers
    ON providers.id = keys.provider_id
  WHERE keys.id = $1
    AND lower(BTRIM(COALESCE(providers.provider_type, ''))) = 'codex'
),
window_items AS (
  SELECT
    target.id,
    item.window_item
  FROM target
  CROSS JOIN LATERAL jsonb_array_elements(
    CASE
      WHEN jsonb_typeof(target.status_snapshot #> '{quota,windows}') = 'array'
      THEN target.status_snapshot #> '{quota,windows}'
      ELSE '[]'::jsonb
    END
  ) AS item(window_item)
),
parsed_windows AS (
  SELECT
    window_items.id,
    lower(BTRIM(COALESCE(window_item ->> 'code', ''))) AS window_code,
    lower(BTRIM(COALESCE(window_item ->> 'scope', 'account'))) AS window_scope,
    CASE
      WHEN BTRIM(COALESCE(window_item ->> 'reset_at', '')) ~ '^[0-9]+$'
       AND length(BTRIM(window_item ->> 'reset_at')) <= 19
       AND (
         length(BTRIM(window_item ->> 'reset_at')) < 19
         OR BTRIM(window_item ->> 'reset_at') <= '9223372036854775807'
       )
      THEN CAST(window_item ->> 'reset_at' AS BIGINT)
      ELSE NULL
    END AS reset_at,
    COALESCE(
      CASE
        WHEN BTRIM(COALESCE(window_item ->> 'window_seconds', '')) ~ '^[0-9]+$'
         AND length(BTRIM(window_item ->> 'window_seconds')) <= 19
        THEN CAST(window_item ->> 'window_seconds' AS BIGINT)
        ELSE NULL
      END,
      CASE
        WHEN BTRIM(COALESCE(window_item ->> 'window_minutes', '')) ~ '^[0-9]+$'
         AND length(BTRIM(window_item ->> 'window_minutes')) <= 16
        THEN CAST(window_item ->> 'window_minutes' AS BIGINT) * 60
        ELSE NULL
      END,
      CASE lower(BTRIM(COALESCE(window_item ->> 'code', '')))
        WHEN '5h' THEN 18000
        WHEN '7d' THEN 604800
        WHEN 'weekly' THEN 604800
        WHEN '1m' THEN 2592000
        WHEN 'monthly' THEN 2592000
        ELSE NULL
      END
    ) AS window_seconds
  FROM window_items
),
natural_windows AS (
  SELECT DISTINCT
    id,
    window_code,
    window_scope,
    reset_at - window_seconds AS window_start_unix_secs,
    reset_at AS window_end_unix_secs
  FROM parsed_windows
  WHERE window_code <> ''
    AND window_scope NOT IN ('feature', 'model', 'workspace')
    AND reset_at IS NOT NULL
    AND window_seconds IS NOT NULL
    AND window_seconds > 0
    AND reset_at >= window_seconds
),
valid_windows AS (
  SELECT
    natural_windows.*,
    GREATEST(
      natural_windows.window_start_unix_secs,
      COALESCE(resets.usage_reset_at_unix_secs, natural_windows.window_start_unix_secs)
    ) AS usage_start_unix_secs
  FROM natural_windows
  LEFT JOIN provider_api_key_window_usage_resets AS resets
    ON resets.provider_api_key_id = natural_windows.id
   AND resets.window_scope = natural_windows.window_scope
   AND resets.window_start_unix_secs = natural_windows.window_start_unix_secs
   AND resets.window_end_unix_secs = natural_windows.window_end_unix_secs
),
aggregated AS (
  SELECT
    valid_windows.id AS provider_api_key_id,
    valid_windows.window_scope,
    valid_windows.window_code,
    valid_windows.window_start_unix_secs,
    valid_windows.window_end_unix_secs,
    COALESCE(SUM(contributions.window_request_count), 0)::BIGINT AS request_count,
    COALESCE(SUM(contributions.window_total_tokens), 0)::BIGINT AS total_tokens,
    COALESCE(SUM(contributions.window_total_cost_usd), 0)::NUMERIC(20,8) AS total_cost_usd
  FROM valid_windows
  LEFT JOIN provider_api_key_usage_contributions AS contributions
    ON contributions.provider_api_key_id = valid_windows.id
   AND contributions.usage_created_at_unix_secs >= valid_windows.usage_start_unix_secs
   AND contributions.usage_created_at_unix_secs < valid_windows.window_end_unix_secs
  GROUP BY
    valid_windows.id,
    valid_windows.window_scope,
    valid_windows.window_code,
    valid_windows.window_start_unix_secs,
    valid_windows.window_end_unix_secs
),
upserted_counters AS (
  INSERT INTO provider_api_key_window_usage_counters (
    provider_api_key_id,
    window_scope,
    window_code,
    window_start_unix_secs,
    window_end_unix_secs,
    request_count,
    total_tokens,
    total_cost_usd,
    rebuilt_at,
    updated_at
  )
  SELECT
    provider_api_key_id,
    window_scope,
    window_code,
    window_start_unix_secs,
    window_end_unix_secs,
    request_count,
    total_tokens,
    total_cost_usd,
    NOW(),
    NOW()
  FROM aggregated
  ON CONFLICT (
    provider_api_key_id,
    window_scope,
    window_code,
    window_start_unix_secs,
    window_end_unix_secs
  ) DO UPDATE SET
    request_count = EXCLUDED.request_count,
    total_tokens = EXCLUDED.total_tokens,
    total_cost_usd = EXCLUDED.total_cost_usd,
    rebuilt_at = NOW(),
    updated_at = NOW()
  RETURNING provider_api_key_id
),
deleted_stale_counters AS (
  DELETE FROM provider_api_key_window_usage_counters AS counters
  WHERE counters.provider_api_key_id = $1
    AND NOT EXISTS (
      SELECT 1
      FROM valid_windows
      WHERE valid_windows.id = counters.provider_api_key_id
        AND valid_windows.window_scope = counters.window_scope
        AND valid_windows.window_code = counters.window_code
        AND valid_windows.window_start_unix_secs = counters.window_start_unix_secs
        AND valid_windows.window_end_unix_secs = counters.window_end_unix_secs
    )
  RETURNING counters.provider_api_key_id
),
completed_deltas AS (
  UPDATE usage_counter_deltas
  SET processed_at = NOW()
  WHERE kind = 'provider_api_key_window'
    AND target_id = $1
    AND processed_at IS NULL
  RETURNING id
)
SELECT
  (SELECT COUNT(*)::BIGINT FROM upserted_counters) AS rebuilt_windows,
  (SELECT COUNT(*)::BIGINT FROM deleted_stale_counters) AS deleted_stale_windows,
  0::BIGINT AS recorded_applications,
  (SELECT COUNT(*)::BIGINT FROM completed_deltas) AS completed_deltas
