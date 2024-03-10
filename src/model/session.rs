use serde::Deserialize;
use crate::game::Game;
use crate::model::player::Player;

#[derive(Clone, Deserialize)]
pub struct Session<T> {
    session_id: SessionID,
    players: [Option<Player>; 2],
    game: T,
}

#[derive(Eq, PartialEq, Hash, Clone, Deserialize)]
pub struct SessionID(pub(crate) String);

impl<T: Game> Session<T> {
    pub fn new(player: Player, game: T) -> Self {
        Session {
            session_id: Self::generate_session_id(),
            players: [Some(player), None],
            game
        }
    }

    fn generate_session_id() -> SessionID {
        SessionID(String::new())
    }

    pub fn can_join(&self) -> bool {
        self.players[1].is_some()
    }

    pub fn add_player2(&mut self, mut player2: Player) {
        player2.set_session_id(self.session_id.clone());
        self.players[1] = Some(player2);
    }

    pub fn get_session_id(&self) -> SessionID {self.session_id.clone()}

    pub fn get_player1_name(&self) -> String {self.players[1].clone().unwrap().get_username()}
}