use crate::{
    board::{Board, IBoard},
    tile::SIZE,
    types::Direction,
};

pub trait IController {
    fn new(board: Board, rollouts: usize) -> Self;
    fn run_simulation(&self) -> Direction;
    fn simulate(&self, board: Board) -> u64;
}

pub struct Controller {
    board: Board,
    rollouts: usize,
}

impl IController for Controller {
    fn new(board: Board, rollouts: usize) -> Self {
        Controller {
            board,
            rollouts: rollouts.max(1),
        }
    }

    fn run_simulation(&self) -> Direction {
        let candidates: Vec<Direction> = self.board.available_moves();
        let mut best_average = 0;
        let mut best_direction: Direction = candidates[0];

        for direction in candidates {
            let mut total = 0_u64;

            for _ in 0..self.rollouts {
                let board_copy = self.board.make_move(direction);
                total = total.saturating_add(self.simulate(board_copy));
            }

            let average = total / self.rollouts as u64;
            if average > best_average {
                best_average = average;
                best_direction = direction;
            }
        }

        best_direction
    }

    fn simulate(&self, mut board: Board) -> u64 {
        loop {
            board.set_empty_tile();

            let candidates = board.available_moves();
            if candidates.is_empty() {
                return u64::from(board.score);
            }

            // The rollout policy is greedy: make the move whose immediate
            // result leaves the strongest score-and-empty-space position.
            let mut next_boards = candidates
                .into_iter()
                .map(|direction| board.make_move(direction));
            let mut best_board = next_boards.next().expect("available moves cannot be empty");
            let mut best_score = evaluate_board(&best_board);

            for candidate in next_boards {
                let candidate_score = evaluate_board(&candidate);
                if candidate_score > best_score {
                    best_board = candidate;
                    best_score = candidate_score;
                }
            }

            board = best_board;
        }
    }
}

/// Rates a live position using score, space, merge opportunities, tile order,
/// neighboring-tile smoothness, and largest-tile corner placement.
fn evaluate_board(board: &Board) -> u64 {
    let score = u64::from(board.score);
    let magnitude = if score == 0 { 0 } else { score.ilog10() };
    let empty_weight = 100_u64.saturating_pow(magnitude);
    let shape = board_shape(board);

    // Shape is stored in tenths so one empty cell retains exactly the original
    // `empty_tiles * empty_weight` value while smaller signals remain possible.
    let shape_bonus = i128::from(empty_weight) * i128::from(shape) / 10;
    (i128::from(score) + shape_bonus).clamp(0, i128::from(u64::MAX)) as u64
}

fn board_shape(board: &Board) -> i64 {
    let matrix = board.matrix();
    let mut merge_pairs = 0_i64;
    let mut roughness = 0_i64;
    let mut row_increasing = 0_i64;
    let mut row_decreasing = 0_i64;
    let mut col_increasing = 0_i64;
    let mut col_decreasing = 0_i64;

    for row in 0..SIZE {
        for col in 0..SIZE {
            let current = tile_rank(matrix[row][col]);

            if col + 1 < SIZE {
                let neighbor = tile_rank(matrix[row][col + 1]);
                if current > 0 && current == neighbor {
                    merge_pairs += 1;
                }
                if current > 0 && neighbor > 0 {
                    roughness += (current - neighbor).abs();
                    row_increasing += (current - neighbor).max(0);
                    row_decreasing += (neighbor - current).max(0);
                }
            }

            if row + 1 < SIZE {
                let neighbor = tile_rank(matrix[row + 1][col]);
                if current > 0 && current == neighbor {
                    merge_pairs += 1;
                }
                if current > 0 && neighbor > 0 {
                    roughness += (current - neighbor).abs();
                    col_increasing += (current - neighbor).max(0);
                    col_decreasing += (neighbor - current).max(0);
                }
            }
        }
    }

    let monotonicity_penalty = row_increasing
        .min(row_decreasing)
        .saturating_add(col_increasing.min(col_decreasing));
    let corner_bonus = corner_bonus(board);

    board.empty_tiles().len() as i64 * 10 + merge_pairs * 6 + corner_bonus * 2
        - roughness
        - monotonicity_penalty
}

fn tile_rank(tile: u32) -> i64 {
    if tile == 0 { 0 } else { tile.ilog2() as i64 }
}

fn corner_bonus(board: &Board) -> i64 {
    let matrix = board.matrix();
    let maximum = matrix.iter().flatten().copied().max().unwrap_or(0);
    let corners = [
        matrix[0][0],
        matrix[0][SIZE - 1],
        matrix[SIZE - 1][0],
        matrix[SIZE - 1][SIZE - 1],
    ];

    if maximum > 0 && corners.contains(&maximum) {
        tile_rank(maximum)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_bonus_rewards_a_largest_tile_in_a_corner() {
        let corner = Board::new(&[128, 64, 32, 16, 64, 32, 16, 8, 32, 16, 8, 4, 16, 8, 4, 2]);
        let center = Board::new(&[64, 32, 16, 8, 32, 128, 8, 4, 16, 8, 4, 2, 8, 4, 2, 4]);

        assert!(corner_bonus(&corner) > corner_bonus(&center));
    }

    #[test]
    fn shape_rewards_merge_opportunities_and_empty_space() {
        let useful = Board::new(&[2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let crowded = Board::new(&[2, 4, 8, 16, 32, 64, 128, 256, 4, 8, 16, 32, 8, 16, 32, 64]);

        assert!(board_shape(&useful) > board_shape(&crowded));
    }

    #[test]
    fn controller_always_runs_at_least_one_rollout() {
        let board = Board::new(&[2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let controller = Controller::new(board, 0);

        assert_eq!(controller.rollouts, 1);
    }
}
