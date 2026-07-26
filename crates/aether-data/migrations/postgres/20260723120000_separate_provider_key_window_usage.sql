ALTER TABLE IF EXISTS public.usage_counter_deltas
    ADD COLUMN IF NOT EXISTS available_at timestamp with time zone
    NOT NULL DEFAULT '-infinity'::timestamp with time zone;

CREATE TABLE IF NOT EXISTS public.provider_api_key_window_usage_counters (
    provider_api_key_id text NOT NULL,
    window_scope text NOT NULL DEFAULT 'account',
    window_code text NOT NULL,
    window_start_unix_secs bigint NOT NULL,
    window_end_unix_secs bigint NOT NULL,
    request_count bigint NOT NULL DEFAULT 0,
    total_tokens bigint NOT NULL DEFAULT 0,
    total_cost_usd numeric(20,8) NOT NULL DEFAULT 0,
    rebuilt_at timestamp with time zone,
    updated_at timestamp with time zone NOT NULL DEFAULT now(),
    CONSTRAINT provider_api_key_window_usage_counters_pkey PRIMARY KEY (
        provider_api_key_id,
        window_scope,
        window_code,
        window_start_unix_secs,
        window_end_unix_secs
    ),
    CONSTRAINT provider_api_key_window_usage_counters_key_fkey
        FOREIGN KEY (provider_api_key_id)
        REFERENCES public.provider_api_keys(id)
        ON DELETE CASCADE,
    CONSTRAINT provider_api_key_window_usage_counters_range_check
        CHECK (window_start_unix_secs < window_end_unix_secs),
    CONSTRAINT provider_api_key_window_usage_counters_nonnegative_check
        CHECK (request_count >= 0 AND total_tokens >= 0 AND total_cost_usd >= 0)
);

CREATE TABLE IF NOT EXISTS public.provider_api_key_window_usage_resets (
    provider_api_key_id text NOT NULL,
    window_scope text NOT NULL DEFAULT 'account',
    window_start_unix_secs bigint NOT NULL,
    window_end_unix_secs bigint NOT NULL,
    usage_reset_at_unix_secs bigint NOT NULL,
    updated_at timestamp with time zone NOT NULL DEFAULT now(),
    CONSTRAINT provider_api_key_window_usage_resets_pkey PRIMARY KEY (
        provider_api_key_id,
        window_scope,
        window_start_unix_secs,
        window_end_unix_secs
    ),
    CONSTRAINT provider_api_key_window_usage_resets_key_fkey
        FOREIGN KEY (provider_api_key_id)
        REFERENCES public.provider_api_keys(id)
        ON DELETE CASCADE,
    CONSTRAINT provider_api_key_window_usage_resets_range_check
        CHECK (
            window_start_unix_secs < window_end_unix_secs
            AND usage_reset_at_unix_secs >= window_start_unix_secs
            AND usage_reset_at_unix_secs < window_end_unix_secs
        )
);

CREATE TABLE IF NOT EXISTS public.provider_api_key_window_usage_applications (
    delta_id character varying(36) NOT NULL,
    provider_api_key_id text NOT NULL,
    window_scope text NOT NULL,
    window_code text NOT NULL,
    window_start_unix_secs bigint NOT NULL,
    window_end_unix_secs bigint NOT NULL,
    applied_at timestamp with time zone NOT NULL DEFAULT now(),
    CONSTRAINT provider_api_key_window_usage_applications_pkey PRIMARY KEY (
        delta_id,
        provider_api_key_id,
        window_scope,
        window_code,
        window_start_unix_secs,
        window_end_unix_secs
    ),
    CONSTRAINT provider_api_key_window_usage_applications_delta_fkey
        FOREIGN KEY (delta_id)
        REFERENCES public.usage_counter_deltas(id)
        ON DELETE CASCADE,
    CONSTRAINT provider_api_key_window_usage_applications_key_fkey
        FOREIGN KEY (provider_api_key_id)
        REFERENCES public.provider_api_keys(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_provider_api_key_window_usage_applications_key
    ON public.provider_api_key_window_usage_applications (
        provider_api_key_id,
        window_code,
        window_end_unix_secs
    );

ALTER TABLE IF EXISTS public.usage_counter_deltas
    DROP CONSTRAINT IF EXISTS usage_counter_deltas_kind_check;

ALTER TABLE IF EXISTS public.usage_counter_deltas
    ADD CONSTRAINT usage_counter_deltas_kind_check CHECK (
        kind IN (
            'api_key',
            'provider_api_key',
            'provider_api_key_window',
            'model',
            'provider_monthly',
            'proxy_node',
            'management_token',
            'api_key_last_used'
        )
    );
