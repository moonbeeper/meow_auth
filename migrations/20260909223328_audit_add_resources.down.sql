-- Add down migration script here
alter table audit_logs drop column resource_type;
alter table audit_logs drop column resource_id;
alter table audit_logs drop column actor_ip;
