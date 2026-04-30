-- Add down migration script here
alter table users drop column if exists pid;
