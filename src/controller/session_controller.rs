use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::http::StatusCode;
use crate::dao::{DAO, Database};
use crate::error::Error;
use crate::error::Error::SessionNotExist;
use crate::game::Game;
use crate::game::xo::XO;
use crate::model::session::{Session, SessionID};
use crate::model::player::Player;

pub async fn create_session(active_session: Arc<RwLock<HashMap<SessionID,
    RwLock<Session<XO>>>>>, player: Player, game: String) -> Result<impl warp::Reply, warp::Rejection> {
    let game = match game.as_str() {
        "XO" => XO::new(),
        _ => XO::new()
    };
    let session = Session::new(player, game);
    active_session.write().await.insert(session.get_session_id().clone(), RwLock::new(session.clone()));
    Ok(warp::reply::with_status(session.get_session_id().0, StatusCode::OK))
}

//Return joinable session
pub async fn get_session(active_sessions: Arc<RwLock<HashMap<SessionID,
    RwLock<Session<impl Game + Clone>>>>>) -> Result<impl warp::Reply, warp::Rejection> {
    clean_up_session(active_sessions.clone()).await;

    let result: Vec<SessionID> = active_sessions.read().await.keys().cloned().collect();
    return Ok(warp::reply::json(&result));
}

pub async fn join_session(
    session_id: String, active_sessions: Arc<RwLock<HashMap<SessionID, RwLock<Session<impl Game + Clone>>>>>,
    player2: Player) -> Result<impl warp::Reply, warp::Rejection> {
    match active_sessions.read().await.get(&SessionID(session_id.clone())) {
        Some(session) => {
            let mut session = session.write().await;
            session.add_player2(player2)
        },
        None => { return Err(warp::reject::custom(SessionNotExist)); }
    }
    Ok(warp::reply::with_status(session_id.clone(), StatusCode::OK))
}

pub async fn clean_up_session(
    mut active_sessions: Arc<RwLock<HashMap<SessionID, RwLock<Session<impl Game + Clone>>>>>
) {
    let mut ended_session_list = Vec::new();
    for (session_id, session) in active_sessions.read().await.iter() {
        if session.read().await.end {
            ended_session_list.push(session_id.clone());
        }
    }
    for session_id in ended_session_list {
        active_sessions.write().await.remove(&session_id);
    }
}

// pub async fn get_session_by_id(
//     active_sessions: Arc<RwLock<HashMap<SessionID, RwLock<Session<XO>>>>>,
//     session_id: String
// ) -> Option<&'static RwLock<Session<XO>>> {
//     active_sessions.clone().read().await.get(&SessionID(session_id))
// }
//
// async fn check_session_validity(session: Option<&RwLock<Session<XO>>>) -> bool {
//     if let Some(_) = session {
//         return true;
//     }
//     false
// }

pub async fn handle_make_a_move(
    session_id: String, active_sessions: Arc<RwLock<HashMap<SessionID, RwLock<Session<XO>>>>>,
    params: HashMap<String, String>, player: Player, dao: DAO<impl Database>
) -> Result<impl warp::Reply, warp::Rejection> {
    if let None = active_sessions.read().await.get(&SessionID(session_id.clone())) {
        return Err(warp::reject::custom(SessionNotExist));
    }
    let session = active_sessions.read().await;
    let session = session.get(&SessionID(session_id)).unwrap();
    let mut session = session.write().await;

    if !auth(session.deref_mut(), player).await {
        return Err(warp::reject::custom(Error::Unauthorized));
    }

    match params.get("move") {
        Some(value) => {
            let mut status = 0;
            {
                status = session.game.make_a_move(value.parse::<usize>().unwrap());
                session.turn = (session.turn + 1) % 2;
            }

            match status {
                1..=3 => {
                    session.end = true;
                    dao.save_session(session.clone(), status).await;
                    Ok(warp::reply::with_status(status.to_string(), StatusCode::OK))
                }
                _ => Ok(warp::reply::with_status(session.game.print(), StatusCode::OK))
            }
        }
        None => Err(warp::reject::custom(Error::AuthenticationFail))
    }
}

pub async fn handle_wait_for_move(
    session_id: String, active_sessions: Arc<RwLock<HashMap<SessionID, RwLock<Session<XO>>>>>,
    player: Player
) -> Result<impl warp::Reply, warp::Rejection> {
    if let None = active_sessions.read().await.get(&SessionID(session_id.clone())) {
        return Err(warp::reject::custom(SessionNotExist));
    }
    let session = active_sessions.write().await;
    let session = session.get(&SessionID(session_id)).unwrap();
    let mut session = session.write().await;

    if session.players[0].clone().unwrap() == player {
        if session.turn == 0 {
            Ok(warp::reply::with_status(true.to_string(), StatusCode::OK))
        } else {
            Ok(warp::reply::with_status(false.to_string(), StatusCode::OK))
        }
    } else if session.players[1].clone().unwrap() == player {
        if session.turn == 1 {
            Ok(warp::reply::with_status(true.to_string(), StatusCode::OK))
        } else {
            Ok(warp::reply::with_status(false.to_string(), StatusCode::OK))
        }
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

pub async fn handle_surrender(
    session_id: String, active_sessions: Arc<RwLock<HashMap<SessionID, RwLock<Session<XO>>>>>,
    player: Player, dao: DAO<impl Database>
) -> Result<impl warp::Reply, warp::Rejection> {
    if let None = active_sessions.read().await.get(&SessionID(session_id.clone())) {
        return Err(warp::reject::custom(SessionNotExist));
    }
    let session = active_sessions.write().await;
    let session = session.get(&SessionID(session_id)).unwrap();
    let mut session = session.write().await;

    if session.players[0].clone().unwrap() == player {
        //Player 1 at index 0 surrendered so player 2 win
        session.end = true;
        dao.save_session(session.clone(), 2).await;
        Ok(warp::reply::with_status("2", StatusCode::OK))
    } else if session.players[1].clone().unwrap() == player {
        //Player 2 at index 1 surrendered so player 1 win
        session.end = true;
        dao.save_session(session.clone(), 1).await;
        Ok(warp::reply::with_status("1", StatusCode::OK))
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

async fn save_and_shutdown(mut session: &mut Session<impl Game + Clone>, status: i32, dao: DAO<impl Database>) {
    //TODO connect to DB and do shutdown
    session.end = true;
    dao.save_session(session.clone(), status).await;
}

async fn auth(session: &Session<impl Game + Clone>, player: Player) -> bool {
    return session.players[session.turn].clone().unwrap() == player;
}