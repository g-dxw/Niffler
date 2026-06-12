WITH target AS (
  SELECT
    id,
    COALESCE(status_snapshot::jsonb, '{}'::jsonb) AS status_snapshot
  FROM provider_api_keys
  WHERE id = $1
    AND $5::BIGINT IS NOT NULL
    AND lower(BTRIM(COALESCE((status_snapshot::jsonb) #>> '{quota,provider_type}', ''))) = 'codex'
    AND jsonb_typeof((status_snapshot::jsonb) #> '{quota,windows}') = 'array'
),
windows AS (
  SELECT
    target.id,
    parsed.ordinality,
    parsed.window_item,
    parsed.code,
    parsed.reset_at,
    parsed.window_minutes,
    parsed.usage_reset_at,
    COALESCE(parsed.window_minutes, CASE parsed.code
      WHEN '5h' THEN 300
      WHEN 'weekly' THEN 10080
      ELSE NULL
    END) AS resolved_window_minutes
  FROM target
  CROSS JOIN LATERAL (
    SELECT
      item.window_item,
      item.ordinality,
      lower(BTRIM(COALESCE(item.window_item ->> 'code', ''))) AS code,
      CASE
        WHEN BTRIM(COALESCE(item.window_item ->> 'reset_at', '')) ~ '^[0-9]+$'
        THEN CAST(item.window_item ->> 'reset_at' AS BIGINT)
        ELSE NULL
      END AS reset_at,
      CASE
        WHEN BTRIM(COALESCE(item.window_item ->> 'window_minutes', '')) ~ '^[0-9]+$'
        THEN CAST(item.window_item ->> 'window_minutes' AS BIGINT)
        ELSE NULL
      END AS window_minutes,
      CASE
        WHEN BTRIM(COALESCE(item.window_item ->> 'usage_reset_at', '')) ~ '^[0-9]+$'
        THEN CAST(item.window_item ->> 'usage_reset_at' AS BIGINT)
        ELSE NULL
      END AS usage_reset_at
    FROM jsonb_array_elements(target.status_snapshot #> '{quota,windows}')
      WITH ORDINALITY AS item(window_item, ordinality)
  ) AS parsed
),
updated AS (
  SELECT
    id,
    jsonb_agg(
      CASE
        WHEN code IN ('5h', 'weekly')
          AND reset_at IS NOT NULL
          AND resolved_window_minutes IS NOT NULL
          AND reset_at >= resolved_window_minutes * 60
          AND $5::BIGINT >= GREATEST(reset_at - resolved_window_minutes * 60, COALESCE(usage_reset_at, 0))
          AND $5::BIGINT < reset_at
        THEN jsonb_set(
          window_item,
          '{usage}',
          jsonb_build_object(
            'request_count',
            GREATEST(
              COALESCE(
                CASE
                  WHEN BTRIM(COALESCE(window_item #>> '{usage,request_count}', '')) ~ '^[+-]?[0-9]+$'
                  THEN CAST(window_item #>> '{usage,request_count}' AS BIGINT)
                  ELSE NULL
                END,
                0
              ) + $2::BIGINT,
              0
            ),
            'total_tokens',
            GREATEST(
              COALESCE(
                CASE
                  WHEN BTRIM(COALESCE(window_item #>> '{usage,total_tokens}', '')) ~ '^[+-]?[0-9]+$'
                  THEN CAST(window_item #>> '{usage,total_tokens}' AS BIGINT)
                  ELSE NULL
                END,
                0
              ) + $3::BIGINT,
              0
            ),
            'total_cost_usd',
            TO_CHAR(
              GREATEST(
                COALESCE(
                  CASE
                    WHEN BTRIM(COALESCE(window_item #>> '{usage,total_cost_usd}', ''))
                         ~ '^[+-]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][+-]?[0-9]+)?$'
                    THEN CAST(window_item #>> '{usage,total_cost_usd}' AS NUMERIC)
                    ELSE NULL
                  END,
                  0
                ) + CAST($4::DOUBLE PRECISION AS NUMERIC),
                0
              ),
              'FM999999999999999990.00000000'
            )
          ),
          TRUE
        )
        ELSE window_item
      END
      ORDER BY ordinality
    ) AS windows
  FROM windows
  GROUP BY id
)
UPDATE provider_api_keys AS keys
SET
  status_snapshot = jsonb_set(
    COALESCE(keys.status_snapshot::jsonb, '{}'::jsonb),
    '{quota,windows}',
    updated.windows,
    FALSE
  )::json,
  updated_at = NOW()
FROM updated
WHERE keys.id = updated.id
