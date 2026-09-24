-- Add up migration script here
alter table audit_logs add column actor_location text not null default 'Unknown';
alter table audit_logs add column actor_user_agent text not null default 'Unknown';
update audit_logs set actor_ip = '0.0.0.0' where actor_ip is null;
alter table audit_logs alter column actor_ip set not null;
