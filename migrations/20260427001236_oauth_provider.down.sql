-- Add down migration script here
drop table if exists oauth_authorizations;
drop table if exists oauth_tokens;
drop table if exists oauth_applications;
