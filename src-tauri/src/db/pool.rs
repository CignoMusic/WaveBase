use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn create_pool(db_path: &Path) -> DbPool {
    let manager = SqliteConnectionManager::file(db_path);
    Pool::builder()
        .max_size(4)
        .build(manager)
        .expect("Failed to create database connection pool")
}
