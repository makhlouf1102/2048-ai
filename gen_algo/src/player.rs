use crate::{
    game_2048::{CELL_COUNT, Direction, Game},
    neural_network::NeuralNetwork,
};
use log::debug;

pub const FITNESS_GAMES: usize = 5;
pub const INVALID_MOVE_PENALTY: f32 = 100.0;
const NETWORK_LAYOUT: [usize; 4] = [CELL_COUNT, 32, 16, 4];
const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];

/// One individual that can play 2048 using its neural network as a brain.
#[derive(Debug, Clone)]
pub struct Player {
    brain: NeuralNetwork,
    game: Game,
    invalid_moves: u32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            brain: NeuralNetwork::new(&NETWORK_LAYOUT),
            game: Game::new(),
            invalid_moves: 0,
        }
    }

    pub fn from_brain(brain: NeuralNetwork) -> Self {
        Self {
            brain,
            game: Game::new(),
            invalid_moves: 0,
        }
    }

    pub fn brain(&self) -> &NeuralNetwork {
        &self.brain
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn invalid_moves(&self) -> u32 {
        self.invalid_moves
    }

    pub fn mutate(&mut self, mutation_rate: f32, mutation_strength: f32) {
        self.brain.mutate(mutation_rate, mutation_strength);
    }

    /// Resets the board, plays until no legal moves remain, and returns the score.
    pub fn play_game(&mut self) -> u32 {
        self.game = Game::new();
        self.invalid_moves = 0;

        while !self.game.is_game_over() {
            let preferred_move = self.choose_move(false).expect("brain has four outputs");

            if !self.game.make_move(preferred_move) {
                self.invalid_moves += 1;

                // Keep the simulation moving after penalizing the bad decision.
                let legal_fallback = self
                    .choose_move(true)
                    .expect("a non-finished game must have a legal move");
                self.game.make_move(legal_fallback);
            }
        }

        let score = self.game.score();
        debug!(
            "game finished: score={score}, invalid_moves={}",
            self.invalid_moves
        );
        score
    }

    /// Plays five independent games and averages their scores after subtracting
    /// a penalty for every illegal move selected by the brain.
    pub fn fitness(&mut self) -> f32 {
        let mut total = 0.0;
        for game_number in 1..=FITNESS_GAMES {
            let score = self.play_game() as f32;
            let penalty = self.invalid_moves as f32 * INVALID_MOVE_PENALTY;
            let adjusted_score = score - penalty;
            debug!(
                "fitness game {game_number}/{FITNESS_GAMES}: raw={score:.2}, penalty={penalty:.2}, adjusted={adjusted_score:.2}"
            );
            total += adjusted_score;
        }

        let fitness = total / FITNESS_GAMES as f32;
        debug!("player fitness={fitness:.2}");
        fitness
    }

    fn choose_move(&self, legal_only: bool) -> Option<Direction> {
        let input = encode_board(&self.game);
        let output = self.brain.forward(&input);

        assert_eq!(
            output.len(),
            DIRECTIONS.len(),
            "Player brain must have four output neurons"
        );

        best_move(output.as_slice(), &self.game, legal_only)
    }
}

fn best_move(output: &[f32], game: &Game, legal_only: bool) -> Option<Direction> {
    DIRECTIONS
        .into_iter()
        .enumerate()
        .filter(|(_, direction)| !legal_only || game.can_move(*direction))
        .max_by(|(left, _), (right, _)| output[*left].total_cmp(&output[*right]))
        .map(|(_, direction)| direction)
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

/// Tile ranks keep large tile values from overwhelming the network inputs.
/// Dividing by 16 keeps typical 2048 boards close to the network's activation range.
fn encode_board(game: &Game) -> [f32; CELL_COUNT] {
    game.flattened_board().map(|tile| {
        if tile == 0 {
            0.0
        } else {
            tile.ilog2() as f32 / 16.0
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_encoding_uses_normalized_tile_ranks() {
        let game = Game::from_board([[0, 2, 4, 8], [16, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);

        let encoded = encode_board(&game);
        assert_eq!(encoded[0], 0.0);
        assert_eq!(encoded[1], 1.0 / 16.0);
        assert_eq!(encoded[2], 2.0 / 16.0);
        assert_eq!(encoded[3], 3.0 / 16.0);
        assert_eq!(encoded[4], 4.0 / 16.0);
    }

    #[test]
    fn illegal_preference_is_visible_and_has_a_legal_fallback() {
        let game = Game::from_board([[2, 4, 8, 16], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]);
        let output = [0.0, 0.0, 0.5, 1.0];

        assert_eq!(best_move(&output, &game, false), Some(Direction::Left));
        assert_eq!(best_move(&output, &game, true), Some(Direction::Down));
    }
}
