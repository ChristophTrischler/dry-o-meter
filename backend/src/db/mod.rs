use sqlx::postgres::PgPool;
use std::sync::OnceLock;

use crate::config::DbConfig;

pub mod repository;

pub static SQLX_POOL: OnceLock<PgPool> = OnceLock::new();

pub async fn init(config: &DbConfig) {
    let sqlx_pool = PgPool::connect(&config.url)
        .await
        .expect("Database connection failed.");

    // Run database migrations on startup
    sqlx::migrate!("./migrations")
        .run(&sqlx_pool)
        .await
        .expect("Database migration failed.");

    crate::db::SQLX_POOL
        .set(sqlx_pool)
        .expect("sqlx pool should be set")
}

pub fn pool() -> &'static PgPool {
    SQLX_POOL.get().expect("sqlx pool should be set")
}
