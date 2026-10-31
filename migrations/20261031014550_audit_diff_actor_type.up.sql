-- Add up migration script here
alter table audit_logs drop column metadata;
alter table audit_logs add column actor_type text default 'user_self';
alter table audit_logs add column changes jsonb not null default '[]'::jsonb;

update audit_logs set actor_type = case
    when actor_id = user_id then 'user_self'
    when actor_id = '00000000-0000-0000-0000-000000000000' then 'system'
    else 'admin'
end;

alter table audit_logs alter column actor_type set not null;
