use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::http::StatusCode;
use crate::game::Game;
use crate::model::player::Player;
use crate::model::multithread_session::{Session, SessionID, start};
use crate::game::xo::XO;

pub async fn create_session(active_session: Arc<RwLock<HashMap<SessionID,
    Session<XO>>>>, player: Player, game: String) -> Result<impl warp::Reply, warp::Rejection> {
    let game = match game.as_str() {
        "XO" => XO::new(),
        _ => XO::new()
    };
    let session = Session::new(player, game);
    active_session.write().await.insert(session.get_session_id().clone(), session.clone());
    Ok(warp::reply::with_status(session.get_session_id().0, StatusCode::OK))
}

pub async fn get_session(active_sessions: Arc<RwLock<HashMap<SessionID,
     Session<impl Game + Clone + Send>>>>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut result:Vec<(String, String)> = Vec::<(String, String)>::new();
    for (session_id, session) in active_sessions.read().await.iter() {
        result.push((session_id.0.clone(), session.get_player1_name()))
    }
    Ok(warp::reply::json(&result))
}

pub async fn join_session(active_sessions: Arc<RwLock<HashMap<SessionID,
    Session<impl Game + 'static + Clone + Send + Sync>>>>, session_id: String, player2: Player) -> Result<impl warp::Reply, warp::Rejection> {
    match active_sessions.write().await.remove(&SessionID(session_id.clone())) {
        Some(mut session) => {
            session.add_player2(player2);
            tokio::spawn(start(session));
        },
        None => return Err(warp::reject::custom(crate::Error::AuthenticationFail))
    }
    Ok(warp::reply::with_status(session_id.clone(), StatusCode::OK))
    //TODO this is fail-able, make sure to return the right thing
}