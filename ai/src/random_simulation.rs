use crate::{
    board::{Board, IBoard},
    types::Direction,
};

/// Number of complete random games evaluated for every available first move.
pub const DEFAULT_RUN_COUNT: usize = 100;

/// Chooses the opening move whose random continuations have the highest
/// average final score.
pub fn best_move(board: &Board, run_count: usize) -> Direction {
    let moves = board.available_moves();
    debug_assert!(!moves.is_empty());

    let runs = run_count.max(1);
    let base_seed = board_seed(board);
    let mut best_direction = moves[0];
    let mut best_total = 0_u64;

    for (move_index, direction) in moves.into_iter().enumerate() {
        let mut total = 0_u64;

        for run in 0..runs {
            let seed = base_seed
                ^ (move_index as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ (run as u64 + 1).wrapping_mul(0xD1B5_4A32_D192_ED03);
            total += u64::from(play_random_game(board, direction, Random::new(seed)));
        }

        if total > best_total {
            best_total = total;
            best_direction = direction;
        }
    }

    best_direction
}

fn play_random_game(board: &Board, first_move: Direction, mut random: Random) -> u32 {
    let mut board = board.make_move(first_move);
    spawn_random_tile(&mut board, &mut random);

    loop {
        let moves = board.available_moves();
        if moves.is_empty() {
            return board.score;
        }

        let direction = moves[random.index(moves.len())];
        board = board.make_move(direction);
        spawn_random_tile(&mut board, &mut random);
    }
}

fn spawn_random_tile(board: &mut Board, random: &mut Random) {
    let empty = board.empty_tiles();
    if empty.is_empty() {
        return;
    }

    let (row, col) = empty[random.index(empty.len())];
    // Standard 2048 distribution: 90% twos and 10% fours. Board stores ranks.
    let rank = if random.index(10) < 9 { 1 } else { 2 };
    *board = board.with_rank(row, col, rank);
}

fn board_seed(board: &Board) -> u64 {
    let mut hash = 0xCBF2_9CE4_8422_2325_u64;
    for rank in board.matrix().iter().flatten() {
        hash ^= u64::from(*rank);
        hash = hash.wrapping_mul(0x100_0000_01B3);
    }
    hash
}

/// Small self-contained generator so native and WebAssembly builds use the
/// same simulation without requiring a browser randomness API.
struct Random(u64);

impl Random {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 {
            0xA076_1D64_78BD_642F
        } else {
            seed
        })
    }

    fn next(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.0 = value;
        value.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn index(&mut self, upper_bound: usize) -> usize {
        (self.next() % upper_bound as u64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_move_is_legal() {
        let board = Board::new(&[2, 2, 4, 8, 0, 4, 8, 16, 0, 0, 16, 32, 0, 0, 0, 64]);

        let direction = best_move(&board, 4);

        assert!(board.can_move(direction));
    }

    #[test]
    fn zero_runs_still_evaluates_each_move_once() {
        let board = Board::new(&[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        assert!(board.can_move(best_move(&board, 0)));
    }
}
