ALTER TABLE public.stats_user_daily_model
    ADD COLUMN IF NOT EXISTS official_cost numeric(20,8) DEFAULT '0'::double precision NOT NULL;

ALTER TABLE public.stats_user_daily_provider
    ADD COLUMN IF NOT EXISTS official_cost numeric(20,8) DEFAULT '0'::double precision NOT NULL;

ALTER TABLE public.stats_user_daily_api_format
    ADD COLUMN IF NOT EXISTS official_cost numeric(20,8) DEFAULT '0'::double precision NOT NULL;
