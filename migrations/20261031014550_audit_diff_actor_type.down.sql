-- Add down migration script here
alter table audit_logs add column metadata jsonb not null default '{}'::jsonb;
alter table audit_logs drop column actor_type;
alter table audit_logs drop column changes;
