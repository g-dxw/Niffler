ALTER TABLE public.payment_gateway_configs
  ADD COLUMN IF NOT EXISTS webhook_secret_encrypted text;
