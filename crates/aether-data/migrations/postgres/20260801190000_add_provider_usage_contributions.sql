CREATE TABLE IF NOT EXISTS public.provider_api_key_usage_contributions (
    request_id character varying(128) NOT NULL,
    provider_api_key_id character varying(64),
    request_count bigint NOT NULL DEFAULT 0,
    success_count bigint NOT NULL DEFAULT 0,
    error_count bigint NOT NULL DEFAULT 0,
    total_tokens bigint NOT NULL DEFAULT 0,
    total_cost_usd numeric(20,8) NOT NULL DEFAULT 0,
    total_response_time_ms bigint NOT NULL DEFAULT 0,
    last_used_at_unix_secs bigint,
    usage_created_at_unix_secs bigint,
    window_request_count bigint NOT NULL DEFAULT 0,
    window_total_tokens bigint NOT NULL DEFAULT 0,
    window_total_cost_usd numeric(20,8) NOT NULL DEFAULT 0,
    revision bigint NOT NULL DEFAULT 0,
    updated_at timestamp with time zone NOT NULL DEFAULT now(),
    CONSTRAINT provider_api_key_usage_contributions_pkey PRIMARY KEY (request_id),
    CONSTRAINT provider_api_key_usage_contributions_key_fkey
        FOREIGN KEY (provider_api_key_id)
        REFERENCES public.provider_api_keys(id)
        ON DELETE CASCADE,
    CONSTRAINT provider_api_key_usage_contributions_nonnegative_check CHECK (
        request_count >= 0
        AND success_count >= 0
        AND error_count >= 0
        AND total_tokens >= 0
        AND total_cost_usd >= 0
        AND total_response_time_ms >= 0
        AND window_request_count >= 0
        AND window_total_tokens >= 0
        AND window_total_cost_usd >= 0
        AND revision >= 0
    )
);

CREATE INDEX IF NOT EXISTS ix_provider_api_key_usage_contributions_key
    ON public.provider_api_key_usage_contributions (provider_api_key_id, usage_created_at_unix_secs);

CREATE TABLE IF NOT EXISTS public.provider_api_key_usage_contribution_backfill_state (
    id smallint NOT NULL DEFAULT 1,
    high_water_created_at timestamp with time zone,
    initialized_at timestamp with time zone,
    completed_at timestamp with time zone,
    updated_at timestamp with time zone NOT NULL DEFAULT now(),
    CONSTRAINT provider_api_key_usage_contribution_backfill_state_pkey PRIMARY KEY (id),
    CONSTRAINT provider_api_key_usage_contribution_backfill_state_singleton_check CHECK (id = 1)
);

INSERT INTO public.provider_api_key_usage_contribution_backfill_state (id)
VALUES (1)
ON CONFLICT (id) DO NOTHING;

CREATE TABLE IF NOT EXISTS public.provider_api_key_usage_contribution_backfills (
    provider_api_key_id character varying(64) NOT NULL,
    high_water_created_at timestamp with time zone,
    cursor_created_at timestamp with time zone,
    cursor_request_id character varying(128),
    status character varying(16) NOT NULL DEFAULT 'pending',
    backfilled_at timestamp with time zone,
    updated_at timestamp with time zone NOT NULL DEFAULT now(),
    CONSTRAINT provider_api_key_usage_contribution_backfills_pkey PRIMARY KEY (provider_api_key_id),
    CONSTRAINT provider_api_key_usage_contribution_backfills_key_fkey
        FOREIGN KEY (provider_api_key_id)
        REFERENCES public.provider_api_keys(id)
        ON DELETE CASCADE,
    CONSTRAINT provider_api_key_usage_contribution_backfills_status_check
        CHECK (status IN ('pending', 'running', 'completed'))
);

INSERT INTO public.provider_api_key_usage_contribution_backfills (
    provider_api_key_id,
    high_water_created_at
)
SELECT id, clock_timestamp()
FROM public.provider_api_keys
ON CONFLICT (provider_api_key_id) DO NOTHING;

CREATE INDEX IF NOT EXISTS ix_provider_api_key_usage_contribution_backfills_ready
    ON public.provider_api_key_usage_contribution_backfills (status, updated_at);

CREATE TABLE IF NOT EXISTS public.provider_api_key_usage_projection_repairs (
    provider_api_key_id character varying(64) NOT NULL,
    main_requested boolean NOT NULL DEFAULT false,
    window_requested boolean NOT NULL DEFAULT false,
    available_at timestamp with time zone NOT NULL DEFAULT now(),
    attempts integer NOT NULL DEFAULT 0,
    last_error text,
    updated_at timestamp with time zone NOT NULL DEFAULT now(),
    CONSTRAINT provider_api_key_usage_projection_repairs_pkey PRIMARY KEY (provider_api_key_id),
    CONSTRAINT provider_api_key_usage_projection_repairs_key_fkey
        FOREIGN KEY (provider_api_key_id)
        REFERENCES public.provider_api_keys(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_provider_api_key_usage_projection_repairs_ready
    ON public.provider_api_key_usage_projection_repairs (available_at, updated_at);
