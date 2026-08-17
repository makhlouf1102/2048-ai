use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    neural_network::NeuralNetwork,
    player::{Player, robust_fitness},
};
use log::{debug, info};
use rand::RngExt;
use rayon::prelude::*;
use serde::Serialize;

pub const DEFAULT_MUTATION_RATE: f32 = 0.05;
pub const DEFAULT_MUTATION_STRENGTH: f32 = 0.1;
pub const VALIDATION_GAME_COUNT: usize = 50;
pub const VALIDATION_SEEDS: [u64; VALIDATION_GAME_COUNT] = validation_seeds();
const REFINEMENT_GAME_COUNT: usize = 20;
const HOLDOUT_BATCH_COUNT: usize = 3;
const HOLDOUT_GAMES_PER_BATCH: usize = 20;
const ELITE_PERCENT: usize = 5;
const IMMIGRANT_PERCENT: usize = 5;
const TOURNAMENT_SIZE: usize = 3;

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
    refinement_games_per_finalist: usize,
    finalist_count: usize,
    validation_game_count: usize,
    holdout_batch_count: usize,
    holdout_games_per_batch: usize,
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
    all_time_best_validation_fitness: Option<f32>,
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
            all_time_best_validation_fitness: None,
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
            all_time_best_validation_fitness: None,
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
        let finalist_count = (population_size / 5).max(1);
        let next_generation = self.generation + 1;
        info!(
            "generation {next_generation}: evaluating {population_size} players across {} threads",
            rayon::current_num_threads()
        );
        let game_seeds: [u64; crate::player::FITNESS_GAMES] =
            std::array::from_fn(|_| rand::random());

        let mut evaluations: Vec<(Vec<f32>, f32)> = self
            .players
            .par_iter_mut()
            .enumerate()
            .map(|(index, player)| {
                let scores = player.scores_with_seeds(&game_seeds);
                let fitness = robust_fitness(&scores);
                debug!(
                    "generation {next_generation}: player {}/{} fitness={fitness:.2}",
                    index + 1,
                    population_size
                );
                (scores, fitness)
            })
            .collect();
        let mut initial_order: Vec<usize> = (0..population_size).collect();
        initial_order.sort_by(|left, right| evaluations[*right].1.total_cmp(&evaluations[*left].1));
        let finalist_indices: Vec<usize> =
            initial_order.iter().copied().take(finalist_count).collect();
        let mut is_finalist = vec![false; population_size];
        for index in &finalist_indices {
            is_finalist[*index] = true;
        }

        // Spend extra games only on the initially promising 20%. This reduces
        // selection noise without charging the full population for 30 games.
        let refinement_seeds: [u64; REFINEMENT_GAME_COUNT] =
            std::array::from_fn(|_| rand::random());
        self.players
            .par_iter_mut()
            .zip(evaluations.par_iter_mut())
            .enumerate()
            .filter(|(index, _)| is_finalist[*index])
            .for_each(|(_, (player, evaluation))| {
                evaluation
                    .0
                    .extend(player.scores_with_seeds(&refinement_seeds));
                evaluation.1 = robust_fitness(&evaluation.0);
            });
        let fitnesses: Vec<f32> = evaluations
            .into_iter()
            .map(|(_, fitness)| fitness)
            .collect();
        let average_fitness = fitnesses.iter().sum::<f32>() / fitnesses.len() as f32;
        let mut sorted_fitnesses = fitnesses.clone();
        sorted_fitnesses.sort_by(f32::total_cmp);
        let worst_fitness = sorted_fitnesses[0];
        let median_fitness = (sorted_fitnesses[population_size / 2 - 1]
            + sorted_fitnesses[population_size / 2])
            / 2.0;

        let mut ranked: Vec<(usize, Player, f32)> = self
            .players
            .drain(..)
            .zip(fitnesses)
            .enumerate()
            .map(|(index, (player, fitness))| (index, player, fitness))
            .collect();
        ranked.sort_by(|left, right| {
            is_finalist[right.0]
                .cmp(&is_finalist[left.0])
                .then_with(|| right.2.total_cmp(&left.2))
        });

        let best_fitness = ranked[0].2;

        let mut candidate = ranked[0].1.clone();
        let promoted = match self.champion.as_mut() {
            None => true,
            Some(champion) => beats_champion(&mut candidate, champion, next_generation),
        };
        if promoted {
            self.champion = Some(candidate);
        }

        // This fixed test set is reporting-only: it never decides promotion.
        let candidate_validation_fitness = ranked[0].1.fitness_with_seeds(&VALIDATION_SEEDS);
        let champion_validation_fitness = self
            .champion
            .as_mut()
            .expect("champion was initialized")
            .fitness_with_seeds(&VALIDATION_SEEDS);
        self.champion_validation_fitness = Some(champion_validation_fitness);
        let all_time_best_fitness = self
            .all_time_best_validation_fitness
            .map_or(champion_validation_fitness, |best| {
                best.max(champion_validation_fitness)
            });
        self.all_time_best_validation_fitness = Some(all_time_best_fitness);
        info!(
            "generation {next_generation}: selection complete; training_best={best_fitness:.2}, candidate_validation={candidate_validation_fitness:.2}, champion_validation={champion_validation_fitness:.2}, average={average_fitness:.2}, median={median_fitness:.2}, worst={worst_fitness:.2}"
        );

        let elite_count = ((population_size * ELITE_PERCENT).div_ceil(100)).max(1);
        let immigrant_count = ((population_size * IMMIGRANT_PERCENT).div_ceil(100)).max(1);
        let mut next_population = Vec::with_capacity(population_size);
        next_population.push(
            self.champion
                .as_ref()
                .expect("champion was initialized")
                .clone(),
        );
        next_population.extend(
            ranked
                .iter()
                .take(elite_count.saturating_sub(1))
                .map(|(_, p, _)| p.clone()),
        );

        let parent_pool: Vec<Player> = ranked
            .iter()
            .take(finalist_count)
            .map(|(_, player, _)| player.clone())
            .collect();
        let child_count = population_size - elite_count - immigrant_count;
        let mut rng = rand::rng();
        for child_index in 0..child_count {
            let left = tournament_parent(&parent_pool, &mut rng);
            let right = tournament_parent(&parent_pool, &mut rng);
            let mut child = left.crossover(right);
            let profile = self.mutation_profiles[child_index % self.mutation_profiles.len()];
            child.mutate(profile.rate, profile.strength);
            next_population.push(child);
        }
        next_population.extend((0..immigrant_count).map(|_| Player::new()));

        self.players = next_population;
        self.generation += 1;
        info!(
            "generation {}: retained {} elites, created {} crossover children, and added {} immigrants",
            self.generation, elite_count, child_count, immigrant_count
        );

        GenerationResult {
            generation: self.generation,
            best_fitness,
            average_fitness,
            median_fitness,
            worst_fitness,
            candidate_validation_fitness,
            champion_validation_fitness,
            all_time_best_fitness,
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
            schema_version: 2,
            created_at_unix_ms: timestamp,
            population_size: self.players.len(),
            completed_generations: self.generation,
            fitness_games_per_player: crate::player::FITNESS_GAMES,
            refinement_games_per_finalist: REFINEMENT_GAME_COUNT,
            finalist_count: self.players.len() / 5,
            validation_game_count: VALIDATION_GAME_COUNT,
            holdout_batch_count: HOLDOUT_BATCH_COUNT,
            holdout_games_per_batch: HOLDOUT_GAMES_PER_BATCH,
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

fn tournament_parent<'a, R: rand::Rng + ?Sized>(pool: &'a [Player], rng: &mut R) -> &'a Player {
    let winner = (0..TOURNAMENT_SIZE)
        .map(|_| rng.random_range(0..pool.len()))
        .min()
        .expect("tournament contains competitors");
    &pool[winner]
}

