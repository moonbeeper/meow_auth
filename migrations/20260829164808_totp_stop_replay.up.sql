-- Add up migration script here
alter table user_totp add column last_step_used bigint not null default 0;
