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
pub const VALIDATION_GAME_COUNT: usize = 50;
pub const VALIDATION_SEEDS: [u64; VALIDATION_GAME_COUNT] = validation_seeds();

const fn validation_seeds() -> [u64; VALIDATION_GAME_COUNT] {
    let mut seeds = [0; VALIDATION_GAME_COUNT];
    let mut state = 0x0002_048A_11CE_5EED_u64;
    let mut index = 0;
    while index < VALIDATION_GAME_COUNT {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        seeds[index] = state;
        index += 1;
    }
    seeds
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct MutationProfile {
    pub rate: f32,
    pub strength: f32,
}

pub const DEFAULT_MUTATION_PROFILES: [MutationProfile; 4] = [
    MutationProfile {
        rate: 0.02,
        strength: 0.03,
    },
    MutationProfile {
        rate: 0.05,
        strength: 0.10,
    },
    MutationProfile {
        rate: 0.08,
        strength: 0.20,
    },
    MutationProfile {
        rate: 0.15,
        strength: 0.40,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct GenerationResult {
    pub generation: u64,
    pub best_fitness: f32,
    pub average_fitness: f32,
    pub median_fitness: f32,
    pub worst_fitness: f32,
    pub candidate_validation_fitness: f32,
    pub champion_validation_fitness: f32,
    pub all_time_best_fitness: f32,
}

#[derive(Debug, Serialize)]
struct TrainingStats<'a> {
    schema_version: u8,
    created_at_unix_ms: u128,
    population_size: usize,
    completed_generations: u64,
    fitness_games_per_player: usize,
    validation_game_count: usize,
    validation_seeds: &'static [u64],
    mutation_profiles: [MutationProfile; 4],
    generations: &'a [GenerationResult],
}

/// Owns and evolves a population of 2048 players.
#[derive(Debug)]
pub struct God {
    players: Vec<Player>,
    generation: u64,
    mutation_profiles: [MutationProfile; 4],
    champion_validation_fitness: Option<f32>,
    champion: Option<Player>,
}

impl God {
    pub fn new(population_size: usize) -> Self {
        Self::with_mutation_profiles(population_size, DEFAULT_MUTATION_PROFILES)
    }

    pub fn with_mutation(
        population_size: usize,
        mutation_rate: f32,
        mutation_strength: f32,
    ) -> Self {
        Self::with_mutation_profiles(
            population_size,
            [MutationProfile {
                rate: mutation_rate,
                strength: mutation_strength,
            }; 4],
        )
    }

    pub fn with_mutation_profiles(
        population_size: usize,
        mutation_profiles: [MutationProfile; 4],
    ) -> Self {
        validate_population_size(population_size);

        info!(
            "creating population: size={population_size}, mutation_profiles={mutation_profiles:?}"
        );

        Self {
            players: (0..population_size).map(|_| Player::new()).collect(),
            generation: 0,
            mutation_profiles,
            champion_validation_fitness: None,
            champion: None,
        }
    }

    /// Creates a population around a previously saved brain. The first player
    /// preserves the loaded weights; every other player starts as a mutation of it.
    pub fn from_brain(population_size: usize, brain: NeuralNetwork) -> Self {
        Self::from_brain_with_mutation_profiles(population_size, brain, DEFAULT_MUTATION_PROFILES)
    }

    pub fn from_brain_with_mutation(
        population_size: usize,
        brain: NeuralNetwork,
        mutation_rate: f32,
        mutation_strength: f32,
    ) -> Self {
        Self::from_brain_with_mutation_profiles(
            population_size,
            brain,
            [MutationProfile {
                rate: mutation_rate,
                strength: mutation_strength,
            }; 4],
        )
    }

    pub fn from_brain_with_mutation_profiles(
        population_size: usize,
        brain: NeuralNetwork,
        mutation_profiles: [MutationProfile; 4],
    ) -> Self {
        validate_population_size(population_size);
        info!(
            "creating seeded population: size={population_size}, mutation_profiles={mutation_profiles:?}"
        );

        let original = Player::from_brain(brain);
        let mut players = Vec::with_capacity(population_size);
        players.push(original.clone());
        players.extend((1..population_size).map(|index| {
            let mut child = original.clone();
            let profile = mutation_profiles[(index - 1) % mutation_profiles.len()];
            child.mutate(profile.rate, profile.strength);
            child
        }));

        Self {
            players,
            generation: 0,
            mutation_profiles,
            champion_validation_fitness: None,
            champion: None,
        }
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Evaluates everyone on identical game seeds, keeps the strongest 20%,
    /// preserves the all-time champion, and creates four children per survivor.
    pub fn run_generation(&mut self) -> GenerationResult {
        let population_size = self.players.len();
        let survivor_count = population_size / 5;
        let next_generation = self.generation + 1;
        info!("generation {next_generation}: evaluating {population_size} players");
        let game_seeds: [u64; crate::player::FITNESS_GAMES] =
            std::array::from_fn(|_| rand::random());

        let fitnesses: Vec<f32> = self
            .players
            .iter_mut()
            .enumerate()
            .map(|(index, player)| {
                let fitness = player.fitness_with_seeds(&game_seeds);
                debug!(
                    "generation {next_generation}: player {}/{} fitness={fitness:.2}",
                    index + 1,
                    population_size
                );
                fitness
            })
            .collect();
        let average_fitness = fitnesses.iter().sum::<f32>() / fitnesses.len() as f32;

        let mut ranked: Vec<(usize, Player, f32)> = self
            .players
            .drain(..)
            .zip(fitnesses)
            .enumerate()
            .map(|(index, (player, fitness))| (index, player, fitness))
            .collect();
        ranked.sort_by(|left, right| right.2.total_cmp(&left.2));

        let best_fitness = ranked[0].2;
        let worst_fitness = ranked[population_size - 1].2;
        let median_fitness =
            (ranked[population_size / 2 - 1].2 + ranked[population_size / 2].2) / 2.0;

        let mut candidate = ranked[0].1.clone();
        let candidate_validation_fitness = candidate.fitness_with_seeds(&VALIDATION_SEEDS);
        let champion_source_index = if self
            .champion_validation_fitness
            .is_none_or(|champion_fitness| candidate_validation_fitness > champion_fitness)
        {
            self.champion_validation_fitness = Some(candidate_validation_fitness);
            self.champion = Some(candidate);
            ranked[0].0
        } else {
            // The champion is always placed at index zero in each new population.
            0
        };
        let champion_validation_fitness = self
            .champion_validation_fitness
            .expect("champion fitness was initialized");
        info!(
            "generation {next_generation}: selection complete; training_best={best_fitness:.2}, candidate_validation={candidate_validation_fitness:.2}, champion_validation={champion_validation_fitness:.2}, average={average_fitness:.2}, median={median_fitness:.2}, worst={worst_fitness:.2}"
        );

        let mut survivors = Vec::with_capacity(survivor_count);
        survivors.push(
            self.champion
                .as_ref()
                .expect("champion was initialized")
                .clone(),
        );
        survivors.extend(
            ranked
                .into_iter()
                .filter(|(index, _, _)| *index != champion_source_index)
                .take(survivor_count - 1)
                .map(|(_, player, _)| player),
        );

        let mut next_population = survivors.clone();
        for survivor in survivors {
            next_population.extend(self.mutation_profiles.map(|profile| {
                let mut child = survivor.clone();
                child.mutate(profile.rate, profile.strength);
                child
            }));
        }

        self.players = next_population;
        self.generation += 1;
        info!(
            "generation {}: retained {} survivors and created {} mutated children",
            self.generation,
            survivor_count,
            survivor_count * 4
        );

        GenerationResult {
            generation: self.generation,
            best_fitness,
            average_fitness,
            median_fitness,
            worst_fitness,
            candidate_validation_fitness,
            champion_validation_fitness,
            all_time_best_fitness: champion_validation_fitness,
        }
    }

    pub fn run_generations(&mut self, count: usize) -> Vec<GenerationResult> {
        (0..count).map(|_| self.run_generation()).collect()
    }

    /// Saves the strongest survivor from the latest evaluated generation.
    /// The returned filename contains Unix time in milliseconds and its fitness.
    pub fn save_best_brain(&self, directory: impl AsRef<Path>) -> io::Result<PathBuf> {
        let fitness = self.champion_validation_fitness.ok_or_else(|| {
            io::Error::other("run at least one generation before saving the best brain")
        })?;
        let directory = directory.as_ref();
        fs::create_dir_all(directory)?;

        let timestamp = current_unix_time_ms()?;
        let filename = format!("{timestamp}_score_{fitness:.2}.json");
        let path = directory.join(filename);
        self.champion
            .as_ref()
            .expect("a champion exists after a generation")
            .brain()
            .save(&path)?;
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
            validation_game_count: VALIDATION_GAME_COUNT,
            validation_seeds: &VALIDATION_SEEDS,
            mutation_profiles: self.mutation_profiles,
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
        population_size >= 5,
        "population must contain at least five players"
    );
    assert!(
        population_size.is_multiple_of(5),
        "population size must be divisible by five"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_requested_population_of_random_players() {
        let god = God::new(10);

        assert_eq!(god.players().len(), 10);
        assert_eq!(god.generation(), 0);
    }

    #[test]
    fn runs_a_generation_without_changing_population_size() {
        let mut god = God::new(5);

        let first = god.run_generation();
        let second = god.run_generation();

        assert_eq!(second.generation, 2);
        assert_eq!(god.generation(), 2);
        assert_eq!(god.players().len(), 5);
        assert!(second.champion_validation_fitness >= first.champion_validation_fitness);
    }

    #[test]
    #[should_panic(expected = "population size must be divisible by five")]
    fn rejects_population_sizes_that_cannot_split_into_fifths() {
        God::new(6);
    }
}
