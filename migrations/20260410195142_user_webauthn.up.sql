-- Add up migration script here
alter type user_auth_challenges_kind add value if not exists 'webauthn';

create table user_webauthn (
    id uuid primary key,
    pid uuid unique not null,
    user_id uuid not null references users(id) on delete cascade,
    display_name varchar(50) not null default 'New Passkey',
    credential_id bytea unique not null,
    aaguid uuid not null default '00000000-0000-0000-0000-000000000000',
    big_data jsonb not null,
    last_used_at timestamptz,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create type webauthn_challenge_kind as enum (
    'register',
    'authenticate'
);

create table user_webauthn_challenges (
    id uuid primary key,
    user_id uuid not null references users(id) on delete cascade,
    kind webauthn_challenge_kind not null default 'authenticate',
    big_data jsonb not null,
    expires_at timestamptz not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);


alter table users add column has_webauthn boolean not null default false;
