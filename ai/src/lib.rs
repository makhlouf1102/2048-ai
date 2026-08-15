pub mod board;
pub mod controller;
pub mod tile;
pub mod types;

use crate::board::*;
use crate::controller::*;
use crate::types::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn next_move(flatten_board: &[u32]) -> i8 {
    let board = Board::new(flatten_board);
    let moves = board.available_moves();

    if moves.len() < 1 {
        return -1;
    }

    let mut boards: Vec<Board> = Vec::new();

    let mut best_score = 0;
    let mut best_index = 0;

    for i in 0..moves.len() {
        let new_board = board.make_move(moves[i]);

        if new_board.score > best_score {
            best_score = new_board.score;
            best_index = i;
        }

        boards.push(new_board);
    }

    let best_move = &moves[best_index];

    match best_move {
        Direction::Down => 0,
        Direction::Left => 1,
        Direction::Right => 2,
        Direction::Up => 3,
    }
}
