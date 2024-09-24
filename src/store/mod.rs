pub mod logic;
pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use once_cell::sync::Lazy;

pub static GLOBAL_POOL: Lazy<Pool<ConnectionManager<SqliteConnection>>> = Lazy::new(|| {
    return get_connection_pool(&AppConfig::get_database_url());
});

pub struct AppConfig;

impl AppConfig {
    pub fn get_database_url() -> String {
        if let Some(mut tmp) = dirs::config_dir() {
            tmp = tmp.join("ipmsg-rs");
            if tmp.exists() == false {
                std::fs::create_dir_all(&tmp).expect("create dir failed");
            }
            tmp = tmp.join("data.dat");
            tmp.into_os_string().into_string().unwrap()
        } else {
            panic!("获取操作系统配置目录失败！");
        }
    }
}

pub fn establish_connection(database_url: &str) -> SqliteConnection {
    SqliteConnection::establish(&database_url).unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_connection_pool(database_url: &str) -> Pool<ConnectionManager<SqliteConnection>> {
    println!("database_url:{}", database_url);
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder().test_on_check_out(true).build(manager).expect("Could not build connection pool")
}

#[test]
pub mod test {
    use super::*;

    #[test]
    pub fn test() {
        let pool = get_connection_pool("");
        let conn = pool.clone().get().expect("Could not get connection from pool");
    }
}
