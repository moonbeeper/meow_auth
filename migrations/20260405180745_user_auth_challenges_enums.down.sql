-- Add down migration script here
alter table user_auth_challenges alter column kind type text;
alter table user_auth_challenges alter column kind set default 'otp';
alter table user_auth_challenges alter column state type text;
alter table user_auth_challenges alter column state set default 'peding';
alter table user_auth_challenges alter column purpose type text;
alter table user_auth_challenges alter column purpose set default 'login';

drop type if exists user_auth_challenges_kind;
drop type if exists user_auth_challenges_state;
drop type if exists user_auth_challenges_purpose;
