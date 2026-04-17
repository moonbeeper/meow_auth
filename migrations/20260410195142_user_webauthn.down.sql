-- Add down migration script here
alter table if exists users drop column if exists has_webauthn;

drop table if exists user_webauthn;
drop table if exists user_webauthn_challenges;
drop type if exists webauthn_challenge_kind;
