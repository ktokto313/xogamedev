pub mod xo;


pub trait Game {
    //*
    // Main function of the XO return 1 if player 1 win, 2 if player 2 win, 0 if nothing happened
    // 3 if draw
    // */
    fn make_a_move(&mut self, player_input: usize) -> i32;
    fn print(&self) -> String;
}