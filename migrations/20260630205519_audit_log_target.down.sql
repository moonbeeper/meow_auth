-- Add down migration script here
alter table audit_logs drop column if exists actor_id;
