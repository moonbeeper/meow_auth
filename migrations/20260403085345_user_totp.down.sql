-- Add down migration script here
alter table users add column if not exists password_hash text;
drop table if exists user_totp;
alter table users drop column if exists totp_enabled;
