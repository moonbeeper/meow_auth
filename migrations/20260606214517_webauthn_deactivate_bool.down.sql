-- Add down migration script here
alter table user_webauthn drop column if exists enabled;
alter table user_webauthn drop column if exists disabled_at;
