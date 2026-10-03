-- Add up migration script here
insert into users (
    id,
    pid,
    name,
    email,
    email_verified,
    totp_enabled,
    has_webauthn,
    flags,
    name_updated_at,
    created_at,
    updated_at
) values (
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000001',
    'system',
    'system@meowauth.invalid', -- :)
    true, false, false, 0,
    now(), now(), now()
);
