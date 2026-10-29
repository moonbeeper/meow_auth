-- Add up migration script here
-- for some reason this dumb bird forgot to remove the unique constraints on user_id and client_id
-- in both models while also making a tuple unique constraint... one user could auth a client one time
-- and the same for an app
alter table oauth_pending_authorizations drop constraint if exists oauth_pending_authorizations_user_id_key;
alter table oauth_pending_authorizations drop constraint if exists oauth_pending_authorizations_client_id_key;

alter table oauth_authorizations drop constraint if exists oauth_authorizations_user_id_key;
alter table oauth_authorizations drop constraint if exists oauth_authorizations_client_id_key;
