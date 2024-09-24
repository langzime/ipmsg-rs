-- Your SQL goes here
create table "messages" (
    msg_id integer PRIMARY KEY AUTOINCREMENT,
    msg_type int not null,
    chat_user_id text not null,
    body text not null default '',
    to_user_id text not null ,
    created_at timestamp default (UNIXEPOCH()),
    updated_at timestamp default (UNIXEPOCH())
);