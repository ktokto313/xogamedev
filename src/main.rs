mod model;
mod error;
mod controller;
mod DAO;
mod game;

use std::collections::HashMap;
use std::sync::Arc;
use log;
use tokio::sync::RwLock;
use crate::model::player::Player;
use crate::model::session::{Session, SessionID};

#[tokio::main]
async fn main() {
    env_logger::init();
    //TODO: DI for DAO

    //create game session, create new thread, move game session there and start
    //when user login, check player for session_id
    let active_sessions = Arc::new(RwLock::new(HashMap::<SessionID, Session>::new()));
    //when player2 want to join, give them something

}




