use crate::{
    board::{Board, IBoard},
    types::Direction,
};

pub trait IController {
    fn new(board: Board, depth: usize) -> Self;
    fn run_simulation(&self) -> Direction;
    fn simulate(&self, board: &mut Board, depth: usize) -> u64;
}

pub struct Controller {
    board: Board,
    depth: usize,
}

impl IController for Controller {
    fn new(board: Board, depth: usize) -> Self {
        Controller { board, depth }
    }

    fn run_simulation(&self) -> Direction {
        // get all the possible next moves
        let candidates: Vec<Direction> = self.board.available_moves();
        let mut max_score = evaluate_board(&self.board);
        let mut best_direction: Direction = candidates[0];

        for direction in candidates.iter() {
            let mut board_copy = self.board.make_move(*direction);
            let new_score = self.simulate(&mut board_copy, self.depth);
            if new_score > max_score {
                max_score = new_score;
                best_direction = *direction;
            }
        }

        best_direction
    }

    fn simulate(&self, board: &mut Board, depth: usize) -> u64 {
        if depth == 0 {
            return evaluate_board(board);
        }
        board.set_empty_tile();
        let candidates: Vec<Direction> = board.available_moves();
        let mut max_score = evaluate_board(board);

        for direction in candidates.iter() {
            let mut board_copy = board.make_move(*direction);
            let new_score = self.simulate(&mut board_copy, depth - 1);
            if new_score > max_score {
                max_score = new_score;
            }
        }

        max_score
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
