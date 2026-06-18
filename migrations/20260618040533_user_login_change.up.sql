-- Add up migration script here
alter table users add column login_updated_at timestamptz not null default now()
