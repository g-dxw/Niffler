WITH aggregated AS (
  SELECT
    provider_api_key_id,
    COUNT(*)::BIGINT AS request_count,
    COALESCE(SUM(
      CASE
        WHEN status IN ('completed', 'success', 'ok', 'billed', 'settled')
             AND (status_code IS NULL OR status_code < 400)
             AND NULLIF(BTRIM(error_message), '') IS NULL
        THEN 1
        ELSE 0
      END
    ), 0)::BIGINT AS success_count,
    COALESCE(SUM(
      CASE
        WHEN status NOT IN ('pending', 'streaming')
             AND NOT (
               status IN ('completed', 'success', 'ok', 'billed', 'settled')
               AND (status_code IS NULL OR status_code < 400)
               AND NULLIF(BTRIM(error_message), '') IS NULL
             )
        THEN 1
        ELSE 0
      END
    ), 0)::BIGINT AS error_count,
    COALESCE(SUM(
      GREATEST(
        COALESCE(
          total_tokens,
          COALESCE(input_tokens, 0) + COALESCE(output_tokens, 0)
        ),
        0
      )::BIGINT
    ), 0)::BIGINT AS total_tokens,
    COALESCE(SUM(
      CASE
        WHEN "usage".billing_status = 'settled'
        THEN provider_cost.base_cost_usd
        ELSE 0
      END
    ), 0)::NUMERIC(20,8) AS total_cost_usd,
    COALESCE(SUM(
      CASE
        WHEN status IN ('completed', 'success', 'ok', 'billed', 'settled')
             AND (status_code IS NULL OR status_code < 400)
             AND NULLIF(BTRIM(error_message), '') IS NULL
             AND response_time_ms IS NOT NULL
        THEN GREATEST(response_time_ms, 0)
        ELSE 0
      END
    ), 0)::BIGINT AS total_response_time_ms,
    MAX(created_at) AS last_used_at
  FROM usage_billing_facts AS "usage"
  LEFT JOIN "usage" AS raw_usage
    ON raw_usage.request_id = "usage".request_id
  LEFT JOIN usage_settlement_snapshots AS settlement
    ON settlement.request_id = "usage".request_id
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
          THEN CAST("usage".total_cost_usd AS DOUBLE PRECISION) / provider_cost_inputs.sales_multiplier
          ELSE NULL
        END,
        CAST("usage".total_cost_usd AS DOUBLE PRECISION),
        0
      ),
      0
    ) AS base_cost_usd
  ) AS provider_cost ON TRUE
  WHERE provider_api_key_id IS NOT NULL
    AND BTRIM(provider_api_key_id) <> ''
    AND status NOT IN ('pending', 'streaming')
  GROUP BY provider_api_key_id
)
UPDATE provider_api_keys
SET
  request_count = aggregated.request_count,
  success_count = aggregated.success_count,
  error_count = aggregated.error_count,
  total_tokens = aggregated.total_tokens,
  total_cost_usd = aggregated.total_cost_usd,
  total_response_time_ms = aggregated.total_response_time_ms,
  last_used_at = aggregated.last_used_at
FROM aggregated
WHERE provider_api_keys.id = aggregated.provider_api_key_id
