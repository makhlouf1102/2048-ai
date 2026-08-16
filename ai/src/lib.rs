pub mod board;
pub mod controller;
pub mod tile;
pub mod types;

use crate::board::*;
use crate::controller::*;
use crate::tile::CELL_COUNT;
use crate::types::*;
use wasm_bindgen::prelude::*;

const BASE_SEARCH_DEPTH: usize = 3;
const DEEP_SEARCH_EMPTY_TILE_THRESHOLD: usize = 6;
const EMPTY_TILES_PER_DEPTH_INCREASE: usize = 2;
const MAX_SEARCH_DEPTH: usize = 6;

fn search_depth(board: &Board) -> usize {
    let empty_tiles = board.empty_tiles().len();
    let pressure = DEEP_SEARCH_EMPTY_TILE_THRESHOLD.saturating_sub(empty_tiles);
    let depth_increase = pressure.div_ceil(EMPTY_TILES_PER_DEPTH_INCREASE);

    (BASE_SEARCH_DEPTH + depth_increase).min(MAX_SEARCH_DEPTH)
}

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

    let depth = search_depth(&board);
    let controller = Controller::new(board, depth);
    let best_move = controller.best_move();

    match best_move {
        Direction::Down => 0,
        Direction::Left => 1,
        Direction::Right => 2,
        Direction::Up => 3,
    }
}
