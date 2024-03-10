use sqlx::{Error, PgPool, Row};
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::query::Query;
use crate::dao::Database;
use crate::model::player::Player;

#[derive(Clone)]
pub struct Postgres_DB {
    pool: PgPool
}

impl Postgres_DB {
    pub async fn new(db_url: &str) -> Postgres_DB {
        let pool = PgPoolOptions::new().connect(db_url).await.expect("Database url failed");
        Postgres_DB {
            pool
        }
    }
}

impl Database for Postgres_DB {
    async fn login(&self, player: Player) -> Result<bool, Error> {
        match sqlx::query("select * from player where username = $1 and password = $2")
            .bind(player.get_username())
            .bind(player.get_password())
            .map(|row: PgRow| {
                Player::new(row.get("username"), row.get("password"));
            })
            .fetch_optional(&self.pool)
            .await {
            Ok(result) => match result {
                Some(_) => Ok(true),
                None => Ok(false)
            },
            Err(e) => Err(e)
        }
    }

    async fn register(&self, player: Player) -> Result<bool, Error> {
        match sqlx::query("insert into player (username, password) values \
            ($1, $2)")
            .bind(player.get_username())
            .bind(player.get_password())
            .execute(&self.pool)
            .await {
            Ok(_) => Ok(true),
            Err(e) => Err(e)
        }
    }
}