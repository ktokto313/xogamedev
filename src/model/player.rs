use crate::model::session::SessionID;

pub struct Player {
    username: String,
    password: String,
    pub session_id: Option<SessionID>
}

impl Player {
    fn new(username: String, password: String) -> Player {
        Player {
            username,
            password,
            session_id: None
        }
    }

    pub fn set_session_id(&mut self, session_id: SessionID) {
        self.session_id = Some(session_id)
    }

    pub fn get_username(&self) -> String {self.username.clone()}
}