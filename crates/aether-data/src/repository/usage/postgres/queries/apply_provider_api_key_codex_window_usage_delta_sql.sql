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
    parsed.scope,
    parsed.reset_at,
    parsed.window_seconds,
    parsed.window_minutes,
    parsed.usage_reset_at,
    COALESCE(
      parsed.window_seconds,
      CASE
        WHEN parsed.window_minutes BETWEEN 0 AND 153722867280912930
        THEN parsed.window_minutes * 60
        ELSE NULL
      END,
      CASE parsed.code
        WHEN '5h' THEN 18000
        WHEN '7d' THEN 604800
        WHEN 'weekly' THEN 604800
        WHEN '1m' THEN 2592000
        WHEN 'monthly' THEN 2592000
        ELSE NULL
      END
    ) AS resolved_window_seconds
  FROM target
  CROSS JOIN LATERAL (
    SELECT
      item.window_item,
      item.ordinality,
      lower(BTRIM(COALESCE(item.window_item ->> 'code', ''))) AS code,
      lower(BTRIM(COALESCE(item.window_item ->> 'scope', 'account'))) AS scope,
      CASE
        WHEN BTRIM(COALESCE(item.window_item ->> 'reset_at', '')) ~ '^[0-9]+$'
          AND (
            length(BTRIM(item.window_item ->> 'reset_at')) < 19
            OR (
              length(BTRIM(item.window_item ->> 'reset_at')) = 19
              AND BTRIM(item.window_item ->> 'reset_at') <= '9223372036854775807'
            )
          )
        THEN CAST(item.window_item ->> 'reset_at' AS BIGINT)
        ELSE NULL
      END AS reset_at,
      CASE
        WHEN BTRIM(COALESCE(item.window_item ->> 'window_seconds', '')) ~ '^[0-9]+$'
          AND (
            length(BTRIM(item.window_item ->> 'window_seconds')) < 19
            OR (
              length(BTRIM(item.window_item ->> 'window_seconds')) = 19
              AND BTRIM(item.window_item ->> 'window_seconds') <= '9223372036854775807'
            )
          )
        THEN CAST(item.window_item ->> 'window_seconds' AS BIGINT)
        ELSE NULL
      END AS window_seconds,
      CASE
        WHEN BTRIM(COALESCE(item.window_item ->> 'window_minutes', '')) ~ '^[0-9]+$'
          AND (
            length(BTRIM(item.window_item ->> 'window_minutes')) < 19
            OR (
              length(BTRIM(item.window_item ->> 'window_minutes')) = 19
              AND BTRIM(item.window_item ->> 'window_minutes') <= '9223372036854775807'
            )
          )
        THEN CAST(item.window_item ->> 'window_minutes' AS BIGINT)
        ELSE NULL
      END AS window_minutes,
      CASE
        WHEN BTRIM(COALESCE(item.window_item ->> 'usage_reset_at', '')) ~ '^[0-9]+$'
          AND (
            length(BTRIM(item.window_item ->> 'usage_reset_at')) < 19
            OR (
              length(BTRIM(item.window_item ->> 'usage_reset_at')) = 19
              AND BTRIM(item.window_item ->> 'usage_reset_at') <= '9223372036854775807'
            )
          )
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
        WHEN scope NOT IN ('feature', 'model', 'workspace')
          AND reset_at IS NOT NULL
          AND resolved_window_seconds IS NOT NULL
          AND reset_at >= resolved_window_seconds
          AND $5::BIGINT >= GREATEST(reset_at - resolved_window_seconds, COALESCE(usage_reset_at, 0))
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
