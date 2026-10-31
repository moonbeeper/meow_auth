-- Add up migration script here
alter table user_sessions add column ip text not null default '0.0.0.0';
alter table user_sessions add column location text not null default 'Unknown';
alter table user_sessions add column user_agent text not null default 'Unknown';
