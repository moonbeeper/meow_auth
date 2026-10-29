-- Add down migration script here
alter table oauth_pending_authorizations add constraint if exists oauth_pending_authorizations_user_id_key (user_id);
alter table oauth_pending_authorizations add constraint if exists oauth_pending_authorizations_client_id_key (client_id);

alter table oauth_authorizations add constraint if exists oauth_authorizations_user_id_key (user_id);
alter table oauth_authorizations add constraint if exists oauth_authorizations_client_id_key (client_id);
