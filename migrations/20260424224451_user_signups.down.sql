-- Add down migration script here
drop table if exists user_signups;
alter table user_auth_challenges drop constraint if exists user_auth_challenges_either_user_id_or_signup_id;
alter table user_auth_challenges drop column if exists user_signup_id;
alter table user_auth_challenges alter column user_id set not null;
