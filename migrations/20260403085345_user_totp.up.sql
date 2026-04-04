-- Add up migration script here
alter table users drop column if exists password_hash;

create table user_totp (
    id uuid primary key,
    user_id uuid unique not null references users(id) on delete cascade,
    recovery_secret bytea not null,
    recovery_secret_nonce bytea not null,
    recovery_used int not null default 0,
    secret bytea not null,
    secret_nonce bytea not null,
    last_used_at timestamptz,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

alter table users add column totp_enabled boolean not null default false;
