-- Your SQL goes here
create table "messages" (
    id integer PRIMARY KEY AUTOINCREMENT,
    message_id text not null,
    ver text not null,
    msg_type int not null,
    sender_id text not null,
    sender_name text not null,
    receiver_id text not null ,
    receiver_name text not null ,
    group_id text not null default '',
    is_self boolean not null,
    content text not null default '',
    is_read boolean not null ,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER IF NOT EXISTS trigger_messages_updated_at AFTER UPDATE ON messages
BEGIN
    UPDATE messages SET updated_at = CURRENT_TIMESTAMP WHERE rowid == NEW.rowid;
END;

create table "meta" (
    id integer PRIMARY KEY AUTOINCREMENT,
    key text not null,
    value text not null,
    created_at timestamp default (UNIXEPOCH()),
    updated_at timestamp default (UNIXEPOCH())
);

create table "group" (
    id integer PRIMARY KEY AUTOINCREMENT,
    group_name text not null,
    value text not null,
    created_at timestamp default (UNIXEPOCH()),
    updated_at timestamp default (UNIXEPOCH())
);