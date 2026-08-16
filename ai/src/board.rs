use crate::tile::*;
use crate::types::*;
use rand::RngExt;

pub trait IBoard {
    fn new(flatten_board: &[UTile]) -> Self;
    fn empty_tiles(&self) -> Vec<(usize, usize)>;
    fn set_empty_tile(&mut self);
    fn can_move(&self, direction: Direction) -> bool;
    fn available_moves(&self) -> Vec<Direction>;
    fn make_move(&self, direction: Direction) -> Self;
    fn merge_left(&self) -> Self;
    fn merge_right(&self) -> Self;
    fn merge_up(&self) -> Self;
    fn merge_down(&self) -> Self;
}
#[derive(Clone)]
pub struct Board {
    matrix: Matrix,
    pub(crate) score: u32,
}

impl Board {
    fn random_number(max: usize) -> usize {
        rand::rng().random_range(0..max)
    }

    pub(crate) fn matrix(&self) -> &Matrix {
        &self.matrix
    }

    pub(crate) fn with_tile(&self, row: usize, col: usize, value: UTile) -> Self {
        debug_assert_eq!(self.matrix[row][col], 0);
        let mut board = self.clone();
        board.matrix[row][col] = value;
        board
    }
}

impl IBoard for Board {
    fn new(flatten_board: &[UTile]) -> Self {
        let mut matrix: Matrix = [[0; SIZE]; SIZE];
        for row in 0..SIZE {
            for col in 0..SIZE {
                matrix[row][col] = flatten_board[row * SIZE + col];
            }
        }

        Self { score: 0, matrix }
    }

    fn empty_tiles(&self) -> Vec<(usize, usize)> {
        let mut empty = Vec::new();

        for row in 0..SIZE {
            for col in 0..SIZE {
                if self.matrix[row][col] == 0 {
                    empty.push((row, col));
                }
            }
        }

        empty
    }

    fn set_empty_tile(&mut self) {
        let tiles = self.empty_tiles();

        if tiles.is_empty() {
            return;
        }

        let value = if Board::random_number(100) < 90 { 2 } else { 4 };

        let index = Board::random_number(tiles.len());
        let tile = tiles[index];

        self.matrix[tile.0][tile.1] = value;
    }

