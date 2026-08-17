pub mod board;
pub mod controller;
pub mod random_simulation;
pub mod tile;
mod trained_model;
pub mod types;

use crate::board::*;
use crate::controller::{Controller, IController};
use crate::tile::CELL_COUNT;
use crate::types::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn next_move(flatten_board: &[u32]) -> i8 {
    next_move_model(flatten_board)
}

/// Chooses a move using the depth-based expectimax AI.
#[wasm_bindgen]
pub fn next_move_depth(flatten_board: &[u32], depth: usize) -> i8 {
    choose_move(flatten_board, |board| {
        Controller::new(board.clone(), depth).best_move()
    })
}

/// Chooses a move using the embedded trained model.
#[wasm_bindgen]
pub fn next_move_model(flatten_board: &[u32]) -> i8 {
    choose_move(flatten_board, trained_model::best_move)
}

/// Chooses a move by averaging 100 complete games of random play per move.
#[wasm_bindgen]
pub fn next_move_random_simulation(flatten_board: &[u32]) -> i8 {
    next_move_random_simulation_with_runs(flatten_board, random_simulation::DEFAULT_RUN_COUNT)
}

/// Random simulation with a configurable number of games per available move.
/// Higher run counts are slower but generally produce a stronger decision.
#[wasm_bindgen]
pub fn next_move_random_simulation_with_runs(flatten_board: &[u32], run_count: usize) -> i8 {
    choose_move(flatten_board, |board| {
        random_simulation::best_move(board, run_count)
    })
}

fn choose_move(flatten_board: &[u32], select_move: impl FnOnce(&Board) -> Direction) -> i8 {
    if flatten_board.len() != CELL_COUNT {
        return -1;
    }

    let board = Board::new(flatten_board);
    if board.available_moves().is_empty() {
        return -1;
    }

    match select_move(&board) {
        Direction::Down => 0,
        Direction::Left => 1,
        Direction::Right => 2,
        Direction::Up => 3,
    }
}
