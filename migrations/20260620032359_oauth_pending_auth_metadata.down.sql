-- Add down migration script here

alter table oauth_pending_authorizations drop column old_authorization_id;
alter table oauth_pending_authorizations drop column redirect_url;
