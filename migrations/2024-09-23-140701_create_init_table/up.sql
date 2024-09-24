-- Your SQL goes here
create table messages(
    id integer primary key,
    msg_type integer not null,
    chat_user_id text not null,
    body text not null,
    to_user_id text not null default '',
    created_at timestamp default now(),
    updated_at timestamp default now()
);