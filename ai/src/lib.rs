use wasm_bindgen::prelude::*;

const SIZE: usize = 4;

type Tile = i32;
type Row = [Tile; SIZE];
type Board = [Row; SIZE];

enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[wasm_bindgen]
pub fn next_move(flatten_board: &[i32]) -> i8 {
    let board = get_board(flatten_board);
    if !can_move(&board) {
        return -1;
    }

    // create different instances 
    

    (flatten_board[0] % 4) as i8
}

fn get_board(flatten_board: &[i32]) -> Board {
    let mut board = [[0; SIZE]; SIZE];

    for row in 0..SIZE {
        for col in 0..SIZE {
            board[row][col] = flatten_board[row * SIZE + col];
        }
    }

    board
}

fn compress_row(row: &mut Row) {
    let mut write = 0;

    for read in 0..row.len() {
        if row[read] != 0 {
            row[write] = row[read];

            if write != read {
                row[read] = 0;
            }

            write += 1;
        }
    }
}

fn merge(row: &mut Row) {
    compress_row(row);

    for i in 0..row.len() - 1 {
        if row[i] != 0 && row[i] == row[i + 1] {
            row[i] *= 2;
            row[i + 1] = 0;
        }
    }

    compress_row(row);
}

fn can_move(board: &Board) -> bool {
    for i in 0..4 {
        for j in 0..4 {
            // Empty cell
            if board[i][j] == 0 {
                return true;
            }

            // Compare with cell on the right
            if j < 3 && board[i][j] == board[i][j + 1] {
                return true;
            }

            // Compare with cell below
            if i < 3 && board[i][j] == board[i + 1][j] {
                return true;
            }
        }
    }

    false
}
