use crate::store::models::{Messages, NewMessages};
use crate::store::schema::messages::dsl::*;
use crate::store::schema::messages::*;
use crate::store::GLOBAL_POOL;
use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
pub fn db_init() -> Result<()> {
    let mut conn = GLOBAL_POOL.clone().get().expect("Could not get connection from pool");
    run_migrations(&mut conn)?;
    Ok(())
}

fn run_migrations(connection: &mut impl MigrationHarness<Sqlite>) -> Result<()> {
    connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow!("run_migrations fail!{}", e))?;
    Ok(())
}

pub fn list_latest_messages(user_id: String, num: i64) -> Result<Vec<Messages>> {
    let mut conn = GLOBAL_POOL.clone().get().expect("Could not get connection from pool");
    let mut vec = messages
        .filter(chat_user_id.eq(user_id))
        .limit(num)
        .select(Messages::as_select())
        .order_by(id.desc())
        .load(&mut conn)?;
    vec.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(vec)
}

pub fn insert_message(msg: NewMessages) -> Result<i32> {
    let mut conn = GLOBAL_POOL.clone().get().expect("Could not get connection from pool");
    let msg_id = diesel::insert_into(messages).values(msg).returning(id).get_result(&mut conn)?;
    Ok(msg_id)
}
