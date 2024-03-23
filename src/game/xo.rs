use serde::{Deserialize, Serialize};
use crate::game::Game;

//TODO we need to standardize games so we can have multiple game implementation
//A game module need player1 and player2 input stream, then someway to start, stop and get result
#[derive(Clone, Serialize, Deserialize)]
pub struct XO {
    #[serde(borrow)]
    board: [[&'static str;3];3],
    number_of_move: i32,
    is_x: bool
}

impl XO {
    pub fn new() -> XO {
        XO {
            board: [[" ";3];3],
            number_of_move: 0,
            is_x: true
        }
    }

    fn check(array: [[&str; 3]; 3], row: usize, col: usize) -> bool {
        let mut bool = true;

        //check top left to bottom right
        if row == col {
            //check diagonal
            for i in 0..3 {
                if array[i][i] != array[row][col] {
                    bool = false;
                    break;
                }
            }
            if bool {
                return true;
            }
        }

        //check top right to bottom left
        if row == 2 - col {
            bool = true;
            for i in 0..3 {
                if array[i][2 - i] != array[row][col] {
                    bool = false;

                    break;
                }
            }
            if bool {
                return true;
            }
        }

        //check column
        bool = true;
        for i in 0..3 {
            if array[i][col] != array[row][col] {
                bool = false;
                break;
            }
        }
        if bool {
            return true;
        }

        //check row
        bool = true;
        for i in 0..3 {
            if array[row][i] != array[row][col] {
                bool = false;
                break;
            }
        }
        if bool {
            return true;
        }
        false
    }
}

impl Game for XO {

    //*
    // Main function of the XO return 1 if player 1 win, 2 if player 2 win, 0 if nothing happened
    // 3 if draw
    // */
    fn make_a_move(&mut self, player_input: usize) -> usize {
        if self.number_of_move == 9 {
            return 3;
        }


        //trim input to fit our calculation
        let trim = player_input - 1;
        let row = trim / 3;
        let col = trim % 3;
        if self.board[row][col] == " " {
            if self.is_x {
                self.is_x = false;
                self.board[row][col] = "X";
            } else {
                self.is_x = true;
                self.board[row][col] = "O";
            }
            self.number_of_move += 1;
        }
        if Self::check(self.board, row, col) {
            //X is the first player that move, and here is_x flipped, so it should be !is_x
            if !self.is_x {
                return 1;
            } else {
                return 2;
            }
        }
        0
    }

    fn print(&self) -> String {
        let mut board = String::new();
        for i in self.board {
            for j in i {
                board.push_str( & * format ! ("{}|", j))
            }
            board.push_str("\n______\n");
        }
        board
    }
}