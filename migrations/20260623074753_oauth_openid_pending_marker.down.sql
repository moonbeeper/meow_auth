-- Add down migration script here
alter table oauth_pending_tokens drop column is_openid;
alter table oauth_pending_authorizations drop column is_openid;
