use diesel::prelude::*;
use time::{OffsetDateTime, PrimitiveDateTime};

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::store::schema::messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Messages {
    pub id: i32,
    pub ver: String,
    pub message_id: String,
    pub msg_type: i32,
    pub sender_id: String,
    pub sender_name: String,
    pub receiver_id: String,
    pub receiver_name: String,
    pub group_id: String,
    pub is_self: bool,
    pub content: String,
    pub is_read: bool,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::store::schema::messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewMessage {
    pub ver: String,
    pub message_id: String,
    pub msg_type: i32,
    pub sender_id: String,
    pub sender_name: String,
    pub receiver_id: String,
    pub receiver_name: String,
    pub group_id: String,
    pub is_self: bool,
    pub content: String,
    pub is_read: bool,
}
