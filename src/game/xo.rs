use std::io;
use crate::game::Game;

//TODO we need to standardize games so we can have multiple game implementation
//A game module need player1 and player2 input stream, then someway to start, stop and get result
pub struct XO {

}

impl XO {
    pub(crate) fn new() -> XO {
        XO {}
    }
}

impl Game for XO {
}

pub fn start_game() {
    let mut array = [[" "; 3]; 3];

    //May x_moves and o_moves doesn't have any use right now, I think it would be good for a replay system
    //let x_moves = Vec::<usize>::new();
    //let o_moves = Vec::<usize>::new();

    let mut number_of_move = 0;
    let mut is_x = true;

    loop {
        print_board(array);

        if number_of_move == 9 {
            println!("Draw");
            break;
        }

        //begin: std input for testing
        println!("Please input your move");
        let mut string = String::new();
        io::stdin().read_line(&mut string).unwrap();
        //trim input to fit our calculation
        let trim = string.trim().parse::<usize>().unwrap() - 1;
        let row = trim / 3;
        let col = trim % 3;
        if array[row][col] == " " {
            if is_x {
                is_x = false;
                array[row][col] = "X";
            } else {
                is_x = true;
                array[row][col] = "O";
            }
            number_of_move += 1;
        } else {
            println!("Wrong input, please make a valid move!");
            continue;
        }
        if check(array, row, col) {
            //X is the first player that move, and here is_x flipped, so it should be !is_x
            if !is_x {
                print_board(array);
                println!("1");
            } else {
                print_board(array);
                println!("2");
            }
            break;
        }
        //end of testing
    };
}

fn print_board(array: [[&str; 3]; 3]) {
    for i in array {
        for j in i {
            print!("{}|", j);
        }
        println!("\n______");
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
            println!("Top left to bottom right win");
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
            println!("Top right to bottom left win");
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
        println!("Column win");
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
        println!("Row win");
        return true;
    }
    false
}