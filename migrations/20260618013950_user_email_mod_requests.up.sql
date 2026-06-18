-- Add up migration script here

create table user_email_mod_requests (
    id uuid primary key,
    user_id uuid not null references users(id) on delete cascade, -- must delete old requests to be able to create a new one
    current_email text not null,
    current_email_verified boolean not null default false,
    current_email_token bytea not null,
    new_email text not null,
    new_email_verified boolean not null default false,
    new_email_token bytea not null,
    expires_at timestamptz not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    unique (user_id, current_email, new_email) -- one request per user per email change
);
