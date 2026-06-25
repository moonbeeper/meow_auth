-- Add up migration script here
alter table oauth_pending_tokens add column is_openid boolean not null default false;
alter table oauth_pending_authorizations add column is_openid boolean not null default false;
