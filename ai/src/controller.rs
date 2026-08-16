use crate::{
    board::{Board, IBoard},
    types::Direction,
};

pub trait IController {
    fn new(board: Board, depth: usize) -> Self;
    fn run_simulation(&self) -> Direction;
    fn simulate(&self, board: &mut Board, depth: usize) -> u32;
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
        let mut max_score = self.board.score;
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

    fn simulate(&self, board: &mut Board, depth: usize) -> u32 {
        if depth == 0 {
            return board.score
        }
        board.set_empty_tile();
        let candidates: Vec<Direction> = board.available_moves();
        let mut max_score = board.score;

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
