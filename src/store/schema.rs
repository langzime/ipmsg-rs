diesel::table! {
    messages (id) {
        id -> Integer,
        msg_type -> Integer,
        sender_id -> Varchar,
        receiver_id -> Varchar,
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
