use log::error;
use crate::Game;
use sqlx::{Error, PgPool, Row};
use sqlx::postgres::{PgPoolOptions, PgRow};
use crate::dao::Database;
use crate::game::xo::XO;
use crate::model::player::Player;
use crate::model::session::{Session, SessionID};

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

    //Default to save XO game
    async fn save_session(&self, session: Session<impl Game + Clone>, result: i32) {
        match sqlx::query("insert into session(player1_username, player2_username, result, board)\
        values ($1, $2, $3, $4)")
            .bind(session.players[0].clone().unwrap().get_username())
            .bind(session.players[1].clone().unwrap().get_username())
            .bind(result)
            .bind(session.game.to_string())
            .execute(&self.pool)
            .await {
            Ok(_) => {}
            Err(e) => {error!("Database error {}", e)}
            //TODO implement this
        };
        sqlx::query("delete from session where created_on not in (select created_on from session order by created_on desc limit 50)")
            .execute(&self.pool)
            .await
            .expect("Failed to delete to 50 session records");
    }

    //Default to deserialize XO game
    async fn get_scoreboard(&self) -> Result<Vec<Session<impl Game + Clone>>, Error> {
        match sqlx::query("select * from session order by created_on desc")
            .map(|pg_row: PgRow| {
                Session::<XO>::new_session_for_scoreboard(
                    pg_row.get("session_id"),
                    [
                        Some(Player::new(pg_row.get("player1_username"), String::new())),
                        Some(Player::new(pg_row.get("player1_username"), String::new()))
                    ],
                    pg_row.get("result"),
                    pg_row.get("board")
                )
            })
            .fetch_all(&self.pool)
            .await {
            Ok(vector) => Ok(vector),
            Err(e) => Err(e)
        }
    }
}