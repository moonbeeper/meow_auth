-- Add up migration script here
alter table user_sessions add column if not exists sudo_expires_at timestamptz;
alter table user_login_requests rename to user_auth_challenges;
alter table user_auth_challenges add column if not exists purpose text not null default 'login';
alter table user_auth_challenges add column if not exists user_session_id uuid references user_sessions(id) on delete cascade;
