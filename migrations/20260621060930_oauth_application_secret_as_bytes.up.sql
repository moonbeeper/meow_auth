-- Add up migration script here
alter table oauth_applications alter column secret type bytea using cast(secret as bytea);
