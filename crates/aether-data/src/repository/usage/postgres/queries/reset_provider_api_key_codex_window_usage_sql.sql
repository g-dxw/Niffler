WITH input_windows AS (
  SELECT
    input.provider_api_key_id,
    lower(BTRIM(input.window_scope)) AS window_scope,
    input.window_start_unix_secs,
    input.window_end_unix_secs
  FROM UNNEST(
    $1::TEXT[],
    $2::TEXT[],
    $3::BIGINT[],
    $4::BIGINT[]
  ) AS input(
    provider_api_key_id,
    window_scope,
    window_start_unix_secs,
    window_end_unix_secs
  )
),
valid_windows AS (
  SELECT DISTINCT
    provider_api_key_id,
    CASE
      WHEN window_scope = '' THEN 'account'
      ELSE window_scope
    END AS window_scope,
    window_start_unix_secs,
    window_end_unix_secs
  FROM input_windows
  WHERE provider_api_key_id <> ''
    AND window_start_unix_secs < window_end_unix_secs
    AND $5::BIGINT >= window_start_unix_secs
    AND $5::BIGINT < window_end_unix_secs
),
upserted_resets AS (
  INSERT INTO provider_api_key_window_usage_resets (
    provider_api_key_id,
    window_scope,
    window_start_unix_secs,
    window_end_unix_secs,
    usage_reset_at_unix_secs,
    updated_at
  )
  SELECT
    provider_api_key_id,
    window_scope,
    window_start_unix_secs,
    window_end_unix_secs,
    $5::BIGINT,
    NOW()
  FROM valid_windows
  ON CONFLICT (
    provider_api_key_id,
    window_scope,
    window_start_unix_secs,
    window_end_unix_secs
  ) DO UPDATE SET
    usage_reset_at_unix_secs = EXCLUDED.usage_reset_at_unix_secs,
    updated_at = NOW()
  RETURNING provider_api_key_id
)
SELECT COUNT(*)::BIGINT AS reset_windows
FROM upserted_resets
