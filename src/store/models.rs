use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::store::schema::messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Messages {
    pub id: i32,
    pub chat_user_id: String,
    pub msg_type: i32,
    pub to_user_id: String,
    pub body: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::store::schema::messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewMessages {
    pub chat_user_id: String,
    pub msg_type: i32,
    pub to_user_id: String,
    pub body: String,
}
