-- Add up migration script here
create table oauth_applications (
    id uuid primary key,
    user_id uuid not null references users(id) on delete cascade,
    name varchar(32) not null,
    redirect_uri text not null,
    secret text not null,
    public boolean not null default false,
    scopes bigint not null default 0, -- bitfield of scopes as always :)
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table oauth_tokens (
    id uuid primary key,
    user_id uuid not null references users(id) on delete cascade,
    client_id uuid not null references oauth_applications(id) on delete cascade,
    token text not null,
    scopes bigint not null default 0, -- bitfield of scopes as always :)
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table oauth_authorizations (
    id uuid primary key,
    user_id uuid unique not null references users(id) on delete cascade,
    client_id uuid unique not null references oauth_applications(id) on delete cascade,
    scopes bigint not null default 0, -- bitfield of scopes as always :)
    last_used_at timestamptz,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    unique (user_id, client_id) -- one authorization per user per client
);
