-- Add up migration script here
alter table queued_jobs add column available_at timestamptz not null default now();

-- add more filters to mr trigger
drop trigger if exists added_queued_job_trigger on queued_jobs;

-- will always a tiny fraction of time of jitter :D
create trigger added_queued_job_trigger after insert or update on queued_jobs for each row when (new.state = 'pending' and new.available_at <= now()) execute function added_queued_job_func();
