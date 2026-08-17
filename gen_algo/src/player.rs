use crate::{
    game_2048::{CELL_COUNT, Direction, Game},
    neural_network::NeuralNetwork,
};
use log::debug;

pub const FITNESS_GAMES: usize = 10;
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
        self.play_game_with_seed(rand::random())
    }

    pub fn play_game_with_seed(&mut self, seed: u64) -> u32 {
        self.game = Game::from_seed(seed);
        self.invalid_moves = 0;

        while !self.game.is_game_over() {
            let preferred_move = self.choose_move();

            if !self.game.make_move(preferred_move) {
                self.invalid_moves += 1;

                debug!("game stopped after the brain selected an invalid move");
                break;
            }
        }

        let score = self.game.score();
        debug!(
            "game finished: score={score}, invalid_moves={}",
            self.invalid_moves
        );
        score
    }

    /// Plays ten independent games and averages their raw 2048 scores.
    /// An invalid choice ends its game at the score earned so far.
    pub fn fitness(&mut self) -> f32 {
        let seeds: [u64; FITNESS_GAMES] = std::array::from_fn(|_| rand::random());
        self.fitness_with_seeds(&seeds)
    }

    pub fn fitness_with_seeds(&mut self, seeds: &[u64]) -> f32 {
        assert!(!seeds.is_empty(), "fitness requires at least one game seed");
        let mut total = 0.0;
        for (index, seed) in seeds.iter().enumerate() {
            let game_number = index + 1;
            let score = self.play_game_with_seed(*seed) as f32;
            debug!(
                "fitness game {game_number}/{}: score={score:.2}, invalid_moves={}",
                seeds.len(),
                self.invalid_moves
            );
            total += score;
        }

        let fitness = total / seeds.len() as f32;
        debug!("player fitness={fitness:.2}");
        fitness
    }

    fn choose_move(&self) -> Direction {
        let input = encode_board(&self.game);
        let output = self.brain.forward(&input);

        assert_eq!(
            output.len(),
            DIRECTIONS.len(),
            "Player brain must have four output neurons"
        );

        best_move(output.as_slice())
    }
}

fn best_move(output: &[f32]) -> Direction {
    DIRECTIONS
        .into_iter()
        .enumerate()
        .max_by(|(left, _), (right, _)| output[*left].total_cmp(&output[*right]))
        .map(|(_, direction)| direction)
        .expect("the brain has four direction outputs")
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
    fn highest_output_is_selected_even_when_the_move_is_illegal() {
        let output = [0.0, 0.0, 0.5, 1.0];

        assert_eq!(best_move(&output), Direction::Left);
    }

    #[test]
    fn equal_brains_receive_equal_fitness_on_shared_seeds() {
        let mut first = Player::new();
        let mut second = first.clone();
        let seeds = [11, 22, 33, 44, 55];

        assert_eq!(
            first.fitness_with_seeds(&seeds),
            second.fitness_with_seeds(&seeds)
        );
    }
}
