-- Add down migration script here
alter table user_webauthn_challenges alter column user_id set not null;
