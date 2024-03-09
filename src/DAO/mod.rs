use crate::model::player::Player;

pub mod postgres;

trait Database {}

struct DAO<T: Database> {
    database: T
}

impl<T: Database> DAO<T> {

    fn register(&self, player: Player) {
        self.database.register(player);
    }

    fn login(&self, player: Player) {
        self.database.login(player);
    }
}