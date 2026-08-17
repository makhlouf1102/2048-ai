pub mod board;
pub mod controller;
pub mod tile;
mod trained_model;
pub mod types;

use crate::board::*;
use crate::tile::CELL_COUNT;
use crate::types::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn next_move(flatten_board: &[u32]) -> i8 {
    if flatten_board.len() != CELL_COUNT {
        return -1;
    }

    let board = Board::new(flatten_board);
    let moves = board.available_moves();

    if moves.is_empty() {
        return -1;
    }

    let best_move = trained_model::best_move(&board);

    match best_move {
        Direction::Down => 0,
        Direction::Left => 1,
        Direction::Right => 2,
        Direction::Up => 3,
    }
}
