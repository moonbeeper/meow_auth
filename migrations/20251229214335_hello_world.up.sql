-- Add up migration script here

create table hello_world
(
    id      uuid primary key,
    message text not null
);