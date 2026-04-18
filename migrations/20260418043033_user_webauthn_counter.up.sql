-- Add up migration script here
alter table user_webauthn add column counter int not null default 1;
