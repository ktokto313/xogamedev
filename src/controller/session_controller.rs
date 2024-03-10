use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::game::Game;
use crate::model::player::Player;
use crate::model::session::{Session, SessionID};
use crate::game::xo::XO;

pub async fn create_session(active_session: Arc<RwLock<HashMap<SessionID,
    Session<XO>>>>, player: Player, game: String) -> Result<impl warp::Reply, warp::Rejection> {
    let game = match game.as_str() {
        "XO" => XO::new(),
        _ => XO::new()
    };
    let session = Session::new(player, game);
    active_session.write().await.insert(session.get_session_id().clone(), session);
    Ok(warp::reply())
}

pub async fn get_session(active_sessions: Arc<RwLock<HashMap<SessionID,
    Session<impl Game>>>>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut result:Vec<(String, String)> = Vec::<(String, String)>::new();
    for (sessionID, session) in active_sessions.read().await.iter() {
        result.push((sessionID.0.clone(), session.get_player1_name()))
    }
    Ok(warp::reply())
}

pub async fn join_session(mut active_sessions: Arc<RwLock<HashMap<SessionID,
    Session<impl Game>>>>, session_id: String, player2: Player) -> Result<impl warp::Reply, warp::Rejection> {
    match active_sessions.write().await.remove(&SessionID(session_id)) {
        Some(mut session) => session.add_player2(player2),
        None => () //Do error return thingy here
    }
    Ok(warp::reply())
    //TODO this is fail-able, make sure to return the right thing
}