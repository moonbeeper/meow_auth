-- Add down migration script here
alter table user_webauthn drop column if exists counter;
