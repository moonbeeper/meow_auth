-- Add up migration script here
alter table audit_logs add column resource_type text;
alter table audit_logs add column resource_id uuid;
alter table audit_logs add column actor_ip text;
