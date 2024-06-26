use sqlx::Error;
use sqlx::postgres::PgRow;
use crate::game::Game;
use crate::model::player::Player;
use crate::model::session::Session;

pub mod postgres;

pub trait Database {
    async fn login(&self, player: Player) -> Result<bool, Error>;

    async fn register(&self, player: Player) -> Result<bool, Error>;
    async fn save_session(&self, session: Session<impl Game + Clone>, result: i32);
    //Default to deserialize XO game
    async fn get_scoreboard(&self) -> Result<Vec<Session<impl Game + Clone>>, Error>;
}

#[derive(Clone)]
pub struct DAO<T: Database> {
    database: T
}

impl<T: Database> DAO<T> {
    pub fn new(database: T) -> Self {
        DAO {
            database
        }
    }

    pub async fn register(&self, player: Player) -> Result<bool, sqlx::Error> {
        self.database.register(player).await
    }

    pub async fn login(&self, player: Player) -> Result<bool, sqlx::Error> {
        self.database.login(player).await
    }

    pub async fn save_session(self, session: Session<impl Game + Clone>, result: i32) {
        self.database.save_session(session, result).await
    }

    pub async fn get_scoreboard<'a>(&'a self) -> Result<Vec<Session<impl Game + Clone +'a>>, Error> {
        self.database.get_scoreboard().await
    }

}