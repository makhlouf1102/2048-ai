use rand::RngExt;

pub const BOARD_SIZE: usize = 4;
pub const CELL_COUNT: usize = BOARD_SIZE * BOARD_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    board: [[u32; BOARD_SIZE]; BOARD_SIZE],
    score: u32,
    random_state: u64,
}

impl Game {
    /// Starts a normal 2048 game with two randomly placed tiles.
    pub fn new() -> Self {
        Self::from_seed(rand::rng().random())
    }

    /// Starts a reproducible game. Equal seeds produce equal tile sequences.
    pub fn from_seed(seed: u64) -> Self {
        let mut game = Self {
            board: [[0; BOARD_SIZE]; BOARD_SIZE],
            score: 0,
            random_state: seed,
        };
        game.spawn_random_tile();
        game.spawn_random_tile();
        game
    }

    /// Useful for tests, replays, and evaluating a network from a known state.
    pub fn from_board(board: [[u32; BOARD_SIZE]; BOARD_SIZE]) -> Self {
        Self {
            board,
            score: 0,
            random_state: 0,
        }
    }

    pub fn board(&self) -> &[[u32; BOARD_SIZE]; BOARD_SIZE] {
        &self.board
    }

    pub fn flattened_board(&self) -> [u32; CELL_COUNT] {
        let mut flattened = [0; CELL_COUNT];
        for (index, value) in self.board.iter().flatten().copied().enumerate() {
            flattened[index] = value;
        }
        flattened
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn empty_tiles(&self) -> Vec<(usize, usize)> {
        let mut empty = Vec::new();
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if self.board[row][col] == 0 {
                    empty.push((row, col));
                }
            }
        }
        empty
    }

    pub fn can_move(&self, direction: Direction) -> bool {
        let mut copy = self.clone();
        copy.apply_move(direction)
    }

    pub fn available_moves(&self) -> Vec<Direction> {
        [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ]
        .into_iter()
        .filter(|direction| self.can_move(*direction))
        .collect()
    }

    pub fn is_game_over(&self) -> bool {
        self.available_moves().is_empty()
    }

    /// Applies one turn. A random tile is spawned only after a legal move.
    pub fn make_move(&mut self, direction: Direction) -> bool {
        if !self.apply_move(direction) {
            return false;
        }
        self.spawn_random_tile();
        true
    }

    fn spawn_random_tile(&mut self) -> bool {
        let empty = self.empty_tiles();
        if empty.is_empty() {
            return false;
        }

        let position = self.next_random() as usize % empty.len();
        let (row, col) = empty[position];
        self.board[row][col] = if self.next_random() % 10 < 9 { 2 } else { 4 };
        true
    }

    /// SplitMix64 is small, fast, and sufficient for reproducible game simulation.
    fn next_random(&mut self) -> u64 {
        self.random_state = self.random_state.wrapping_add(0x9E3779B97F4A7C15);
        let mut value = self.random_state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D049BB133111EB);
        value ^ (value >> 31)
    }

    fn apply_move(&mut self, direction: Direction) -> bool {
        let previous = self.board;

        for index in 0..BOARD_SIZE {
            let line = match direction {
                Direction::Left => self.board[index],
                Direction::Right => {
                    std::array::from_fn(|offset| self.board[index][BOARD_SIZE - 1 - offset])
                }
                Direction::Up => std::array::from_fn(|offset| self.board[offset][index]),
                Direction::Down => {
                    std::array::from_fn(|offset| self.board[BOARD_SIZE - 1 - offset][index])
                }
            };

            let (merged, gained_score) = merge_line(line);
            self.score = self.score.saturating_add(gained_score);

            for (offset, tile) in merged.into_iter().enumerate() {
                match direction {
                    Direction::Left => self.board[index][offset] = tile,
                    Direction::Right => self.board[index][BOARD_SIZE - 1 - offset] = tile,
                    Direction::Up => self.board[offset][index] = tile,
                    Direction::Down => self.board[BOARD_SIZE - 1 - offset][index] = tile,
                }
            }
        }

        self.board != previous
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

fn merge_line(line: [u32; BOARD_SIZE]) -> ([u32; BOARD_SIZE], u32) {
    let tiles: Vec<u32> = line.into_iter().filter(|tile| *tile != 0).collect();
    let mut merged = [0; BOARD_SIZE];
    let mut gained_score: u32 = 0;
    let mut source = 0;
    let mut destination = 0;

    while source < tiles.len() {
        if source + 1 < tiles.len() && tiles[source] == tiles[source + 1] {
            merged[destination] = tiles[source].saturating_mul(2);
            gained_score = gained_score.saturating_add(merged[destination]);
            source += 2;
        } else {
            merged[destination] = tiles[source];
            source += 1;
        }
        destination += 1;
    }

    (merged, gained_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_starts_with_two_valid_tiles() {
        let game = Game::new();
        let tiles: Vec<u32> = game
            .board()
            .iter()
            .flatten()
            .copied()
            .filter(|tile| *tile != 0)
            .collect();

        assert_eq!(tiles.len(), 2);
        assert!(tiles.iter().all(|tile| *tile == 2 || *tile == 4));
    }

    #[test]
    fn move_merges_each_pair_only_once() {
        let mut game = Game::from_board([[2, 2, 2, 2], [0; 4], [0; 4], [0; 4]]);

        assert!(game.apply_move(Direction::Left));
        assert_eq!(game.board()[0], [4, 4, 0, 0]);
        assert_eq!(game.score(), 8);
    }

    #[test]
    fn blocked_move_does_not_change_the_game() {
        let mut game = Game::from_board([[2, 4, 8, 16], [0; 4], [0; 4], [0; 4]]);
        let before = game.clone();

        assert!(!game.make_move(Direction::Left));
        assert_eq!(game, before);
    }

    #[test]
    fn detects_a_finished_game() {
        let game = Game::from_board([[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]]);

        assert!(game.is_game_over());
        assert!(game.available_moves().is_empty());
    }

    #[test]
    fn equal_seeds_create_equal_games() {
        assert_eq!(Game::from_seed(42), Game::from_seed(42));
        assert_ne!(Game::from_seed(42), Game::from_seed(43));
    }
}
