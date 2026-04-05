-- Add up migration script here
create type user_auth_challenges_kind as enum (
    'otp',
    'totp'
);

create type user_auth_challenges_state as enum (
    'pending',
    'completed'
);

create type user_auth_challenges_purpose as enum (
    'login',
    'sudo'
);

alter table user_auth_challenges alter column kind set default 'otp'::user_auth_challenges_kind;
alter table user_auth_challenges alter column state set default 'pending'::user_auth_challenges_state;
alter table user_auth_challenges alter column purpose set default 'login'::user_auth_challenges_purpose;
alter table user_auth_challenges alter column kind type user_auth_challenges_kind using lower(kind::text)::user_auth_challenges_kind;
alter table user_auth_challenges alter column state type user_auth_challenges_state using lower(state::text)::user_auth_challenges_state;
alter table user_auth_challenges alter column purpose type user_auth_challenges_purpose using lower(purpose::text)::user_auth_challenges_purpose;
