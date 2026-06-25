-- Add up migration script here
create table jwks_keys (
    id uuid primary key,
    secret bytea not null,
    nonce bytea not null,
    retired boolean not null default false,
    retired_at timestamptz not null,
    max_public_age_at timestamptz not null,
    updated_at timestamptz not null default now(),
    created_at timestamptz not null default now()
);
