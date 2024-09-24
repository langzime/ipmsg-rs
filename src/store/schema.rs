diesel::table! {
    messages (id) {
        id -> Integer,
        msg_type -> Integer,
        chat_user_id -> Varchar,
        to_user_id -> Varchar,
        body -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
