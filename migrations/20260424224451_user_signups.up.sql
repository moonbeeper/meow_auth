-- Add up migration script here
create table user_signups (
    id uuid primary key,
    login varchar(42) not null unique,
    email text not null,
    created_at timestamptz not null default now(),
    expires_at timestamptz not null
);

alter type user_auth_challenges_purpose add value if not exists 'signup';

alter table user_auth_challenges alter column user_id drop not null;
alter table user_auth_challenges add column user_signup_id uuid references user_signups(id) on delete cascade;
alter table user_auth_challenges add constraint user_auth_challenges_either_user_id_or_signup_id check (
    (user_id is not null and user_signup_id is null) or
    (user_id is null and user_signup_id is not null)
)
