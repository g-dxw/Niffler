WITH requested AS (
  SELECT
    request_row.provider_api_key_id,
    request_row.window_code,
    request_row.start_unix_secs,
    request_row.end_unix_secs,
    request_row.ordinality
  FROM UNNEST(
    $1::TEXT[],
    $2::TEXT[],
    $3::BIGINT[],
    $4::BIGINT[]
  ) WITH ORDINALITY AS request_row(
    provider_api_key_id,
    window_code,
    start_unix_secs,
    end_unix_secs,
    ordinality
  )
)
SELECT
  requested.provider_api_key_id,
  requested.window_code,
  COALESCE(window_usage.request_count, 0)::BIGINT AS request_count,
  COALESCE(window_usage.total_tokens, 0)::BIGINT AS total_tokens,
  CAST(COALESCE(window_usage.total_cost_usd, 0) AS DOUBLE PRECISION) AS total_cost_usd
FROM requested
LEFT JOIN LATERAL (
  SELECT
    COUNT(*) FILTER (WHERE usage_facts.is_billable)::BIGINT AS request_count,
    COALESCE(SUM(usage_facts.total_tokens) FILTER (
      WHERE usage_facts.is_billable
    ), 0)::BIGINT AS total_tokens,
    CAST(COALESCE(SUM(usage_facts.window_cost_usd) FILTER (
      WHERE usage_facts.is_billable
    ), 0) AS DOUBLE PRECISION) AS total_cost_usd
  FROM (
    SELECT
      usage_row.id,
      usage_row.total_tokens,
      usage_row.window_cost_usd,
      usage_row.final_billing_status = 'settled'
        AND usage_row.window_cost_usd > 0 AS is_billable
    FROM (
      SELECT
        "usage".id,
        COALESCE(settlement.billing_status, "usage".billing_status) AS final_billing_status,
        GREATEST(
          COALESCE(
            CASE
              WHEN settlement.billing_input_tokens IS NOT NULL
                OR settlement.billing_output_tokens IS NOT NULL
                OR settlement.billing_cache_creation_tokens IS NOT NULL
                OR settlement.billing_cache_creation_5m_tokens IS NOT NULL
                OR settlement.billing_cache_creation_1h_tokens IS NOT NULL
                OR settlement.billing_cache_read_tokens IS NOT NULL
              THEN COALESCE(settlement.billing_input_tokens, 0)
                + COALESCE(settlement.billing_output_tokens, 0)
                + COALESCE(
                    settlement.billing_cache_creation_tokens,
                    COALESCE(settlement.billing_cache_creation_5m_tokens, 0)
                      + COALESCE(settlement.billing_cache_creation_1h_tokens, 0),
                    0
                  )
                + COALESCE(settlement.billing_cache_read_tokens, 0)
            END,
            "usage".total_tokens,
            0
          ),
          0
        )::BIGINT AS total_tokens,
        COALESCE(
          CASE
            WHEN BTRIM(COALESCE(settlement.settlement_snapshot ->> 'base_cost_usd', ''))
                 ~ '^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$'
            THEN CAST(settlement.settlement_snapshot ->> 'base_cost_usd' AS DOUBLE PRECISION)
            ELSE NULL
          END,
          CAST(settlement.billing_total_cost_usd AS DOUBLE PRECISION),
          CAST("usage".total_cost_usd AS DOUBLE PRECISION),
          0
        ) AS window_cost_usd
      FROM "usage"
      LEFT JOIN usage_settlement_snapshots AS settlement
        ON settlement.request_id = "usage".request_id
      WHERE "usage".provider_api_key_id = requested.provider_api_key_id
        AND "usage".created_at >= to_timestamp(requested.start_unix_secs::DOUBLE PRECISION)
        AND "usage".created_at < to_timestamp(requested.end_unix_secs::DOUBLE PRECISION)
    ) AS usage_row
  ) AS usage_facts
) AS window_usage ON TRUE
ORDER BY requested.ordinality ASC
