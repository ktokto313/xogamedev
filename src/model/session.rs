use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::SystemTime;
use serde::Deserialize;
use std::sync::mpsc;
use warp::Filter;
use warp::http::StatusCode;
use crate::dao::{DAO, Database};
use crate::dao::postgres::PostgresDB;
use crate::error::Error;
use crate::game::Game;
use crate::game::xo::XO;
use crate::handle_error;
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
        player2.set_session_id(self.session_id.clone());
        self.players[1] = Some(player2);
    }

    pub fn get_session_id(&self) -> SessionID { self.session_id.clone() }

    pub fn get_player1_name(&self) -> String { self.players[1].clone().unwrap().get_username() }

    pub async fn start(self) {
        let (tx, rx) = mpsc::channel();
        let database = PostgresDB::new("db_url").await;
        let dao = DAO::new(database);

        let dao_filter = warp::any().map(move || { dao.clone() });
        let domain_filter = warp::any()
            .and(warp::path(format!("{}{}", "xogamedev/", self.session_id.0)));
        let session_filter = warp::any().map(move || self.clone());
        let shutdown_filter = warp::any().map(|| tx.clone());
        let make_a_move_filter = warp::post()
            .and(warp::path("make_a_move"))
            .and(warp::path::end())
            .and(session_filter.clone())
            .and(warp::query())
            .and(warp::body::json())
            .and(shutdown_filter.clone())
            .and(dao_filter.clone())
            .and_then(Session::handle_make_a_move);

        let wait_for_move_filter = warp::post()
            .and(domain_filter.clone())
            .and(warp::path::end())
            .and(session_filter.clone())
            .and(warp::body::json())
            .and_then(Self::handle_wait_for_move);

        let surrender_filter = warp::post()
            .and(domain_filter.clone())
            .and(warp::path("surrender"))
            .and(warp::path::end())
            .and(session_filter.clone())
            .and(warp::body::json())
            .and(shutdown_filter.clone())
            .and(dao_filter.clone())
            .and_then(Self::handle_surrender);

        let filter = make_a_move_filter
            .or(make_a_move_filter)
            .or(wait_for_move_filter)
            .recover(handle_error);

        // warp::serve(filter)
        //     .run(([127, 0, 0, 1], 1337))
        //     .await;

        warp::serve(filter).bind_with_graceful_shutdown(([127, 0, 0, 1], 1337), rx.recv().ok());
    }
    async fn handle_make_a_move(mut self, params: HashMap<String, String>, player: Player, tx: mpsc::Sender<String>, dao: DAO<impl Database>) -> Result<impl warp::Reply, warp::Rejection> {
        if self.auth(player) {
            match params.get("move") {
                Some(value) => {
                    let status = self.game.make_a_move(value.parse::<usize>().unwrap());
                    if self.turn == 0 {
                        self.turn = 1
                    } else {
                        self.turn = 0
                    }
                    return match status {
                        1..=3 => {
                            self.save_and_shutdown(tx, status, dao).await;
                            Ok(warp::reply::with_status(status.to_string(), StatusCode::OK))
                        }
                        _ => Ok(warp::reply::with_status(self.game.print(), StatusCode::OK))
                    };
                }
                None => return Err(warp::reject::custom(Error::BadRequest))
            }
        }
        Err(warp::reject::custom(Error::Unauthorized))
    }

    fn handle_wait_for_move(mut self, player: Player) -> Result<impl warp::Reply, warp::Rejection> {
        if self.players[0].clone().unwrap() == player {
            if self.turn == 0 {
                Ok(warp::reply::with_status(true.to_string(), StatusCode::OK))
            } else {
                Ok(warp::reply::with_status(false.to_string(), StatusCode::OK))
            }
        } else if self.players[1].clone().unwrap() == player {
            if self.turn == 1 {
                Ok(warp::reply::with_status(true.to_string(), StatusCode::OK))
            } else {
                Ok(warp::reply::with_status(false.to_string(), StatusCode::OK))
            }
        } else {
            Err(warp::reject::custom(Error::Unauthorized))
        }
    }

    async fn handle_surrender(mut self, player: Player, tx: mpsc::Sender<String>, dao: DAO<impl Database>) -> Result<impl warp::Reply, warp::Rejection> {
        if self.players[0].clone().unwrap() == player {
            //Player 1 at index 0 surrendered so player 2 win
            self.save_and_shutdown(tx, 2, dao).await;
            Ok(warp::reply::with_status("2", StatusCode::OK))
        } else if self.players[1].clone().unwrap() == player {
            //Player 2 at index 1 surrendered so player 1 win
            self.save_and_shutdown(tx, 1, dao).await;
            Ok(warp::reply::with_status("1", StatusCode::OK))
        } else {
            Err(warp::reject::custom(Error::Unauthorized))
        }
    }

    async fn save_and_shutdown(mut self, tx: mpsc::Sender<String>, status: i32, dao: DAO<impl Database>) {
        //TODO connect to DB and do shutdown
        dao.save_session(self, status).await;
        tx.send("shutdown server please".to_string()).unwrap()
    }

    fn auth(&self, player: Player) -> bool {
        return self.players[self.turn].clone().unwrap() == player;
    }
}