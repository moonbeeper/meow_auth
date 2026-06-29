-- Add up migration script here
alter table users add column flags bigint default 0 not null;
