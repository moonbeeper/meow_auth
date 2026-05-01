-- Add up migration script here
create table oauth_pending_authorizations (
    id uuid primary key,
    user_id uuid unique not null references users(id) on delete cascade,
    user_session uuid unique not null references user_sessions(id) on delete cascade,
    client_id uuid unique not null references oauth_applications(id) on delete cascade,
    old_scopes bigint default 0, -- bitfield of scopes as always :)
    requested_scopes bigint not null default 0, -- bitfield of scopes as always :)
    code_challenge text not null, -- always S256!!
    state text,
    nonce text,
    expires_at timestamptz not null,
    created_at timestamptz not null default now(),
    unique (user_id, client_id) -- one pending authorization per user per client
);

create table oauth_pending_tokens (
    code text primary key, -- nanoid or rand
    user_id uuid not null references users(id) on delete cascade,
    client_id uuid not null references oauth_applications(id) on delete cascade,
    scopes bigint not null,
    code_challenge text not null, -- always S256!!
    state text,
    nonce text,
    expires_at timestamptz not null,
    created_at timestamptz not null default now(),
    unique (user_id, client_id) -- one pending authorization per user per client
);
