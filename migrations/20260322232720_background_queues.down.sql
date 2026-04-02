-- Add down migration script here

drop trigger if exists added_queued_job_trigger on queued_jobs;
drop function if exists added_queued_job_func();
drop index if exists queued_jobs_picked_pending_idx;
drop index if exists queued_jobs_picked_inprogress_idx;
-- drop index if exists queued_jobs_picked_state;
drop table if exists queued_jobs;
