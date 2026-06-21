-- Add down migration script here
alter table oauth_applications alter column secret set data type text;
