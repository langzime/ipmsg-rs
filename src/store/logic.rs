use crate::store::models::Messages;
use crate::store::schema::messages::dsl::*;
use crate::store::schema::messages::id;
use crate::store::GLOBAL_POOL;
use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
pub fn db_init() -> Result<()> {
    let mut conn = GLOBAL_POOL.clone().get().expect("Could not get connection from pool");
    // let message1 = messages.select(id).find(id).select(Messages::as_select()).first(&mut conn).optional();
    run_migrations(&mut conn)?;
    Ok(())
}

fn run_migrations(connection: &mut impl MigrationHarness<Sqlite>) -> Result<()> {
    connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow!("run_migrations fail!{}", e))?;
    Ok(())
}
