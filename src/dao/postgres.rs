use crate::Game;
use sqlx::{Error, PgPool, Row};
use sqlx::postgres::{PgPoolOptions, PgRow};
use crate::dao::Database;
use crate::model::player::Player;
use crate::model::session::Session;

#[derive(Clone)]
pub struct PostgresDB {
    pool: PgPool
}

impl PostgresDB {
    pub async fn new(db_url: &str) -> PostgresDB {
        let pool = PgPoolOptions::new().connect(db_url).await.expect("Database url failed");
        PostgresDB {
            pool
        }
    }
}

impl Database for PostgresDB {
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

    async fn save_session(&self, session: Session<impl Game + Clone>, result: i32) {
        match sqlx::query("insert into session(player1_username, player2_username, result)\
        values ($1, $2, $3)")
            .bind(session.players[0].clone().unwrap().get_username())
            .bind(session.players[1].clone().unwrap().get_username())
            .bind(result)
            .execute(&self.pool)
            .await {
            Ok(_) => {}
            Err(_) => {}
            //TODO implement this
        };
        sqlx::query("delete from session where created_on not in (select created_on from session order by created_on desc limit 50)")
            .execute(&self.pool)
            .await
            .expect("Failed to delete to 50 session records");
    }
}