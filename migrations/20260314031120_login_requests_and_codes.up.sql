-- Add up migration script here

create table user_login_requests (
    id uuid primary key,
    user_id uuid not null references users(id) on delete cascade,
    kind text not null,
    secret text,
    state text not null,
    expires_at timestamptz not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);
