-- Add up migration script here

create table users (
    id uuid primary key,
    login varchar(42) not null unique,
    email text not null,
    email_verified boolean not null default false,
    password_hash text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table user_sessions (
    id uuid primary key,
    user_id uuid not null references users(id) on delete cascade,
    pid uuid not null unique,
    active_expires_at timestamptz not null,
    expires_at timestamptz not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

drop table if exists hello_world;