fn beats_champion(candidate: &mut Player, champion: &mut Player, generation: u64) -> bool {
    let mut winning_batches = 0;
    let mut paired_differences = Vec::with_capacity(HOLDOUT_BATCH_COUNT * HOLDOUT_GAMES_PER_BATCH);

    for batch in 0..HOLDOUT_BATCH_COUNT {
        let seeds: Vec<u64> = rotating_holdout_seeds(generation, batch as u64).collect();
        let candidate_scores = candidate.scores_with_seeds(&seeds);
        let champion_scores = champion.scores_with_seeds(&seeds);
        if robust_fitness(&candidate_scores) > robust_fitness(&champion_scores) {
            winning_batches += 1;
        }
        paired_differences.extend(
            candidate_scores
                .into_iter()
                .zip(champion_scores)
                .map(|(candidate, champion)| candidate - champion),
        );
    }

    let mean = paired_differences.iter().sum::<f32>() / paired_differences.len() as f32;
    let variance = paired_differences
        .iter()
        .map(|difference| (difference - mean).powi(2))
        .sum::<f32>()
        / (paired_differences.len() - 1) as f32;
    let standard_error = (variance / paired_differences.len() as f32).sqrt();
    winning_batches >= 2 && mean > standard_error
}

fn rotating_holdout_seeds(generation: u64, batch: u64) -> impl Iterator<Item = u64> {
    let mut state = 0x2048_CAFE_BABE_5EED_u64 ^ generation.rotate_left(17) ^ batch.rotate_left(41);
    (0..HOLDOUT_GAMES_PER_BATCH).map(move |_| {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    })
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
        assert!(first.champion_validation_fitness.is_finite());
        assert!(second.champion_validation_fitness.is_finite());
    }

    #[test]
    fn rotating_holdouts_are_repeatable_and_change_each_generation() {
        let first: Vec<_> = rotating_holdout_seeds(7, 1).collect();
        let repeated: Vec<_> = rotating_holdout_seeds(7, 1).collect();
        let next: Vec<_> = rotating_holdout_seeds(8, 1).collect();

        assert_eq!(first, repeated);
        assert_ne!(first, next);
        assert_eq!(first.len(), HOLDOUT_GAMES_PER_BATCH);
    }

    #[test]
    #[should_panic(expected = "population size must be divisible by five")]
    fn rejects_population_sizes_that_cannot_split_into_fifths() {
        God::new(6);
    }
}
