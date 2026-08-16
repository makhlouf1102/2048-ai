pub mod board;
pub mod controller;
pub mod tile;
pub mod types;

use crate::board::*;
use crate::controller::*;
use crate::tile::CELL_COUNT;
use crate::types::*;
use wasm_bindgen::prelude::*;

const SIMULATION_DEPTH: usize = 5;

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

    let controller = Controller::new(board, SIMULATION_DEPTH);
    let best_move = controller.run_simulation();

    match best_move {
        Direction::Down => 0,
        Direction::Left => 1,
        Direction::Right => 2,
        Direction::Up => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_an_invalid_board_length() {
        assert_eq!(next_move(&[0; 15]), -1);
    }

    #[test]
    fn returns_minus_one_when_no_move_exists() {
        let board = [2, 4, 2, 4, 4, 2, 4, 2, 2, 4, 2, 4, 4, 2, 4, 2];
        assert_eq!(next_move(&board), -1);
    }

    #[test]
    fn returned_direction_uses_the_browser_mapping() {
        let board = [2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert!((0..=3).contains(&next_move(&board)));
    }
}
