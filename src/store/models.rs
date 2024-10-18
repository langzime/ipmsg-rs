use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::store::schema::messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Messages {
    pub id: i32,
    pub msg_type: i32,
    pub sender_id: String,
    pub receiver_id: String,
    pub group_id: String,
    pub is_self: bool,
    pub content: String,
    pub is_read: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::store::schema::messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewMessages {
    pub msg_type: i32,
    pub sender_id: String,
    pub receiver_id: String,
    pub group_id: String,
    pub is_self: bool,
    pub content: String,
    pub is_read: bool,
}
