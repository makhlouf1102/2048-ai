use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{neural_network::NeuralNetwork, player::Player};
use log::{debug, info};
use serde::Serialize;

pub const DEFAULT_MUTATION_RATE: f32 = 0.05;
pub const DEFAULT_MUTATION_STRENGTH: f32 = 0.1;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct GenerationResult {
    pub generation: u64,
    pub best_fitness: f32,
    pub average_fitness: f32,
    pub median_fitness: f32,
    pub worst_fitness: f32,
}

#[derive(Debug, Serialize)]
struct TrainingStats<'a> {
    schema_version: u8,
    created_at_unix_ms: u128,
    population_size: usize,
    completed_generations: u64,
    fitness_games_per_player: usize,
    mutation_rate: f32,
    mutation_strength: f32,
    generations: &'a [GenerationResult],
}

/// Owns and evolves a population of 2048 players.
#[derive(Debug)]
pub struct God {
    players: Vec<Player>,
    generation: u64,
    mutation_rate: f32,
    mutation_strength: f32,
    best_fitness: Option<f32>,
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
        validate_population_size(population_size);

        info!(
            "creating population: size={population_size}, mutation_rate={mutation_rate}, mutation_strength={mutation_strength}"
        );

        Self {
            players: (0..population_size).map(|_| Player::new()).collect(),
            generation: 0,
            mutation_rate,
            mutation_strength,
            best_fitness: None,
        }
    }

    /// Creates a population around a previously saved brain. The first player
    /// preserves the loaded weights; every other player starts as a mutation of it.
    pub fn from_brain(population_size: usize, brain: NeuralNetwork) -> Self {
        Self::from_brain_with_mutation(
            population_size,
            brain,
            DEFAULT_MUTATION_RATE,
            DEFAULT_MUTATION_STRENGTH,
        )
    }

    pub fn from_brain_with_mutation(
        population_size: usize,
        brain: NeuralNetwork,
        mutation_rate: f32,
        mutation_strength: f32,
    ) -> Self {
        validate_population_size(population_size);
        info!(
            "creating seeded population: size={population_size}, mutation_rate={mutation_rate}, mutation_strength={mutation_strength}"
        );

        let original = Player::from_brain(brain);
        let mut players = Vec::with_capacity(population_size);
        players.push(original.clone());
        players.extend((1..population_size).map(|_| {
            let mut child = original.clone();
            child.mutate(mutation_rate, mutation_strength);
            child
        }));

        Self {
            players,
            generation: 0,
            mutation_rate,
            mutation_strength,
            best_fitness: None,
        }
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Evaluates the population, keeps its strongest half, and creates one
    /// mutated child from every survivor.
    pub fn run_generation(&mut self) -> GenerationResult {
        let population_size = self.players.len();
        let next_generation = self.generation + 1;
        info!("generation {next_generation}: evaluating {population_size} players");

        let fitnesses: Vec<f32> = self
            .players
            .iter_mut()
            .enumerate()
            .map(|(index, player)| {
                let fitness = player.fitness();
                debug!(
                    "generation {next_generation}: player {}/{} fitness={fitness:.2}",
                    index + 1,
                    population_size
                );
                fitness
            })
            .collect();
        let average_fitness = fitnesses.iter().sum::<f32>() / fitnesses.len() as f32;

        let mut ranked: Vec<(Player, f32)> = self.players.drain(..).zip(fitnesses).collect();
        ranked.sort_by(|left, right| right.1.total_cmp(&left.1));

        let best_fitness = ranked[0].1;
        let worst_fitness = ranked[population_size - 1].1;
        let median_fitness =
            (ranked[population_size / 2 - 1].1 + ranked[population_size / 2].1) / 2.0;
        info!(
            "generation {next_generation}: selection complete; best={best_fitness:.2}, average={average_fitness:.2}, median={median_fitness:.2}, worst={worst_fitness:.2}"
        );
        let survivors: Vec<Player> = ranked
            .into_iter()
            .take(population_size / 2)
            .map(|(player, _)| player)
            .collect();

        let mut next_population = survivors.clone();
        next_population.extend(survivors.into_iter().map(|mut child| {
            child.mutate(self.mutation_rate, self.mutation_strength);
            child
        }));

        self.players = next_population;
        self.generation += 1;
        self.best_fitness = Some(best_fitness);
        info!(
            "generation {}: retained {} survivors and created {} mutated children",
            self.generation,
            population_size / 2,
            population_size / 2
        );

        GenerationResult {
            generation: self.generation,
            best_fitness,
            average_fitness,
            median_fitness,
            worst_fitness,
        }
    }

    pub fn run_generations(&mut self, count: usize) -> Vec<GenerationResult> {
        (0..count).map(|_| self.run_generation()).collect()
    }

    /// Saves the strongest survivor from the latest evaluated generation.
    /// The returned filename contains Unix time in milliseconds and its fitness.
    pub fn save_best_brain(&self, directory: impl AsRef<Path>) -> io::Result<PathBuf> {
        let fitness = self.best_fitness.ok_or_else(|| {
            io::Error::other("run at least one generation before saving the best brain")
        })?;
        let directory = directory.as_ref();
        fs::create_dir_all(directory)?;

        let timestamp = current_unix_time_ms()?;
        let filename = format!("{timestamp}_score_{fitness:.2}.json");
        let path = directory.join(filename);
        self.players[0].brain().save(&path)?;
        info!(
            "saved best brain with fitness {fitness:.2} to {}",
            path.display()
        );
        Ok(path)
    }

    /// Saves graph-ready performance history as one JSON document.
    pub fn save_training_stats(
        &self,
        results: &[GenerationResult],
        directory: impl AsRef<Path>,
    ) -> io::Result<PathBuf> {
        let directory = directory.as_ref();
        fs::create_dir_all(directory)?;

        let timestamp = current_unix_time_ms()?;
        let path = directory.join(format!("{timestamp}_training-stats.json"));
        let stats = TrainingStats {
            schema_version: 1,
            created_at_unix_ms: timestamp,
            population_size: self.players.len(),
            completed_generations: self.generation,
            fitness_games_per_player: crate::player::FITNESS_GAMES,
            mutation_rate: self.mutation_rate,
            mutation_strength: self.mutation_strength,
            generations: results,
        };

        let writer = io::BufWriter::new(fs::File::create(&path)?);
        serde_json::to_writer_pretty(writer, &stats).map_err(io::Error::other)?;
        info!("saved training stats to {}", path.display());
        Ok(path)
    }

    pub fn load_brain(path: impl AsRef<Path>) -> io::Result<NeuralNetwork> {
        info!("loading brain from {}", path.as_ref().display());
        NeuralNetwork::load(path)
    }
}

fn current_unix_time_ms() -> io::Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)
        .map(|duration| duration.as_millis())
}

fn validate_population_size(population_size: usize) {
    assert!(
        population_size >= 2,
        "population must contain at least two players"
    );
    assert!(
        population_size.is_multiple_of(2),
        "population size must be even"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_requested_population_of_random_players() {
        let god = God::new(4);

        assert_eq!(god.players().len(), 4);
        assert_eq!(god.generation(), 0);
    }

    #[test]
    fn runs_a_generation_without_changing_population_size() {
        let mut god = God::new(2);

        let result = god.run_generation();

        assert_eq!(result.generation, 1);
        assert_eq!(god.generation(), 1);
        assert_eq!(god.players().len(), 2);
    }

    #[test]
    #[should_panic(expected = "population size must be even")]
    fn rejects_odd_population_sizes() {
        God::new(3);
    }
}
