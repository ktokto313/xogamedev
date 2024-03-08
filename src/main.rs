mod game;
use log;
use crate::game::start_game;

fn main() {
    //TODO: XO game where slot is indexed from 1 to 9 from left to right and top to bottom1
    env_logger::init();

    start_game()
}


