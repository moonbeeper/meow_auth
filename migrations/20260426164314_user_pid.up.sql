-- Add up migration script here
alter table users add column pid uuid not null unique;
