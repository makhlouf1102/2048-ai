use nalgebra::{DMatrix, DVector};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralNetwork {
    layers: Vec<Layer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    weights: DMatrix<f32>,
    biases: DVector<f32>,
}

impl Layer {
    pub fn new(input_size: usize, output_size: usize) -> Self {
        let mut rng = rand::rng();

        // Matrix shape: output_size × input_size
        let weights =
            DMatrix::from_fn(output_size, input_size, |_, _| rng.random_range(-1.0..=1.0));

        // One bias per output neuron
        let biases = DVector::from_fn(output_size, |_, _| rng.random_range(-1.0..=1.0));

        Self { weights, biases }
    }

    pub fn forward(&self, input: &DVector<f32>) -> DVector<f32> {
        assert_eq!(
            input.len(),
            self.weights.ncols(),
            "Input size does not match layer input size"
        );

        // y = tanh(Wx + b)
        (&self.weights * input + &self.biases).map(|x| x.tanh())
    }

    pub fn mutate(&mut self, mutation_rate: f32, mutation_strength: f32) {
        let mut rng = rand::rng();

        for weight in self.weights.iter_mut() {
            if rng.random::<f32>() < mutation_rate {
                *weight += rng.random_range(-mutation_strength..=mutation_strength);
            }
        }

        for bias in self.biases.iter_mut() {
            if rng.random::<f32>() < mutation_rate {
                *bias += rng.random_range(-mutation_strength..=mutation_strength);
            }
        }
    }

    pub fn mutate_gaussian(&mut self, mutation_rate: f32, mutation_strength: f32) {
        let mut rng = rand::rng();
        for value in self.weights.iter_mut().chain(self.biases.iter_mut()) {
            if rng.random::<f32>() < mutation_rate {
                // Box-Muller transform avoids another distribution dependency.
                let u1 = rng.random::<f32>().max(f32::MIN_POSITIVE);
                let u2 = rng.random::<f32>();
                let normal = (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos();
                *value += normal * mutation_strength;
            }
        }
    }
}

impl NeuralNetwork {
    pub fn new(sizes: &[usize]) -> Self {
        assert!(
            sizes.len() >= 2,
            "Neural network needs at least an input and output layer"
        );

        let layers = sizes
            .windows(2)
            .map(|window| Layer::new(window[0], window[1]))
            .collect();

        Self { layers }
    }

    pub fn forward(&self, input: &[f32]) -> DVector<f32> {
        let mut output = DVector::from_column_slice(input);

        for layer in &self.layers {
            output = layer.forward(&output);
        }

        output
    }

    pub fn layout(&self) -> Vec<usize> {
        let mut layout = Vec::with_capacity(self.layers.len() + 1);
        if let Some(first) = self.layers.first() {
            layout.push(first.weights.ncols());
            layout.extend(self.layers.iter().map(|layer| layer.weights.nrows()));
        }
        layout
    }

    pub fn mutate(&mut self, mutation_rate: f32, mutation_strength: f32) {
        for layer in &mut self.layers {
            layer.mutate(mutation_rate, mutation_strength);
        }
    }

    pub fn mutate_gaussian(&mut self, mutation_rate: f32, mutation_strength: f32) {
        for layer in &mut self.layers {
            layer.mutate_gaussian(mutation_rate, mutation_strength);
        }
    }

    /// Creates a child by independently inheriting each parameter from either parent.
    pub fn crossover(&self, other: &Self) -> Self {
        assert_eq!(
            self.layers.len(),
            other.layers.len(),
            "network layouts must match"
        );
        let mut rng = rand::rng();
        let mut child = self.clone();

        for ((child_layer, left), right) in
            child.layers.iter_mut().zip(&self.layers).zip(&other.layers)
        {
            assert_eq!(
                left.weights.shape(),
                right.weights.shape(),
                "network layouts must match"
            );
            assert_eq!(
                left.biases.len(),
                right.biases.len(),
                "network layouts must match"
            );

            for ((value, left_value), right_value) in child_layer
                .weights
                .iter_mut()
                .zip(left.weights.iter())
                .zip(right.weights.iter())
            {
                *value = if rng.random::<bool>() {
                    *left_value
                } else {
                    *right_value
                };
            }
            for ((value, left_value), right_value) in child_layer
                .biases
                .iter_mut()
                .zip(left.biases.iter())
                .zip(right.biases.iter())
            {
                *value = if rng.random::<bool>() {
                    *left_value
                } else {
                    *right_value
                };
            }
        }

        child
    }

    /// Inherits each complete neuron (incoming weights plus bias) from one parent.
    pub fn neuron_crossover(&self, other: &Self) -> Self {
        self.assert_same_layout(other);
        let mut rng = rand::rng();
        let mut child = self.clone();
        for ((child_layer, left), right) in
            child.layers.iter_mut().zip(&self.layers).zip(&other.layers)
        {
            for row in 0..child_layer.weights.nrows() {
                let source = if rng.random::<bool>() { left } else { right };
                child_layer
                    .weights
                    .row_mut(row)
                    .copy_from(&source.weights.row(row));
                child_layer.biases[row] = source.biases[row];
            }
        }
        child
    }

    /// Blends corresponding real-valued parameters using one random mix per child.
    pub fn arithmetic_crossover(&self, other: &Self) -> Self {
        self.assert_same_layout(other);
        let alpha = rand::rng().random::<f32>();
        let mut child = self.clone();
        for ((child_layer, left), right) in
            child.layers.iter_mut().zip(&self.layers).zip(&other.layers)
        {
            child_layer.weights = &left.weights * alpha + &right.weights * (1.0 - alpha);
            child_layer.biases = &left.biases * alpha + &right.biases * (1.0 - alpha);
        }
        child
    }

    fn assert_same_layout(&self, other: &Self) {
        assert_eq!(self.layout(), other.layout(), "network layouts must match");
    }

    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let writer = BufWriter::new(File::create(path)?);
        serde_json::to_writer_pretty(writer, self).map_err(io::Error::other)
    }

    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let reader = BufReader::new(File::open(path)?);
        serde_json::from_reader(reader).map_err(io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crossover_operators_preserve_network_layout() {
        let left = NeuralNetwork::new(&[16, 16, 4]);
        let right = NeuralNetwork::new(&[16, 16, 4]);

        assert_eq!(left.neuron_crossover(&right).layout(), vec![16, 16, 4]);
        assert_eq!(left.arithmetic_crossover(&right).layout(), vec![16, 16, 4]);
    }

    #[test]
    fn gaussian_mutation_changes_parameters() {
        let mut network = NeuralNetwork::new(&[2, 2]);
        let before = network.layers[0].weights.clone();

        network.mutate_gaussian(1.0, 0.1);

        assert_ne!(network.layers[0].weights, before);
    }
}
