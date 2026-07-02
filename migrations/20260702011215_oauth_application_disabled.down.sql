-- Add down migration script here
alter table oauth_applications drop column if exists disabled;
