-- Add down migration script here
alter table user_sessions drop column if exists sudo_expires_at;
alter table user_auth_challenges drop column if exists purpose;
alter table user_auth_challenges drop column if exists user_session_id;
alter table user_auth_challenges rename to user_login_requests;
