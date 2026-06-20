-- Add up migration script here

alter table oauth_pending_authorizations add column old_authorization_id uuid references oauth_authorizations(id) on delete cascade;
alter table oauth_pending_authorizations add column redirect_url text not null;
