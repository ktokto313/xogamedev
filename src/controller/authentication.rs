use warp::http::StatusCode;
use crate::model::player::Player;
use crate::{DAO};
use crate::error::Error;

fn register(player: Player, dao: DAO) -> Result<impl warp::Reply, warp::Rejection> {
    match dao.register(player) {
        Ok(_) => Ok(warp::reply::with_status("Operation completed successfully", StatusCode::OK)),
        Err(_) => Err(warp::reject::custom(Error::DuplicatedUsername))
    }
}

fn login(player: Player, dao: DAO) -> Result<impl warp::Reply, warp::Rejection> {
    match dao.login(player) {
        Ok(_) => Ok(warp::reply::with_status("Operation completed successfully", StatusCode::OK)),
        Err(_) => Err(warp::reject::custom(Error::CanNotLogin))
    }
}