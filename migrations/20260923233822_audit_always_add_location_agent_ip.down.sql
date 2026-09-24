-- Add down migration script here
alter table audit_logs drop column actor_location;
alter table audit_logs drop column actor_user_agent;
alter table audit_logs alter column actor_ip drop not null;
