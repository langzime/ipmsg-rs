diesel::table! {
    messages (id) {
        id -> Integer,
        ver -> Text,
        message_id -> Text,
        msg_type -> Integer,
        sender_id -> Varchar,
        sender_name -> Varchar,
        receiver_id -> Varchar,
        receiver_name -> Varchar,
        msg_sub_id -> Integer,
        group_id -> Varchar,
        is_self -> Bool,
        content -> Text,
        is_read -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    meta (id) {
        id -> Integer,
        key -> Text,
        value -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    group (id) {
        id -> Integer,
        group_name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
