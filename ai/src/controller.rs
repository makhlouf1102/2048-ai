use crate::board::{Board, IBoard};
use crate::tile::UTile;

const DEFAULT_BEAM_WIDTH: usize = 4;
const DEFAULT_SIMULATED_TILE_VALUE: UTile = 2;

pub trait IController {
    fn simulate(&self, board: &Board, depth: usize) -> Vec<Board>;
}

#[derive(Debug, Clone, Copy)]
pub struct Controller {
    beam_width: usize,
    simulated_tile_value: UTile,
}

impl Controller {
    pub fn new(beam_width: usize, simulated_tile_value: UTile) -> Self {
        Self {
            beam_width: beam_width.max(1),
            simulated_tile_value,
        }
    }

    fn keep_best(&self, boards: &mut Vec<Board>) {
        boards.sort_by(|left, right| right.score.cmp(&left.score));
        boards.truncate(self.beam_width);
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new(DEFAULT_BEAM_WIDTH, DEFAULT_SIMULATED_TILE_VALUE)
    }
}

impl IController for Controller {
    /// Recursively explores future boards while retaining only the
    /// highest-scoring candidates allowed by the configured beam width.
    fn simulate(&self, board: &Board, depth: usize) -> Vec<Board> {
        if depth == 0 {
            return vec![board.clone()];
        }

        let mut candidates = Vec::new();

        for (row, col) in board.empty_tiles() {
            let mut spawned_board = board.clone();
            if !spawned_board.set_empty_tile(row, col, self.simulated_tile_value) {
                continue;
            }

            for direction in spawned_board.available_moves() {
                candidates.push(spawned_board.make_move(direction));
            }
        }

        self.keep_best(&mut candidates);
        if candidates.is_empty() || depth == 1 {
            return if candidates.is_empty() {
                vec![board.clone()]
            } else {
                candidates
            };
        }

        let mut descendants = candidates
            .iter()
            .flat_map(|candidate| self.simulate(candidate, depth - 1))
            .collect::<Vec<_>>();
        self.keep_best(&mut descendants);
        descendants
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn starting_board() -> Board {
        Board::new(&[2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    }

    #[test]
    fn depth_zero_returns_the_unchanged_board() {
        let board = starting_board();
        let result = Controller::default().simulate(&board, 0);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].score, 0);
        assert_eq!(board.score, 0);
    }

    #[test]
    fn retains_at_most_the_configured_number_of_boards() {
        let controller = Controller::new(2, 2);
        let result = controller.simulate(&starting_board(), 1);

        assert!(!result.is_empty());
        assert!(result.len() <= 2);
        assert!(result.windows(2).all(|pair| pair[0].score >= pair[1].score));
    }

    #[test]
    fn recursively_simulates_to_the_requested_depth() {
        let result = Controller::default().simulate(&starting_board(), 2);

        assert!(!result.is_empty());
        assert!(result.len() <= DEFAULT_BEAM_WIDTH);
        assert!(result.iter().all(|board| board.score >= 4));
    }

    #[test]
    fn zero_beam_width_is_normalized_to_one() {
        let result = Controller::new(0, 2).simulate(&starting_board(), 1);
        assert_eq!(result.len(), 1);
    }
}
