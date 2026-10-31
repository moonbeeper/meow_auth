-- Add down migration script here
alter table user_sessions drop column ip;
alter table user_sessions drop column location ;
alter table user_sessions drop column user_agent;
