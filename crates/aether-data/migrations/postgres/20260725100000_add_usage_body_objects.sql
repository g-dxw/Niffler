CREATE TABLE IF NOT EXISTS public.usage_body_objects (
    body_ref character varying(160) NOT NULL,
    request_id character varying(100) NOT NULL,
    body_field character varying(50) NOT NULL,
    object_key text,
    payload_format character varying(32) NOT NULL DEFAULT 'json',
    content_type character varying(255),
    content_encoding character varying(64),
    size_bytes bigint,
    sha256 character varying(64),
    storage_status character varying(32) NOT NULL,
    error_message text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT usage_body_objects_pkey PRIMARY KEY (body_ref),
    CONSTRAINT usage_body_objects_request_id_field_key UNIQUE (request_id, body_field),
    CONSTRAINT usage_body_objects_request_id_fkey
        FOREIGN KEY (request_id)
        REFERENCES public.usage(request_id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_usage_body_objects_request_id
    ON public.usage_body_objects USING btree (request_id);

CREATE INDEX IF NOT EXISTS ix_usage_body_objects_status
    ON public.usage_body_objects USING btree (storage_status);
