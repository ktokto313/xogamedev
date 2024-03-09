use warp::reject::Reject;

#[derive(Debug)]
pub enum Error {
    InvalidMove,
    DuplicatedUsername,
    CanNotLogin
}

impl Reject for Error {}