use std::sync::OnceLock;

use nalgebra::{DMatrix, DVector};
use serde::Deserialize;

use crate::{
    board::{Board, IBoard},
    tile::CELL_COUNT,
    types::Direction,
};

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];
const MODEL_JSON: &str = include_str!("../model.json");
static MODEL: OnceLock<NeuralNetwork> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct NeuralNetwork {
    layers: Vec<Layer>,
}

#[derive(Debug, Deserialize)]
struct Layer {
    weights: DMatrix<f32>,
    biases: DVector<f32>,
}

impl NeuralNetwork {
    fn forward(&self, input: &[f32]) -> DVector<f32> {
        let mut output = DVector::from_column_slice(input);
        for layer in &self.layers {
            output = (&layer.weights * output + &layer.biases).map(f32::tanh);
        }
        output
    }
}

fn model() -> &'static NeuralNetwork {
    MODEL.get_or_init(|| {
        serde_json::from_str(MODEL_JSON).expect("embedded trained model must be valid JSON")
    })
}

pub fn best_move(board: &Board) -> Direction {
    let input = encode_board(board);
    let output = model().forward(&input);
    assert_eq!(output.len(), DIRECTIONS.len());

    // Training penalizes an illegal first choice and then uses the strongest
    // legal output. At inference time we only need the legal fallback.
    DIRECTIONS
        .into_iter()
        .enumerate()
        .filter(|(_, direction)| board.can_move(*direction))
        .max_by(|(left, _), (right, _)| output[*left].total_cmp(&output[*right]))
        .map(|(_, direction)| direction)
        .expect("best_move requires at least one legal move")
}

fn encode_board(board: &Board) -> [f32; CELL_COUNT] {
    let mut input = [0.0; CELL_COUNT];
    for (index, rank) in board.matrix().iter().flatten().copied().enumerate() {
        input[index] = f32::from(rank) / 16.0;
    }
    input
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_model_has_expected_output_shape() {
        assert_eq!(model().forward(&[0.0; CELL_COUNT]).len(), 4);
    }

    #[test]
    fn trained_model_returns_a_legal_move() {
        let board = Board::new(&[2, 4, 8, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let direction = best_move(&board);

        assert!(board.can_move(direction));
    }
}
