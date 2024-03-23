use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::SystemTime;
use serde::Deserialize;
use std::sync::{Arc};
use tokio::sync::{mpsc, RwLock};
use warp::Filter;
use warp::http::StatusCode;
use crate::dao::{DAO, Database};
use crate::dao::postgres::PostgresDB;
use crate::error::Error;
use crate::game::Game;
use crate::model::player::Player;

#[derive(Clone, Deserialize)]
pub struct Session<T> where T: Clone {
    session_id: SessionID,
    pub players: [Option<Player>; 2],
    game: T,
    turn: usize,
}

#[derive(Eq, PartialEq, Hash, Clone, Deserialize)]
pub struct SessionID(pub(crate) String);

//impl<T: Game + Sized + Clone + Send>
impl<T: Game + Sized + Clone + Send> Session<T> {
    pub fn new(player: Player, game: T) -> Self {
        Session {
            session_id: Self::generate_session_id(),
            players: [Some(player), None],
            game,
            turn: 0,
        }
    }

    fn generate_session_id() -> SessionID {
        let mut hasher = DefaultHasher::new();
        SystemTime::now().hash(&mut hasher);
        SessionID(hasher.finish().to_string())
    }

    pub fn can_join(&self) -> bool {
        self.players[1].is_some()
    }

    pub fn add_player2(&mut self, mut player2: Player) {
        //player2.set_session_id(self.session_id.clone());
        self.players[1] = Some(player2);
    }

    pub fn get_session_id(&self) -> SessionID { self.session_id.clone() }

    pub fn get_player1_name(&self) -> String { self.players[1].clone().unwrap().get_username() }
}

pub async fn start(session: Session<impl Game + Clone>) {
    let (tx, mut rx) = mpsc::channel::<String>(1);
    let database = PostgresDB::new("db_url").await;
    let dao = DAO::new(database);
    let session = Arc::new(RwLock::new(session));

    let dao_filter = warp::any().map(move || dao.clone());
    let domain_filter = warp::any()
        .and(warp::path(format!("{}{}", "xogamedev/", session.read().await.session_id.0)));

    //S ession cause the compiler error todo
    let session_filter = warp::any().map(move || session.clone());
    let shutdown_filter = warp::any().map(move || tx.clone());

    // let dao_filter = warp::any();
    // let session_filter = warp::any();
    // let shutdown_filter = warp::any();

    // let bruh_filter = warp::post()
    //     .and(domain_filter.clone())
    //     .and(shutdown_filter.clone())
    //     .and(dao_filter.clone())
    //     .and_then(move |_, _| async { return Ok(warp::reply::with_status("", StatusCode::OK)); });

    // let make_a_move_filter = warp::post()
    //     .and(warp::path("make_a_move"))
    //     .and(session_filter.clone())
    //     .and(warp::query())
    //     .and(warp::body::json())
    //     .and(shutdown_filter.clone())
    //     .and(dao_filter.clone())
    //     .and(warp::path::end())
    //     .and_then(handle_make_a_move);

    // let wait_for_move_filter = warp::post()
    //     .and(domain_filter.clone())
    //     .and(warp::path::end())
    //     .and(session_filter.clone())
    //     .and(warp::body::json())
    //     .and_then(handle_wait_for_move);
    //
    // let surrender_filter = warp::post()
    //     .and(domain_filter.clone())
    //     .and(warp::path("surrender"))
    //     .and(warp::path::end())
    //     .and(session_filter.clone())
    //     .and(warp::body::json())
    //     .and(shutdown_filter.clone())
    //     .and(dao_filter.clone())
    //     .and_then(handle_surrender);

    // let filter = make_a_move_filter
    //     // .or(wait_for_move_filter)
    //     // .or(surrender_filter)
    //     .recover(handle_error);
    //
    // warp::serve(filter)
    //     .run(([127, 0, 0, 1], 1337))
    //     .await;

    // warp::serve(filter).bind_with_graceful_shutdown(([127, 0, 0, 1], 1337), || async { rx.try_recv() });
}

// async fn handle_make_a_move(params: HashMap<String, String>, player: Player, tx: mpsc::Sender<String>, dao: DAO<impl Database>) -> Result<impl warp::Reply, warp::Rejection> {
//     if true {
//         return Ok(warp::reply::with_status("status.to_string()", StatusCode::OK))
//     }
//     return Err(warp::reject::custom(Error::Unauthorized));
// }

async fn handle_make_a_move(session_arc: Arc<RwLock<Session<impl Game + Clone>>>, params: HashMap<String, String>, player: Player, tx: mpsc::Sender<String>, dao: DAO<impl Database>) -> Result<impl warp::Reply, warp::Rejection> {
    if !auth(session_arc.clone(), player).await {
        return Err(warp::reject::custom(Error::Unauthorized));
    }

    match params.get("move") {
        Some(value) => {
            let mut status = 0;
            {
                let mut session = session_arc.write().await;
                status = session.game.make_a_move(value.parse::<usize>().unwrap());
                session.turn = (session.turn + 1) % 2;
            }

            match status {
                1..=3 => {
                    save_and_shutdown(session_arc.clone(), tx, status, dao).await;
                    Ok(warp::reply::with_status(status.to_string(), StatusCode::OK))
                }
                _ => Ok(warp::reply::with_status(session_arc.read().await.game.print(), StatusCode::OK))
            }
        }
        None => Err(warp::reject::custom(Error::AuthenticationFail))
    }
}

async fn handle_wait_for_move(session: Arc<RwLock<Session<impl Game + Clone>>>, player: Player) -> Result<impl warp::Reply, warp::Rejection> {
    let session = session.read().await;
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

async fn handle_surrender(session_arc: Arc<RwLock<Session<impl Game + Clone>>>, player: Player, tx: mpsc::Sender<String>, dao: DAO<impl Database>) -> Result<impl warp::Reply, warp::Rejection> {
    //let session = session_arc.clone().read().await;
    let session = session_arc.read().await;
    if session.players[0].clone().unwrap() == player {
        //Player 1 at index 0 surrendered so player 2 win
        save_and_shutdown(session_arc.clone(), tx, 2, dao).await;
        Ok(warp::reply::with_status("2", StatusCode::OK))
    } else if session.players[1].clone().unwrap() == player {
        //Player 2 at index 1 surrendered so player 1 win
        save_and_shutdown(session_arc.clone(), tx, 1, dao).await;
        Ok(warp::reply::with_status("1", StatusCode::OK))
    } else {
        Err(warp::reject::custom(Error::Unauthorized))
    }
}

async fn save_and_shutdown(session: Arc<RwLock<Session<impl Game + Clone>>>, tx: mpsc::Sender<String>, status: usize, dao: DAO<impl Database>) {
    //TODO connect to DB and do shutdown
    // dao.save_session(session, status).await;
    tx.send("shutdown server please".to_string()).await.unwrap();
}

async fn auth(session: Arc<RwLock<Session<impl Game + Clone>>>, player: Player) -> bool {
    let session = session.read().await;
    return session.players[session.turn].clone().unwrap() == player;
}