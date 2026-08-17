use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use log::info;
use rayon::prelude::*;

use crate::{neural_network::NeuralNetwork, player::Player};

pub const DEFAULT_MUTATION_RATE: f32 = 0.05;
pub const DEFAULT_MUTATION_STRENGTH: f32 = 0.1;

#[derive(Debug, Clone, Copy)]
pub struct GenerationResult {
    pub generation: u64,
    pub best_fitness: f32,
    pub average_fitness: f32,
}

/// A small genetic algorithm: score, keep the best half, clone, and mutate.
#[derive(Debug)]
pub struct God {
    players: Vec<Player>,
    generation: u64,
    mutation_rate: f32,
    mutation_strength: f32,
    best_player: Option<Player>,
    best_fitness: f32,
}

impl God {
    pub fn new(population_size: usize) -> Self {
        Self::with_mutation(
            population_size,
            DEFAULT_MUTATION_RATE,
            DEFAULT_MUTATION_STRENGTH,
        )
    }

    pub fn with_mutation(
        population_size: usize,
        mutation_rate: f32,
        mutation_strength: f32,
    ) -> Self {
        assert!(population_size >= 2, "population size must be at least 2");
        Self {
            players: (0..population_size).map(|_| Player::new()).collect(),
            generation: 0,
            mutation_rate,
            mutation_strength,
            best_player: None,
            best_fitness: f32::NEG_INFINITY,
        }
    }

    pub fn from_brain(population_size: usize, brain: NeuralNetwork) -> Self {
        let mut god = Self::new(population_size);
        let parent = Player::from_brain(brain);
        god.players[0] = parent.clone();
        for player in &mut god.players[1..] {
            *player = parent.clone();
            player.mutate(god.mutation_rate, god.mutation_strength);
        }
        god
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn run_generation(&mut self) -> GenerationResult {
        let seeds: [u64; crate::player::FITNESS_GAMES] = std::array::from_fn(|_| rand::random());
        let mut ranked: Vec<(Player, f32)> = self
            .players
            .par_iter_mut()
            .map(|player| (player.clone(), player.fitness_with_seeds(&seeds)))
            .collect();
        ranked.sort_by(|left, right| right.1.total_cmp(&left.1));

        let best_fitness = ranked[0].1;
        let average_fitness =
            ranked.iter().map(|(_, fitness)| fitness).sum::<f32>() / ranked.len() as f32;
        if best_fitness > self.best_fitness {
            self.best_fitness = best_fitness;
            self.best_player = Some(ranked[0].0.clone());
        }

        let survivor_count = ranked.len().div_ceil(2);
        let survivors: Vec<Player> = ranked
            .into_iter()
            .take(survivor_count)
            .map(|(player, _)| player)
            .collect();
        self.players = (0..self.players.len())
            .map(|index| {
                let mut player = survivors[index % survivors.len()].clone();
                if index >= survivors.len() {
                    player.mutate(self.mutation_rate, self.mutation_strength);
                }
                player
            })
            .collect();

        self.generation += 1;
        info!(
            "generation {}: best={best_fitness:.2}, average={average_fitness:.2}",
            self.generation
        );
        GenerationResult {
            generation: self.generation,
            best_fitness,
            average_fitness,
        }
    }

    pub fn run_generations(&mut self, count: usize) -> Vec<GenerationResult> {
        (0..count).map(|_| self.run_generation()).collect()
    }

    pub fn save_best_brain(&self, directory: impl AsRef<Path>) -> io::Result<PathBuf> {
        let player = self.best_player.as_ref().ok_or_else(|| {
            io::Error::other("run at least one generation before saving the best brain")
        })?;
        fs::create_dir_all(&directory)?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_millis();
        let path = directory
            .as_ref()
            .join(format!("{timestamp}_score_{:.2}.json", self.best_fitness));
        player.brain().save(&path)?;
        Ok(path)
    }

    pub fn load_brain(path: impl AsRef<Path>) -> io::Result<NeuralNetwork> {
        NeuralNetwork::load(path)
    }
}
