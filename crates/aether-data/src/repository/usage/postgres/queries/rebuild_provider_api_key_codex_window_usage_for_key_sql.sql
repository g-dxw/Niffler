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
    COUNT(usage_facts.id) FILTER (
      WHERE usage_facts.billing_status = 'settled'
    )::BIGINT AS request_count,
    COALESCE(SUM(GREATEST(COALESCE(usage_facts.total_tokens, 0), 0)::BIGINT) FILTER (
      WHERE usage_facts.billing_status = 'settled'
    ), 0)::BIGINT AS total_tokens,
    CAST(COALESCE(SUM(provider_cost.base_cost_usd) FILTER (
      WHERE usage_facts.billing_status = 'settled'
    ), 0) AS DOUBLE PRECISION) AS total_cost_usd
  FROM valid_windows
  LEFT JOIN usage_billing_facts AS usage_facts
    ON usage_facts.provider_api_key_id = valid_windows.id
   AND usage_facts.created_at >= to_timestamp(valid_windows.usage_start_unix_secs::DOUBLE PRECISION)
   AND usage_facts.created_at < to_timestamp(valid_windows.window_end_unix_secs::DOUBLE PRECISION)
  LEFT JOIN public."usage" AS raw_usage
    ON raw_usage.request_id = usage_facts.request_id
  LEFT JOIN usage_settlement_snapshots AS settlement
    ON settlement.request_id = usage_facts.request_id
  LEFT JOIN LATERAL (
    SELECT
      COALESCE(
        CASE
          WHEN BTRIM(COALESCE(raw_usage.request_metadata ->> 'base_cost_usd', ''))
               ~ '^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$'
          THEN CAST(raw_usage.request_metadata ->> 'base_cost_usd' AS DOUBLE PRECISION)
          ELSE NULL
        END,
        CASE
          WHEN BTRIM(COALESCE(raw_usage.request_metadata #>> '{settlement_snapshot,base_cost_usd}', ''))
               ~ '^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$'
          THEN CAST(raw_usage.request_metadata #>> '{settlement_snapshot,base_cost_usd}' AS DOUBLE PRECISION)
          ELSE NULL
        END,
        CASE
          WHEN BTRIM(COALESCE(settlement.settlement_snapshot ->> 'base_cost_usd', ''))
               ~ '^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$'
          THEN CAST(settlement.settlement_snapshot ->> 'base_cost_usd' AS DOUBLE PRECISION)
          ELSE NULL
        END
      ) AS direct_base_cost_usd,
      COALESCE(
        CASE
          WHEN BTRIM(COALESCE(raw_usage.request_metadata ->> 'sales_multiplier', ''))
               ~ '^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$'
          THEN CAST(raw_usage.request_metadata ->> 'sales_multiplier' AS DOUBLE PRECISION)
          ELSE NULL
        END,
        CASE
          WHEN BTRIM(COALESCE(raw_usage.request_metadata #>> '{settlement_snapshot,pricing_snapshot,sales_multiplier}', ''))
               ~ '^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$'
          THEN CAST(raw_usage.request_metadata #>> '{settlement_snapshot,pricing_snapshot,sales_multiplier}' AS DOUBLE PRECISION)
          ELSE NULL
        END,
        CASE
          WHEN BTRIM(COALESCE(settlement.settlement_snapshot -> 'pricing_snapshot' ->> 'sales_multiplier', ''))
               ~ '^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$'
          THEN CAST(settlement.settlement_snapshot -> 'pricing_snapshot' ->> 'sales_multiplier' AS DOUBLE PRECISION)
          ELSE NULL
        END
      ) AS sales_multiplier
  ) AS provider_cost_inputs ON TRUE
  LEFT JOIN LATERAL (
    SELECT GREATEST(
      COALESCE(
        provider_cost_inputs.direct_base_cost_usd,
        CASE
          WHEN provider_cost_inputs.sales_multiplier > 0
          THEN CAST(usage_facts.total_cost_usd AS DOUBLE PRECISION)
            / provider_cost_inputs.sales_multiplier
          ELSE NULL
        END,
        CAST(usage_facts.total_cost_usd AS DOUBLE PRECISION),
        0
      ),
      0
    ) AS base_cost_usd
  ) AS provider_cost ON TRUE
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
    COALESCE(request_count, 0),
    COALESCE(total_tokens, 0),
    GREATEST(CAST(COALESCE(total_cost_usd, 0) AS NUMERIC), 0),
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
pending_deltas AS (
  SELECT
    delta.id AS delta_id,
    delta.target_id AS provider_api_key_id,
    COALESCE(delta.usage_created_at_unix_secs, 0) AS usage_created_at_unix_secs
  FROM usage_counter_deltas AS delta
  WHERE delta.kind = 'provider_api_key_window'
    AND delta.target_id = $1
    AND delta.processed_at IS NULL
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
    pending_deltas.delta_id,
    pending_deltas.provider_api_key_id,
    valid_windows.window_scope,
    valid_windows.window_code,
    valid_windows.window_start_unix_secs,
    valid_windows.window_end_unix_secs
  FROM pending_deltas
  JOIN valid_windows
    ON valid_windows.id = pending_deltas.provider_api_key_id
   AND pending_deltas.usage_created_at_unix_secs >= valid_windows.usage_start_unix_secs
   AND pending_deltas.usage_created_at_unix_secs < valid_windows.window_end_unix_secs
  ON CONFLICT DO NOTHING
  RETURNING delta_id
),
completed_deltas AS (
  UPDATE usage_counter_deltas AS delta
  SET processed_at = NOW()
  FROM pending_deltas
  WHERE delta.id = pending_deltas.delta_id
    AND EXISTS (
      SELECT 1
      FROM valid_windows
      WHERE valid_windows.id = pending_deltas.provider_api_key_id
    )
    AND NOT EXISTS (
      SELECT 1
      FROM valid_windows
      WHERE valid_windows.id = pending_deltas.provider_api_key_id
        AND pending_deltas.usage_created_at_unix_secs >= valid_windows.window_end_unix_secs
    )
  RETURNING delta.id
)
SELECT
  (SELECT COUNT(*)::BIGINT FROM upserted_counters) AS rebuilt_windows,
  (SELECT COUNT(*)::BIGINT FROM deleted_stale_counters) AS deleted_stale_windows,
  (SELECT COUNT(*)::BIGINT FROM inserted_applications) AS recorded_applications,
  (SELECT COUNT(*)::BIGINT FROM completed_deltas) AS completed_deltas
