-- Add up migration script here
alter table users drop column login;
alter table user_signups drop column login;
alter table users add column name varchar(50);
update users set name = email where name is null;
alter table users alter column name set not null;
alter table users add constraint users_email_unique unique (email);
alter table user_signups add constraint user_signups_email_unique unique (email);
alter table users rename column login_updated_at to name_updated_at;
