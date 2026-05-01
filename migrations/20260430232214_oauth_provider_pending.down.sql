-- Add down migration script here
drop table if exists oauth_pending_authorizations;
drop table if exists oauth_pending_tokens;
