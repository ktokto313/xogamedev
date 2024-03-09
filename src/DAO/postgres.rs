use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use crate::DAO::Database;

struct Postgres_DB {
    pool: PgPool
}

impl Postgres_DB {
    fn new(db_url: &str) -> Postgres_DB {
        let pool = PgPoolOptions::new().connect(db_url);
        Postgres_DB {
            pool
        }
    }
}

impl Database for Postgres_DB {}