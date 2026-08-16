use std::collections::HashMap;

use crate::{
    board::{Board, IBoard},
    tile::{CELL_COUNT, SIZE},
    types::Direction,
};

pub trait IController {
    fn new(board: Board, depth: usize) -> Self;
    fn best_move(&self) -> Direction;
}

pub struct Controller {
    board: Board,
    depth: usize,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum NodeKind {
    Player,
    Chance,
}

type CacheKey = ([u8; CELL_COUNT], u32, usize, NodeKind);

// Space is the strongest survival signal in 2048. One additional empty cell
// must be worth more than a small immediate merge or a minor shape improvement.
const EMPTY_TILE_WEIGHT: f64 = 1_200.0;

impl IController for Controller {
    fn new(board: Board, depth: usize) -> Self {
        Self {
            board,
            depth: depth.max(1),
        }
    }

    fn best_move(&self) -> Direction {
        let candidates = self.board.available_moves();
        let mut cache = HashMap::new();
        let mut best_direction = candidates[0];
        let mut best_value = f64::NEG_INFINITY;

        for direction in candidates {
            let moved = self.board.make_move(direction);
            let value = chance_value(&moved, self.depth - 1, &mut cache);

            if value > best_value {
                best_value = value;
                best_direction = direction;
            }
        }

        best_direction
    }
}

fn player_value(board: &Board, depth: usize, cache: &mut HashMap<CacheKey, f64>) -> f64 {
    if depth == 0 {
        return evaluate_board(board);
    }

    let key = cache_key(board, depth, NodeKind::Player);
    if let Some(value) = cache.get(&key) {
        return *value;
    }

    let moves = board.available_moves();
    let value = if moves.is_empty() {
        evaluate_board(board) - 1_000_000.0
    } else {
        moves
            .into_iter()
            .map(|direction| chance_value(&board.make_move(direction), depth - 1, cache))
            .fold(f64::NEG_INFINITY, f64::max)
    };

    cache.insert(key, value);
    value
}

fn chance_value(board: &Board, depth: usize, cache: &mut HashMap<CacheKey, f64>) -> f64 {
    let key = cache_key(board, depth, NodeKind::Chance);
    if let Some(value) = cache.get(&key) {
        return *value;
    }

    let empty = board.empty_tiles();
    if empty.is_empty() {
        return player_value(board, depth, cache);
    }

    let cell_probability = 1.0 / empty.len() as f64;
    let value = empty
        .into_iter()
        .map(|(row, col)| {
            let with_two = board.with_rank(row, col, 1);
            let with_four = board.with_rank(row, col, 2);
            cell_probability
                * (0.9 * player_value(&with_two, depth, cache)
                    + 0.1 * player_value(&with_four, depth, cache))
        })
        .sum();

    cache.insert(key, value);
    value
}

fn cache_key(board: &Board, depth: usize, kind: NodeKind) -> CacheKey {
    let mut cells = [0; CELL_COUNT];
    for (index, value) in board.matrix().iter().flatten().copied().enumerate() {
        cells[index] = value;
    }
    (cells, board.score, depth, kind)
}

/// Stable leaf evaluation. Tile comparisons use base-2 ranks so a large tile
/// cannot numerically swamp all of the structural features of the position.
fn evaluate_board(board: &Board) -> f64 {
    let matrix = board.matrix();
    let empty = board.empty_tiles().len() as f64;
    let mut merge_pairs: f64 = 0.0;
    let mut roughness: f64 = 0.0;
    let mut row_up: f64 = 0.0;
    let mut row_down: f64 = 0.0;
    let mut col_up: f64 = 0.0;
    let mut col_down: f64 = 0.0;

    for row in 0..SIZE {
        for col in 0..SIZE {
            let current = tile_rank(matrix[row][col]);

            if col + 1 < SIZE {
                let neighbor = tile_rank(matrix[row][col + 1]);
                if current > 0.0 && current == neighbor {
                    merge_pairs += 1.0;
                }
                if current > 0.0 && neighbor > 0.0 {
                    roughness += (current - neighbor).abs();
                    row_up += (current - neighbor).max(0.0);
                    row_down += (neighbor - current).max(0.0);
                }
            }

            if row + 1 < SIZE {
                let neighbor = tile_rank(matrix[row + 1][col]);
                if current > 0.0 && current == neighbor {
                    merge_pairs += 1.0;
                }
                if current > 0.0 && neighbor > 0.0 {
                    roughness += (current - neighbor).abs();
                    col_up += (current - neighbor).max(0.0);
                    col_down += (neighbor - current).max(0.0);
                }
            }
        }
    }

    let monotonicity = row_up.max(row_down) + col_up.max(col_down);
    let maximum = matrix.iter().flatten().copied().max().unwrap_or(0);
    let corner = if maximum > 0
        && [
            matrix[0][0],
            matrix[0][SIZE - 1],
            matrix[SIZE - 1][0],
            matrix[SIZE - 1][SIZE - 1],
        ]
        .contains(&maximum)
    {
        tile_rank(maximum)
    } else {
        0.0
    };

    f64::from(board.score)
        + empty * EMPTY_TILE_WEIGHT
        + merge_pairs * 80.0
        + monotonicity * 35.0
        + corner * 120.0
        - roughness * 25.0
}

fn tile_rank(tile: u8) -> f64 {
    f64::from(tile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluation_rewards_empty_space() {
        let open = Board::new(&[2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let crowded = Board::new(&[2, 4, 8, 16, 32, 64, 128, 256, 4, 8, 16, 32, 8, 16, 32, 64]);

        assert!(evaluate_board(&open) > evaluate_board(&crowded));
    }

    #[test]
    fn one_empty_tile_has_a_large_survival_value() {
        let with_space = Board::new(&[2, 4, 8, 16, 4, 8, 16, 32, 8, 16, 32, 64, 4, 8, 0, 128]);
        let full = Board::new(&[2, 4, 8, 16, 4, 8, 16, 32, 8, 16, 32, 64, 4, 8, 64, 128]);

        assert!(evaluate_board(&with_space) - evaluate_board(&full) > 500.0);
    }

    #[test]
    fn evaluation_rewards_a_maximum_tile_in_a_corner() {
        let corner = Board::new(&[128, 64, 32, 16, 64, 32, 16, 8, 32, 16, 8, 4, 16, 8, 4, 2]);
        let center = Board::new(&[64, 32, 16, 8, 32, 128, 8, 4, 16, 8, 4, 2, 8, 4, 2, 4]);

        assert!(evaluate_board(&corner) > evaluate_board(&center));
    }

    #[test]
    fn expectimax_is_deterministic() {
        let cells = [2, 4, 8, 16, 4, 8, 16, 32, 2, 4, 8, 16, 0, 2, 4, 8];
        let first = Controller::new(Board::new(&cells), 2).best_move();
        let second = Controller::new(Board::new(&cells), 2).best_move();

        assert_eq!(first, second);
    }
}
