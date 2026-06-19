-- Add up migration script here

create table audit_logs (
    id uuid primary key,
    user_id uuid not null references users(id) on delete cascade, -- must delete old requests to be able to create a new one
    action text not null, -- enum in rust app
    metadata jsonb not null default '{}'::jsonb,
    created_at timestamptz not null default now()
);
