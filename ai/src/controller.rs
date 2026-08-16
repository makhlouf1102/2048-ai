use crate::{
    board::{Board, IBoard},
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
                return evaluate_board(&board);
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

/// Rewards both merge score and the room left for future moves.
///
/// This mirrors the original JavaScript heuristic:
/// `score + empty_tiles * 100^floor(log10(score))`.
/// A zero score uses a weight of one so the evaluation remains defined.
fn evaluate_board(board: &Board) -> u64 {
    let score = u64::from(board.score);
    let magnitude = if score == 0 { 0 } else { score.ilog10() };
    let empty_weight = 100_u64.saturating_pow(magnitude);

    score.saturating_add((board.empty_tiles().len() as u64).saturating_mul(empty_weight))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluation_rewards_empty_tiles_with_a_score_dependent_weight() {
        let mut board = Board::new(&[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        board.score = 999;

        assert_eq!(evaluate_board(&board), 999 + 15 * 10_000);
    }

    #[test]
    fn controller_always_runs_at_least_one_rollout() {
        let board = Board::new(&[2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let controller = Controller::new(board, 0);

        assert_eq!(controller.rollouts, 1);
    }
}
