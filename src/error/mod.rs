use warp::reject::Reject;

#[derive(Debug)]
pub enum Error {
    InvalidMove,
    AuthenticationFail,
    SessionNotExist,
    Unauthorized,
    DatabaseError(sqlx::Error)
}

impl Reject for Error {}