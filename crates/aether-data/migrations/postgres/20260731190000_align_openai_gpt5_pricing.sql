BEGIN;

UPDATE public.global_models
SET enabled = FALSE,
    is_active = FALSE,
    updated_at = NOW()
WHERE name IN ('gpt-5.5-pro', 'gpt-5.4-pro');

UPDATE public.models AS m
SET enabled = FALSE,
    is_active = FALSE,
    is_available = FALSE,
    updated_at = NOW()
WHERE m.provider_model_name IN ('gpt-5.5-pro', 'gpt-5.4-pro')
   OR m.global_model_name IN ('gpt-5.5-pro', 'gpt-5.4-pro');

UPDATE public.global_models
SET default_tiered_pricing = CASE name
        WHEN 'gpt-5.4' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":2.5,"output_price_per_1m":15.0,"cache_creation_price_per_1m":0.0,"cache_read_price_per_1m":0.25},
            {"up_to":null,"input_price_per_1m":5.0,"output_price_per_1m":30.0,"cache_creation_price_per_1m":0.0,"cache_read_price_per_1m":0.5}
          ]}
        $json$::json
        WHEN 'gpt-5.4-mini' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":0.75,"output_price_per_1m":4.5,"cache_creation_price_per_1m":0.0,"cache_read_price_per_1m":0.075},
            {"up_to":null,"input_price_per_1m":1.5,"output_price_per_1m":9.0,"cache_creation_price_per_1m":0.0,"cache_read_price_per_1m":0.15}
          ]}
        $json$::json
        WHEN 'gpt-5.5' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":5.0,"output_price_per_1m":30.0,"cache_creation_price_per_1m":0.0,"cache_read_price_per_1m":0.5},
            {"up_to":null,"input_price_per_1m":12.5,"output_price_per_1m":75.0,"cache_creation_price_per_1m":0.0,"cache_read_price_per_1m":1.25}
          ]}
        $json$::json
        WHEN 'gpt-5.6' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":5.0,"output_price_per_1m":30.0,"cache_creation_price_per_1m":6.25,"cache_read_price_per_1m":0.5},
            {"up_to":null,"input_price_per_1m":10.0,"output_price_per_1m":60.0,"cache_creation_price_per_1m":12.5,"cache_read_price_per_1m":1.0}
          ]}
        $json$::json
        WHEN 'gpt-5.6-sol' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":5.0,"output_price_per_1m":30.0,"cache_creation_price_per_1m":6.25,"cache_read_price_per_1m":0.5},
            {"up_to":null,"input_price_per_1m":10.0,"output_price_per_1m":60.0,"cache_creation_price_per_1m":12.5,"cache_read_price_per_1m":1.0}
          ]}
        $json$::json
        WHEN 'gpt-5.6-terra' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":2.0,"output_price_per_1m":12.0,"cache_creation_price_per_1m":2.5,"cache_read_price_per_1m":0.2},
            {"up_to":null,"input_price_per_1m":4.0,"output_price_per_1m":24.0,"cache_creation_price_per_1m":5.0,"cache_read_price_per_1m":0.4}
          ]}
        $json$::json
        WHEN 'gpt-5.6-luna' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":0.2,"output_price_per_1m":1.2,"cache_creation_price_per_1m":0.25,"cache_read_price_per_1m":0.02},
            {"up_to":null,"input_price_per_1m":0.4,"output_price_per_1m":2.4,"cache_creation_price_per_1m":0.5,"cache_read_price_per_1m":0.04}
          ]}
        $json$::json
        ELSE default_tiered_pricing
    END,
    config = jsonb_set(
        jsonb_set(
            COALESCE(config, '{}'::jsonb) - 'long_context',
            '{pricing_source}',
            '"openai_official_api_pricing"'::jsonb
        ),
        '{pricing_note}',
        '"Exact official OpenAI API pricing with explicit short- and long-context tiers."'::jsonb
    ),
    updated_at = NOW()
WHERE name IN (
    'gpt-5.4',
    'gpt-5.4-mini',
    'gpt-5.5',
    'gpt-5.6',
    'gpt-5.6-sol',
    'gpt-5.6-terra',
    'gpt-5.6-luna'
);

