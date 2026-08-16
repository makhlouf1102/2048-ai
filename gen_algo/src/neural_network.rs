use nalgebra::{DMatrix, DVector};
use rand::RngExt;

#[derive(Debug)]
pub struct NeuralNetwork {
    layers: Vec<Layer>,
}

#[derive(Debug)]
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

    pub fn mutate(&mut self, mutation_rate: f32, mutation_strength: f32) {
        for layer in &mut self.layers {
            layer.mutate(mutation_rate, mutation_strength);
        }
    }
}

fn main() {
    let network = NeuralNetwork::new(&[3, 4, 4, 2]);

    let input = [1.0, 0.5, -0.25];

    let output = network.forward(&input);

    println!("Output:");
    println!("{output}");
}
