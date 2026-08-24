-- Add up migration script here
alter table user_webauthn_challenges alter column user_id drop not null;
