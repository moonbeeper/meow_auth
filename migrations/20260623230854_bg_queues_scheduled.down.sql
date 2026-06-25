-- Add down migration script here
-- redo back old stuffies
drop trigger if exists added_queued_job_trigger on queued_jobs;

-- will always a tiny fraction of time of jitter :D
create trigger added_queued_job_trigger after insert or update on queued_jobs for each row when (new.state = 'pending') execute function added_queued_job_func();

alter table queued_jobs drop column available_at;
