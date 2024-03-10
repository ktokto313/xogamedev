use sqlx::Error;
use warp::http::StatusCode;
use crate::model::player::Player;

pub mod postgres;

pub trait Database {
    async fn login(&self, player: Player) -> Result<bool, Error>;

    async fn register(&self, player: Player) -> Result<bool, Error>;
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
}