UPDATE public.models AS m
SET tiered_pricing = CASE m.provider_model_name
        WHEN 'gpt-5.4' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":2.5,"output_price_per_1m":15.0,"cache_creation_price_per_1m":0.0,"cache_read_price_per_1m":0.25},
            {"up_to":null,"input_price_per_1m":5.0,"output_price_per_1m":30.0,"cache_creation_price_per_1m":0.0,"cache_read_price_per_1m":0.5}
          ]}
        $json$::json
        WHEN 'gpt-5.4-mini' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":0.75,"output_price_per_1m":4.5,"cache_creation_price_per_1m":0.0,"cache_read_price_per_1m":0.075},
            {"up_to":null,"input_price_per_1m":1.5,"output_price_per_1m":9.0,"cache_creation_price_per_1m":0.0,"cache_read_price_per_1m":0.15}
          ]}
        $json$::json
        WHEN 'gpt-5.5' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":5.0,"output_price_per_1m":30.0,"cache_creation_price_per_1m":0.0,"cache_read_price_per_1m":0.5},
            {"up_to":null,"input_price_per_1m":12.5,"output_price_per_1m":75.0,"cache_creation_price_per_1m":0.0,"cache_read_price_per_1m":1.25}
          ]}
        $json$::json
        WHEN 'gpt-5.6' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":5.0,"output_price_per_1m":30.0,"cache_creation_price_per_1m":6.25,"cache_read_price_per_1m":0.5},
            {"up_to":null,"input_price_per_1m":10.0,"output_price_per_1m":60.0,"cache_creation_price_per_1m":12.5,"cache_read_price_per_1m":1.0}
          ]}
        $json$::json
        WHEN 'gpt-5.6-sol' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":5.0,"output_price_per_1m":30.0,"cache_creation_price_per_1m":6.25,"cache_read_price_per_1m":0.5},
            {"up_to":null,"input_price_per_1m":10.0,"output_price_per_1m":60.0,"cache_creation_price_per_1m":12.5,"cache_read_price_per_1m":1.0}
          ]}
        $json$::json
        WHEN 'gpt-5.6-terra' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":2.0,"output_price_per_1m":12.0,"cache_creation_price_per_1m":2.5,"cache_read_price_per_1m":0.2},
            {"up_to":null,"input_price_per_1m":4.0,"output_price_per_1m":24.0,"cache_creation_price_per_1m":5.0,"cache_read_price_per_1m":0.4}
          ]}
        $json$::json
        WHEN 'gpt-5.6-luna' THEN $json$
          {"tiers":[
            {"up_to":272000,"input_price_per_1m":0.2,"output_price_per_1m":1.2,"cache_creation_price_per_1m":0.25,"cache_read_price_per_1m":0.02},
            {"up_to":null,"input_price_per_1m":0.4,"output_price_per_1m":2.4,"cache_creation_price_per_1m":0.5,"cache_read_price_per_1m":0.04}
          ]}
        $json$::json
        ELSE tiered_pricing
    END,
    updated_at = NOW()
FROM public.providers AS p
WHERE p.id = m.provider_id
  AND p.provider_type = 'codex'
  AND p.name IN ('Plus号池', 'Pro号池')
  AND m.provider_model_name IN (
      'gpt-5.4',
      'gpt-5.4-mini',
      'gpt-5.5',
      'gpt-5.6',
      'gpt-5.6-sol',
      'gpt-5.6-terra',
      'gpt-5.6-luna'
  );

-- Keep the shadow price table append-only: effective_from_unix_ms identifies a price version.
INSERT INTO public.niffler_model_base_prices (
    id,
    model_name,
    input_price_per_million,
    output_price_per_million,
    cache_write_price_per_million,
    cache_read_price_per_million,
    source,
    effective_from_unix_ms,
    created_at_unix_ms,
    updated_at_unix_ms
)
SELECT
    md5('niffler-model-base-price:' || price.model_name || ':1785524400000'),
    price.model_name,
    price.input_price_per_million,
    price.output_price_per_million,
    price.cache_write_price_per_million,
    price.cache_read_price_per_million,
    'openai_official_api_pricing',
    1785524400000,
    1785524400000,
    1785524400000
FROM (
    VALUES
        ('gpt-5.6', 5.0, 30.0, 6.25, 0.5),
        ('gpt-5.6-sol', 5.0, 30.0, 6.25, 0.5),
        ('gpt-5.6-terra', 2.0, 12.0, 2.5, 0.2),
        ('gpt-5.6-luna', 0.2, 1.2, 0.25, 0.02)
) AS price(
    model_name,
    input_price_per_million,
    output_price_per_million,
    cache_write_price_per_million,
    cache_read_price_per_million
)
WHERE NOT EXISTS (
    SELECT 1
    FROM public.niffler_model_base_prices AS existing
    WHERE existing.model_name = price.model_name
      AND existing.effective_from_unix_ms = 1785524400000
);

COMMIT;
