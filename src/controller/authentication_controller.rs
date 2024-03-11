use crate::dao::Database;
use warp::http::StatusCode;
use crate::model::player::Player;
use crate::dao::DAO;
use crate::error::Error;

pub async fn register(player: Player, dao: DAO<impl Database>) -> Result<impl warp::Reply, warp::Rejection> {
    match dao.register(player).await {
        Ok(true) => Ok(warp::reply::with_status("Operation completed successfully", StatusCode::OK)),
        Ok(false) => Err(warp::reject::custom(Error::BadRequest)),
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e))),
    }
}

pub async fn login(player: Player, dao: DAO<impl Database>) -> Result<impl warp::Reply, warp::Rejection> {
    match dao.login(player).await {
        Ok(true) => Ok(warp::reply::with_status("Operation completed successfully", StatusCode::OK)),
        Ok(false) => Err(warp::reject::custom(Error::BadRequest)),
        Err(e) => Err(warp::reject::custom(Error::DatabaseError(e)))
    }
}