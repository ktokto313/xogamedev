use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::model::player::Player;
use crate::model::session::{Session, SessionID};
use crate::game::xo::XO;

async fn create_session(active_session: Arc<RwLock<HashMap<SessionID, Session>>>, player: Player, game: String) {
    let game = match game {
        String::from("XO") => XO::new(),
        _ => XO::new()
    };
    let session = Session::new(player, Box::new(game));
    active_session.write().await.insert(session.get_session_id(), session);
}

async fn get_session(active_sessions: Arc<RwLock<HashMap<SessionID, Session>>>) {
    let mut result:Vec<(String, String)> = Vec::<(String, String)>::new();
    for session in active_sessions.read().await {
        result.push((session.get_session_id(), session.get_player1_name()))
    }

}

async fn join_session(mut active_sessions: Arc<RwLock<HashMap<SessionID, Session>>>, session_id: String, player2: Player) {
    match active_sessions.write().await.remove(&SessionID(session_id)) {
        Some(mut session) => session.add_player2(player2),
        None => () //Do error return thingy here
    }
    //TODO this is fail-able, make sure to return the right thing
}