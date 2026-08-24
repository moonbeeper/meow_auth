-- Add up migration script here
alter table user_webauthn alter column counter set default 0;
