-- Add up migration script here
alter table user_webauthn add column enabled bool not null default true;
alter table user_webauthn add column disabled_at timestamptz;
