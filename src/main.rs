mod model;
mod error;
mod controller;
mod dao;
mod game;

use std::collections::HashMap;
use std::sync::Arc;
use log;
use log::error;
use tokio::sync::RwLock;
use warp::{Filter, Rejection};
use warp::body::BodyDeserializeError;
use warp::http::StatusCode;
use crate::controller::{authentication_controller, session_controller};
use crate::controller::authentication_controller::login;
use crate::dao::DAO;
use crate::dao::postgres::PostgresDB;
use crate::error::Error;
use crate::game::Game;
use crate::game::xo::XO;
use crate::model::player::Player;
use crate::model::session::{Session, SessionID};

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() {
    env_logger::init();
    //TODO: DI for dao

    //create game session, create new thread, move game session there and start
    //when user login, check player for session_id

    let session_list = Arc::new(RwLock::new(HashMap::<SessionID, Session<XO>>::new()));
    //when player2 want to join, give them something

    let db_url = "postgres://postgres:3132006kto@localhost:5432/xogamedev";
    let database = PostgresDB::new(db_url).await;
    let dao = DAO::new(database);

    let dao_filter = warp::any().map(move || {dao.clone()});
    let session_list_filter = warp::any().map(move || {session_list.clone()});
    let xo_filter = warp::any().map(|| {"XO".to_string()});
    let session_id_filter = warp::any().and(warp::path::param()).map(|session_id:String| {session_id});
    let domain_filter = warp::any().and(warp::path("xogamedev"));

    let login_filter = warp::post()
        .and(domain_filter)
        .and(warp::path("login"))
        .and(warp::body::json())
        .and(dao_filter.clone())
        .and(warp::path::end())
        .and_then(authentication_controller::login);

    let register_filter = warp::post()
        .and(domain_filter)
        .and(warp::path("register"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(dao_filter.clone())
        .and_then(authentication_controller::register);

    let create_session_filter = warp::post()
        .and(domain_filter)
        .and(warp::path("create_new_game"))
        .and(session_list_filter.clone())
        .and(warp::path::end())
        .and(warp::body::json())
        .and(xo_filter)
        .and(dao_filter)
        .and_then(session_controller::create_session);

    let get_session_filter = warp::get()
        .and(domain_filter)
        .and(warp::path("get_session"))
        .and(warp::path::end())
        .and(session_list_filter.clone())
        .and_then(session_controller::get_session);

    let join_session_filter = warp::post()
        .and(domain_filter)
        .and(warp::path("join_session"))
        .and(session_list_filter)
        .and(session_id_filter)
        .and(warp::body::json())
        .and_then(session_controller::join_session);

    let filter = login_filter
        .or(register_filter)
        .or(create_session_filter)
        .or(get_session_filter)
        .or(join_session_filter)
        .recover(handle_error);

    warp::serve(filter)
        .run(([127, 0, 0, 1], 1337))
        .await;
}

async fn handle_error(r: Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(e) = r.find::<BodyDeserializeError>() {
        error!("{}", e.to_string());
        Ok(warp::reply::with_status(e.to_string(), StatusCode::UNPROCESSABLE_ENTITY))
    } else if let Some(Error::InvalidMove) = r.find() {
        error!("Invalid action");
        Ok(warp::reply::with_status("Invalid action, please try again".to_string(), StatusCode::BAD_REQUEST))
    } else if let Some(Error::BadRequest) = r.find() {
        error!("Username duplicate or wrong username/password");
        Ok(warp::reply::with_status("Username duplicate or wrong username/password".to_string(), StatusCode::BAD_REQUEST))
    } else if let Some(Error::Unauthorized) = r.find() {
        error!("User not logged in");
        Ok(warp::reply::with_status("You are not logged in".to_string(), StatusCode::UNAUTHORIZED))
    } else if let Some(Error::DatabaseError(e)) = r.find() {
        error!("Database error {}", e);
        Ok(warp::reply::with_status(e.to_string(), StatusCode::BAD_REQUEST))
    } else {
        error!("Can't find resources");
        Ok(warp::reply::with_status("Can't find resources".to_string(), StatusCode::NOT_FOUND))
    }
}