    fn can_move(&self, direction: Direction) -> bool {
        match direction {
            Direction::Left => {
                for row in 0..SIZE {
                    for col in 1..SIZE {
                        let current = self.matrix[row][col];

                        if current != 0 {
                            let left = self.matrix[row][col - 1];

                            if left == 0 || left == current {
                                return true;
                            }
                        }
                    }
                }
            }

            Direction::Right => {
                for row in 0..SIZE {
                    for col in 0..SIZE - 1 {
                        let current = self.matrix[row][col];

                        if current != 0 {
                            let right = self.matrix[row][col + 1];

                            if right == 0 || right == current {
                                return true;
                            }
                        }
                    }
                }
            }

            Direction::Up => {
                for row in 1..SIZE {
                    for col in 0..SIZE {
                        let current = self.matrix[row][col];

                        if current != 0 {
                            let above = self.matrix[row - 1][col];

                            if above == 0 || above == current {
                                return true;
                            }
                        }
                    }
                }
            }

            Direction::Down => {
                for row in 0..SIZE - 1 {
                    for col in 0..SIZE {
                        let current = self.matrix[row][col];

                        if current != 0 {
                            let below = self.matrix[row + 1][col];

                            if below == 0 || below == current {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    fn available_moves(&self) -> Vec<Direction> {
        let mut arr: Vec<Direction> = Vec::new();

        // Example conditions
        if self.can_move(Direction::Up) {
            arr.push(Direction::Up);
        }

        if self.can_move(Direction::Down) {
            arr.push(Direction::Down);
        }

        if self.can_move(Direction::Left) {
            arr.push(Direction::Left);
        }

        if self.can_move(Direction::Right) {
            arr.push(Direction::Right);
        }

        arr
    }

    fn make_move(&self, direction: Direction) -> Board {
        match direction {
            Direction::Left => self.merge_left(),
            Direction::Right => self.merge_right(),
            Direction::Up => self.merge_up(),
            Direction::Down => self.merge_down(),
        }
    }

    fn merge_left(&self) -> Self {
        let mut board = self.clone();

        for row in 0..SIZE {
            let tiles: Vec<UTile> = self.matrix[row]
                .iter()
                .copied()
                .filter(|&x| x != 0)
                .collect();

            let mut col = 0;
            let mut i = 0;

            board.matrix[row] = [0; SIZE];

            while i < tiles.len() {
                if i + 1 < tiles.len() && tiles[i] == tiles[i + 1] {
                    let merged = tiles[i] * 2;

                    board.matrix[row][col] = merged;
                    board.score += merged;

                    i += 2;
                } else {
                    board.matrix[row][col] = tiles[i];
                    i += 1;
                }

                col += 1;
            }
        }

        board
    }

    fn merge_right(&self) -> Self {
        let mut board = self.clone();

        for row in 0..SIZE {
            let tiles: Vec<UTile> = self.matrix[row]
                .iter()
                .rev()
                .copied()
                .filter(|&x| x != 0)
                .collect();

            let mut col = SIZE;
            let mut i = 0;

            board.matrix[row] = [0; SIZE];

            while i < tiles.len() {
                col -= 1;

                if i + 1 < tiles.len() && tiles[i] == tiles[i + 1] {
                    let merged = tiles[i] * 2;

                    board.matrix[row][col] = merged;
                    board.score += merged;

                    i += 2;
                } else {
                    board.matrix[row][col] = tiles[i];
                    i += 1;
                }
            }
        }

        board
    }

    fn merge_up(&self) -> Self {
        let mut board = self.clone();

        for col in 0..SIZE {
            let mut tiles: Vec<UTile> = Vec::new();

            // Collect non-zero values from top to bottom
            for row in 0..SIZE {
                if self.matrix[row][col] != 0 {
                    tiles.push(self.matrix[row][col]);
                }

                board.matrix[row][col] = 0;
            }

            let mut row = 0;
            let mut i = 0;

            while i < tiles.len() {
                if i + 1 < tiles.len() && tiles[i] == tiles[i + 1] {
                    let merged = tiles[i] * 2;

                    board.matrix[row][col] = merged;
                    board.score += merged;

                    i += 2;
                } else {
                    board.matrix[row][col] = tiles[i];
                    i += 1;
                }

                row += 1;
            }
        }

        board
    }

    fn merge_down(&self) -> Self {
        let mut board = self.clone();

        for col in 0..SIZE {
            let mut tiles: Vec<UTile> = Vec::new();

            // Collect non-zero values from bottom to top
            for row in (0..SIZE).rev() {
                if self.matrix[row][col] != 0 {
                    tiles.push(self.matrix[row][col]);
                }

                board.matrix[row][col] = 0;
            }

            let mut row = SIZE;
            let mut i = 0;

            while i < tiles.len() {
                row -= 1;

                if i + 1 < tiles.len() && tiles[i] == tiles[i + 1] {
                    let merged = tiles[i] * 2;

                    board.matrix[row][col] = merged;
                    board.score += merged;

                    i += 2;
                } else {
                    board.matrix[row][col] = tiles[i];
                    i += 1;
                }
            }
        }

        board
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_empty_tiles_in_row_major_order() {
        let board = Board::new(&[2, 0, 4, 0, 0, 8, 0, 16, 32, 0, 64, 0, 0, 128, 0, 256]);

        assert_eq!(
            board.empty_tiles(),
            vec![
                (0, 1),
                (0, 3),
                (1, 0),
                (1, 2),
                (2, 1),
                (2, 3),
                (3, 0),
                (3, 2)
            ]
        );
    }
}
