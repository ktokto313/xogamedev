use warp::reject::Reject;

#[derive(Debug)]
pub enum Error {
    InvalidMove,
    BadRequest,
    Unauthorized,
    DatabaseError(sqlx::Error)
}

impl Reject for Error {}