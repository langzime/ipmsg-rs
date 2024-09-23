use crate::store::models::Messages;
use crate::store::schema::messages::dsl::*;
use crate::store::schema::messages::id;
use crate::store::GLOBAL_POOL;
use anyhow::Result;
use diesel::prelude::*;

pub fn save_msg() -> Result<()> {
    let mut conn = GLOBAL_POOL.clone().get().expect("Could not get connection from pool");
    let message1 = messages.select(id).find(id).select(Messages::as_select()).first(&mut conn).optional();
    Ok(())
}
