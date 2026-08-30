-- Add down migration script here
alter table user_totp drop column last_step_used;
