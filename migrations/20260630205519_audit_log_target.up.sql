-- Add up migration script here
alter table audit_logs add column actor_id uuid;
update audit_logs set actor_id = user_id where actor_id is null;
alter table audit_logs alter column actor_id set not null;
