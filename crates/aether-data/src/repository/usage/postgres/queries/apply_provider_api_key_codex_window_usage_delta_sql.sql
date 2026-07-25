WITH input_rows AS (
  SELECT
    input.delta_id,
    input.provider_api_key_id,
    input.request_count_delta,
    input.total_tokens_delta,
    input.total_cost_usd_delta,
    input.usage_created_at_unix_secs
  FROM UNNEST(
    $1::TEXT[],
    $2::TEXT[],
    $3::BIGINT[],
    $4::BIGINT[],
    $5::DOUBLE PRECISION[],
    $6::BIGINT[]
  ) AS input(
    delta_id,
    provider_api_key_id,
    request_count_delta,
    total_tokens_delta,
    total_cost_usd_delta,
    usage_created_at_unix_secs
  )
),
target_keys AS (
  SELECT
    input_rows.*,
    keys.id IS NOT NULL AS key_exists,
    lower(BTRIM(COALESCE(providers.provider_type, ''))) = 'codex' AS is_codex,
    COALESCE(keys.status_snapshot::jsonb, '{}'::jsonb) AS status_snapshot
  FROM input_rows
  LEFT JOIN provider_api_keys AS keys
    ON keys.id = input_rows.provider_api_key_id
  LEFT JOIN providers
    ON providers.id = keys.provider_id
),
window_items AS (
  SELECT
    target_keys.delta_id,
    target_keys.provider_api_key_id,
    target_keys.request_count_delta,
    target_keys.total_tokens_delta,
    target_keys.total_cost_usd_delta,
    target_keys.usage_created_at_unix_secs,
    item.window_item
  FROM target_keys
  CROSS JOIN LATERAL jsonb_array_elements(
    CASE
      WHEN target_keys.is_codex
       AND jsonb_typeof(target_keys.status_snapshot #> '{quota,windows}') = 'array'
      THEN target_keys.status_snapshot #> '{quota,windows}'
      ELSE '[]'::jsonb
    END
  ) AS item(window_item)
),
parsed_windows AS (
  SELECT
    window_items.*,
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
         AND (
           length(BTRIM(window_item ->> 'window_seconds')) < 19
           OR BTRIM(window_item ->> 'window_seconds') <= '9223372036854775807'
         )
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
    parsed_windows.delta_id,
    parsed_windows.provider_api_key_id,
    parsed_windows.request_count_delta,
    parsed_windows.total_tokens_delta,
    parsed_windows.total_cost_usd_delta,
    parsed_windows.usage_created_at_unix_secs,
    parsed_windows.window_code,
    parsed_windows.window_scope,
    parsed_windows.reset_at - parsed_windows.window_seconds AS window_start_unix_secs,
    parsed_windows.reset_at AS window_end_unix_secs
  FROM parsed_windows
  WHERE parsed_windows.window_code <> ''
    AND parsed_windows.window_scope NOT IN ('feature', 'model', 'workspace')
    AND parsed_windows.reset_at IS NOT NULL
    AND parsed_windows.window_seconds IS NOT NULL
    AND parsed_windows.window_seconds > 0
    AND parsed_windows.reset_at >= parsed_windows.window_seconds
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
    ON resets.provider_api_key_id = natural_windows.provider_api_key_id
   AND resets.window_scope = natural_windows.window_scope
   AND resets.window_start_unix_secs = natural_windows.window_start_unix_secs
   AND resets.window_end_unix_secs = natural_windows.window_end_unix_secs
),
matching_windows AS (
  SELECT *
  FROM valid_windows
  WHERE usage_created_at_unix_secs >= usage_start_unix_secs
    AND usage_created_at_unix_secs < window_end_unix_secs
),
inserted_applications AS (
  INSERT INTO provider_api_key_window_usage_applications (
    delta_id,
    provider_api_key_id,
    window_scope,
    window_code,
    window_start_unix_secs,
    window_end_unix_secs
  )
  SELECT
    delta_id,
    provider_api_key_id,
    window_scope,
    window_code,
    window_start_unix_secs,
    window_end_unix_secs
  FROM matching_windows
  ON CONFLICT DO NOTHING
  RETURNING
    delta_id,
    provider_api_key_id,
    window_scope,
    window_code,
    window_start_unix_secs,
    window_end_unix_secs
),
window_increments AS (
  SELECT
    applications.provider_api_key_id,
    applications.window_scope,
    applications.window_code,
    applications.window_start_unix_secs,
    applications.window_end_unix_secs,
    SUM(input_rows.request_count_delta)::BIGINT AS request_count_delta,
    SUM(input_rows.total_tokens_delta)::BIGINT AS total_tokens_delta,
    SUM(input_rows.total_cost_usd_delta)::DOUBLE PRECISION AS total_cost_usd_delta
  FROM inserted_applications AS applications
  JOIN input_rows
    ON input_rows.delta_id = applications.delta_id
  GROUP BY
    applications.provider_api_key_id,
    applications.window_scope,
    applications.window_code,
    applications.window_start_unix_secs,
    applications.window_end_unix_secs
),
updated_counters AS (
  INSERT INTO provider_api_key_window_usage_counters (
    provider_api_key_id,
    window_scope,
    window_code,
    window_start_unix_secs,
    window_end_unix_secs,
    request_count,
    total_tokens,
    total_cost_usd,
    updated_at
  )
  SELECT
    provider_api_key_id,
    window_scope,
    window_code,
    window_start_unix_secs,
    window_end_unix_secs,
    GREATEST(request_count_delta, 0),
    GREATEST(total_tokens_delta, 0),
    GREATEST(CAST(total_cost_usd_delta AS NUMERIC), 0),
    NOW()
  FROM window_increments
  ON CONFLICT (
    provider_api_key_id,
    window_scope,
    window_code,
    window_start_unix_secs,
    window_end_unix_secs
  ) DO UPDATE SET
    request_count = GREATEST(
      provider_api_key_window_usage_counters.request_count + EXCLUDED.request_count,
      0
    ),
    total_tokens = GREATEST(
      provider_api_key_window_usage_counters.total_tokens + EXCLUDED.total_tokens,
      0
    ),
    total_cost_usd = GREATEST(
      provider_api_key_window_usage_counters.total_cost_usd + EXCLUDED.total_cost_usd,
      0
    ),
    updated_at = NOW()
  RETURNING provider_api_key_id
),
classifications AS (
  SELECT
    target_keys.delta_id,
    target_keys.key_exists,
    target_keys.is_codex,
    EXISTS (
      SELECT 1
      FROM valid_windows
      WHERE valid_windows.delta_id = target_keys.delta_id
    ) AS has_valid_windows,
    EXISTS (
      SELECT 1
      FROM valid_windows
      WHERE valid_windows.delta_id = target_keys.delta_id
        AND target_keys.usage_created_at_unix_secs >= valid_windows.window_end_unix_secs
    ) AS waiting_for_window_refresh
  FROM target_keys
)
SELECT
  classifications.delta_id,
  (
    NOT classifications.key_exists
    OR NOT classifications.is_codex
    OR NOT classifications.has_valid_windows
    OR (
      classifications.has_valid_windows
      AND NOT classifications.waiting_for_window_refresh
    )
  ) AS ready_to_complete,
  (SELECT COUNT(*)::BIGINT FROM updated_counters) AS updated_counter_rows
FROM classifications
