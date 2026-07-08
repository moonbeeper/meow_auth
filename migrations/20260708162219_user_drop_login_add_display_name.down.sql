-- Add down migration script here
alter table users drop column name;
alter table users drop constraint users_email_unique;
alter table user_signups drop constraint user_signups_email_unique;
alter table users rename column name_updated_at to login_updated_at;
alter table users add column login varchar(42) not null unique default gen_random_uuid()::text;
alter table user_signups add column login varchar(42) not null unique default gen_random_uuid()::text;
