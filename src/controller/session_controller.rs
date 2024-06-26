use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::http::StatusCode;
use warp::path::param;
use crate::dao::{DAO, Database};
use crate::error::Error;
use crate::error::Error::{DatabaseError, SessionNotExist};
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

    let mut result: Vec<(SessionID, String)> = Vec::new();
    for (session_id, session) in active_sessions.read().await.iter() {
        result.push((session_id.clone(), session.read().await.players[0].clone().unwrap().get_username()));
    }
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

pub async fn handle_make_a_move(
    session_id: String,
    active_sessions: Arc<RwLock<HashMap<SessionID, RwLock<Session<XO>>>>>,
    params: HashMap<String, String>, dao: DAO<impl Database>
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("handle make a move");
    if let None = active_sessions.read().await.get(&SessionID(session_id.clone())) {
        return Err(warp::reject::custom(SessionNotExist));
    }
    let session = active_sessions.read().await;
    let session = session.get(&SessionID(session_id)).unwrap();
    let mut session = session.write().await;

    let mut player = Player::new(
        params.get("username").unwrap_or(&"".to_string()).clone(),
        params.get("password").unwrap_or(&"".to_string()).clone()
    );
    player.set_session_id(session.get_session_id().clone());

    if !auth(session.deref_mut(), player).await {
        return Err(warp::reject::custom(Error::Unauthorized));
    }

    match params.get("move") {
        Some(value) => {
            let mut status : usize = 0;
            status = session.game.make_a_move(value.parse::<usize>().unwrap());
            session.status = status;
            session.turn = (session.turn + 1) % 2;

            match status {
                1..=3 => {
                    session.end = true;
                    dao.save_session(session.clone(), status as i32).await;
                    Ok(warp::reply::with_status(format!("{} {}", session.status.to_string(), session.game.print()), StatusCode::OK))
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
    println!("handle wait for move");
    if let None = active_sessions.read().await.get(&SessionID(session_id.clone())) {
        return Err(warp::reject::custom(SessionNotExist));
    }
    let session = active_sessions.write().await;
    let session = session.get(&SessionID(session_id)).unwrap();
    let mut session = session.write().await;

    if session.can_join() {
        return Ok(warp::reply::with_status(false.to_string(), StatusCode::OK));
    }

    if session.end {
        return Ok(warp::reply::with_status(format!("{} {}", session.status.to_string(), session.game.print()), StatusCode::OK));
    }

    if session.players[0].clone().unwrap() == player {
        if session.turn == 0 {
            Ok(warp::reply::with_status(session.game.print(), StatusCode::OK))
        } else {
            Ok(warp::reply::with_status(false.to_string(), StatusCode::OK))
        }
    } else if session.players[1].clone().unwrap() == player {
        if session.turn == 1 {
            Ok(warp::reply::with_status(session.game.print(), StatusCode::OK))
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
    println!("handle surrender");
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

//todo implement scoreboard
pub async fn handle_scoreboard(dao: DAO<impl Database>) -> Result<impl warp::Reply, warp::Rejection> {
    //format!("| {:<20} | {:<20} | {:<12} \n {}");
    //Player 1, player 2, status, game board
    match dao.get_scoreboard().await {
        Ok(vec) => {
            let mut vec_result = Vec::new();
            for session in vec {
                vec_result.push((
                    session.players[0].clone().unwrap().get_username(),
                    session.players[1].clone().unwrap().get_username(),
                    session.status.to_string(),
                    session.game.print()
                ))
            }

            Ok(warp::reply::with_status(serde_json::to_string(&vec_result).unwrap(), StatusCode::OK))
        },
        Err(e) => Err(warp::reject::custom(DatabaseError(e)))
    }
}

async fn save_and_shutdown(mut session: &mut Session<impl Game + Clone>, status: usize, dao: DAO<impl Database>) {
    //TODO connect to DB and do shutdown
    session.end = true;
    dao.save_session(session.clone(), status as i32).await;
}

async fn auth(session: &Session<impl Game + Clone>, player: Player) -> bool {
    return session.players[session.turn].clone().unwrap() == player;
}