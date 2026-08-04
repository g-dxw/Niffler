ALTER TABLE public.user_groups
    ADD COLUMN IF NOT EXISTS managed_instructions jsonb;
