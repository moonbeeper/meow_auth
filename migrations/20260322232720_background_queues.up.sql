-- Add up migration script here

create table queued_jobs (
    id uuid primary key,
    handler text not null,
    state text not null,
    input bytea not null,
    retry_count int not null default 0,
    retry_max_count int not null default 5,
    error_message text,
    success_at timestamptz,
    expires_at timestamptz not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);


create or replace function added_queued_job_func() returns trigger as $$ begin
    perform pg_notify('added_queued_job', 'hi'); return new; end;
    $$ language plpgsql;

create trigger added_queued_job_trigger after insert or update on queued_jobs for each row when (new.state = 'pending') execute function added_queued_job_func();

-- create index queued_jobs_picked_state on queued_jobs (state);

create index queued_jobs_picked_pending_idx on queued_jobs (created_at, id)
    where success_at is null and retry_count < retry_max_count and state = 'pending';

create index queued_jobs_picked_inprogress_idx on queued_jobs (updated_at, id)
    where success_at is null and retry_count < retry_max_count and state = 'inprogress';
