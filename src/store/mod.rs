pub mod logic;
pub mod models;
pub mod schema;

use crate::utils::util::get_config_dir;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use once_cell::sync::Lazy;
use tracing::debug;

pub static GLOBAL_POOL: Lazy<Pool<ConnectionManager<SqliteConnection>>> = Lazy::new(|| {
    return get_connection_pool(&AppConfig::get_database_url());
});

pub struct AppConfig;

impl AppConfig {
    pub fn get_database_url() -> String {
        let mut config_dir = get_config_dir();
        config_dir = config_dir.join("data.dat");
        config_dir.into_os_string().into_string().unwrap()
    }
}

pub fn get_connection_pool(database_url: &str) -> Pool<ConnectionManager<SqliteConnection>> {
    debug!("database_url:{}", database_url);
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder().test_on_check_out(true).build(manager).expect("Could not build connection pool")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let pool = get_connection_pool("");
        let conn = pool.clone().get().expect("Could not get connection from pool");
    }
}
