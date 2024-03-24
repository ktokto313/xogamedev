use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::SystemTime;
use serde::{Deserialize, Serialize, Serializer};
use tokio::sync::{mpsc};
use warp::http::StatusCode;
use crate::dao::{DAO, Database};
use crate::error::Error;
use crate::game::Game;
use crate::game::xo::XO;
use crate::model::player::Player;

#[derive(Clone, Deserialize)]
pub struct Session<T> where T: Game + Clone {
    session_id: SessionID,
    pub players: [Option<Player>; 2],
    pub game: T,
    pub turn: usize,
    pub end: bool,
    pub status: usize
}

#[derive(Eq, PartialEq, Hash, Clone, Serialize, Deserialize, Debug)]
pub struct SessionID(pub String);

//impl<T: Game + Sized + Clone + Send>
impl<T: Game + Clone> Session<T> {
    pub fn new(mut player: Player, game: T) -> Self {
        let session_id = Self::generate_session_id();
        player.set_session_id(session_id.clone());
        Session {
            session_id,
            players: [Some(player), None],
            game,
            turn: 0,
            end: false,
            status: 0
        }
    }

    pub fn new_session_for_scoreboard(session_id: i32, players: [Option<Player>; 2], status: String, board: String) -> Self {
        Session {
            session_id: SessionID(session_id.to_string()),
            players,
            game: T::from_string(board.as_str()),
            turn: 0,
            end: true,
            status: status.parse::<usize>().unwrap()
        }
    }

    fn generate_session_id() -> SessionID {
        let mut hasher = DefaultHasher::new();
        SystemTime::now().hash(&mut hasher);
        SessionID(hasher.finish().to_string())
    }

    pub fn can_join(&self) -> bool {
        !self.players[1].is_some()
    }

    pub fn add_player2(&mut self, mut player2: Player) {
        player2.set_session_id(self.session_id.clone());
        self.players[1] = Some(player2);
    }

    pub fn get_session_id(&self) -> SessionID { self.session_id.clone() }

    pub fn get_player1_name(&self) -> String { self.players[1].clone().unwrap().get_username() }

    pub fn print_status(&self) -> String {
        match self.status {
            1 => "Player 1 win the game",
            2 => "Player 2 win the game",
            3 => "DRAW!",
            _ => ""
        }.to_string()
    }

    pub fn end(&mut self) {self.end = true;}
}