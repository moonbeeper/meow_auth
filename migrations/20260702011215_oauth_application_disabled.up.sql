-- Add up migration script here
alter table oauth_applications add column disabled boolean not null default false;
