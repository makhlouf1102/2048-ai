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

    #[test]
    fn search_depth_increases_progressively_as_space_disappears() {
        fn board_with_empty_tiles(count: usize) -> Board {
            let mut cells = [2; CELL_COUNT];
            cells[..count].fill(0);
            Board::new(&cells)
        }

        assert_eq!(search_depth(&board_with_empty_tiles(10)), 3);
        assert_eq!(search_depth(&board_with_empty_tiles(6)), 3);
        assert_eq!(search_depth(&board_with_empty_tiles(5)), 4);
        assert_eq!(search_depth(&board_with_empty_tiles(4)), 4);
        assert_eq!(search_depth(&board_with_empty_tiles(3)), 5);
        assert_eq!(search_depth(&board_with_empty_tiles(2)), 5);
        assert_eq!(search_depth(&board_with_empty_tiles(1)), 6);
    }

    #[test]
    fn deepest_search_returns_a_move_on_a_crowded_board() {
        let board = [2, 4, 8, 16, 4, 8, 16, 32, 8, 16, 32, 64, 2, 2, 4, 0];

        assert!((0..=3).contains(&next_move(&board)));
    }
}
