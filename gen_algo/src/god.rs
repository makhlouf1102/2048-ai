use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    neural_network::NeuralNetwork,
    player::{Behavior, Player, robust_fitness},
};
use log::{info, warn};
use rand::RngExt;
use rayon::prelude::*;
use serde::Serialize;

pub const DEFAULT_MUTATION_RATE: f32 = 0.05;
pub const DEFAULT_MUTATION_STRENGTH: f32 = 0.1;
pub const VALIDATION_GAME_COUNT: usize = 50;
pub const VALIDATION_SEEDS: [u64; VALIDATION_GAME_COUNT] = validation_seeds();
const REFINEMENT_GAME_COUNT: usize = 20;
const HOLDOUT_BATCH_COUNT: usize = 5;
const HOLDOUT_GAMES_PER_BATCH: usize = 25;
const ELITE_PERCENT: usize = 5;
const IMMIGRANT_PERCENT: usize = 5;
const TOURNAMENT_SIZE: usize = 3;
const ISLAND_COUNT: usize = 3;
const MIGRATION_INTERVAL: u64 = 25;
const NOVELTY_WEIGHT: f32 = 0.10;
const MUTATION_ADAPTATION_WINDOW: u32 = 10;
const MUTATION_SUCCESS_TARGET: f32 = 0.20;
const FINAL_TEST_GAME_COUNT: usize = 500;

#[derive(Debug, Clone, Copy, Serialize)]
struct IslandState {
    mutation_rate: f32,
    mutation_strength: f32,
    success_rate_sum: f32,
    success_rate_samples: u32,
    minimum_strength: f32,
    maximum_strength: f32,
}

const INITIAL_ISLANDS: [IslandState; ISLAND_COUNT] = [
    IslandState {
        mutation_rate: 0.03,
        mutation_strength: 0.04,
        success_rate_sum: 0.0,
        success_rate_samples: 0,
        minimum_strength: 0.01,
        maximum_strength: 0.10,
    },
    IslandState {
        mutation_rate: 0.06,
        mutation_strength: 0.10,
        success_rate_sum: 0.0,
        success_rate_samples: 0,
        minimum_strength: 0.03,
        maximum_strength: 0.30,
    },
    IslandState {
        mutation_rate: 0.12,
        mutation_strength: 0.25,
        success_rate_sum: 0.0,
        success_rate_samples: 0,
        minimum_strength: 0.10,
        maximum_strength: 0.75,
    },
];

#[derive(Debug, Clone, Copy)]
struct PromotionEvidence {
    promoted: bool,
    candidate_holdout_fitness: f32,
    champion_holdout_fitness: f32,
    winning_batches: usize,
    mean_paired_difference: f32,
    standard_error: f32,
}

#[derive(Serialize)]
struct CheckpointMetadata {
    schema_version: u8,
    champion_kind: &'static str,
    generation: u64,
    fixed_validation_fitness: f32,
    candidate_holdout_fitness: Option<f32>,
    previous_champion_holdout_fitness: Option<f32>,
    winning_holdout_batches: Option<usize>,
    mean_paired_difference: Option<f32>,
    standard_error: Option<f32>,
    source_island: usize,
    island_state: IslandState,
    brain_file: String,
}

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
    pub island_mutation_strengths: [f32; ISLAND_COUNT],
    pub migration_performed: bool,
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
    island_count: usize,
    migration_interval: u64,
    novelty_weight: f32,
    island_states: [IslandState; ISLAND_COUNT],
    final_unseen_test_fitness: Option<f32>,
    selection_champion_final_test_fitness: Option<f32>,
    reporting_champion_final_test_fitness: Option<f32>,
    generations: &'a [GenerationResult],
}

/// Owns and evolves a population of 2048 players.
#[derive(Debug)]
pub struct God {
    players: Vec<Player>,
    parent_fitnesses: Vec<Option<f32>>,
    generation: u64,
    mutation_profiles: [MutationProfile; 4],
    islands: [IslandState; ISLAND_COUNT],
    champion_validation_fitness: Option<f32>,
    all_time_best_validation_fitness: Option<f32>,
    final_unseen_test_fitness: Option<f32>,
    selection_champion_final_test_fitness: Option<f32>,
    reporting_champion_final_test_fitness: Option<f32>,
    champion: Option<Player>,
    reporting_champion: Option<Player>,
    reporting_champion_validation_fitness: Option<f32>,
    checkpoint_directory: Option<PathBuf>,
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
            parent_fitnesses: vec![None; population_size],
            generation: 0,
            mutation_profiles,
            islands: INITIAL_ISLANDS,
            champion_validation_fitness: None,
            all_time_best_validation_fitness: None,
            final_unseen_test_fitness: None,
            selection_champion_final_test_fitness: None,
            reporting_champion_final_test_fitness: None,
            champion: None,
            reporting_champion: None,
            reporting_champion_validation_fitness: None,
            checkpoint_directory: None,
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
        let mut players: Vec<Player> = (0..population_size)
            .map(|index| {
                let mut child = original.clone();
                let profile = mutation_profiles[index % mutation_profiles.len()];
                child.mutate(profile.rate, profile.strength);
                child
            })
            .collect();
        // Every island receives an exact protected copy of the inherited brain.
        for range in island_ranges(population_size) {
            players[range.start] = original.clone();
        }

        Self {
            players,
            parent_fitnesses: vec![None; population_size],
            generation: 0,
            mutation_profiles,
            islands: INITIAL_ISLANDS,
            champion_validation_fitness: None,
            all_time_best_validation_fitness: None,
            final_unseen_test_fitness: None,
            selection_champion_final_test_fitness: None,
            reporting_champion_final_test_fitness: None,
            champion: Some(original.clone()),
            reporting_champion: Some(original),
            reporting_champion_validation_fitness: None,
            checkpoint_directory: None,
        }
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn enable_checkpoints(&mut self, directory: impl Into<PathBuf>) {
        self.checkpoint_directory = Some(directory.into());
    }

    /// Evaluates everyone on identical game seeds, keeps the strongest 20%,
    /// preserves the all-time champion, and creates four children per survivor.
    pub fn run_generation(&mut self) -> GenerationResult {
        let population_size = self.players.len();
        let next_generation = self.generation + 1;
        info!(
            "generation {next_generation}: evaluating {population_size} players on {ISLAND_COUNT} islands across {} threads",
            rayon::current_num_threads(),
        );
        let game_seeds: [u64; crate::player::FITNESS_GAMES] =
            std::array::from_fn(|_| rand::random());

        let mut evaluations: Vec<(Vec<f32>, f32, Behavior)> = self
            .players
            .par_iter_mut()
            .map(|player| {
                let (scores, behavior) = player.evaluate_with_seeds(&game_seeds);
                let fitness = robust_fitness(&scores);
                (scores, fitness, behavior)
            })
            .collect();
        let initial_fitnesses: Vec<f32> =
            evaluations.iter().map(|evaluation| evaluation.1).collect();

        // Adapt from actual child-versus-parent outcomes, using the same ten
        // generation games before progressive finalist refinement changes scores.
        for (island_index, range) in island_ranges(population_size).into_iter().enumerate() {
            let comparisons: Vec<bool> = range
                .filter_map(|index| {
                    self.parent_fitnesses[index].map(|parent| evaluations[index].1 > parent)
                })
                .collect();
            if !comparisons.is_empty() {
                let success_rate = comparisons.iter().filter(|success| **success).count() as f32
                    / comparisons.len() as f32;
                update_island_mutation(&mut self.islands[island_index], success_rate);
            }
        }

        let mut is_finalist = vec![false; population_size];
        for range in island_ranges(population_size) {
            let finalist_count = (range.len() / 5).max(1);
            let mut order: Vec<usize> = range.collect();
            order.sort_by(|left, right| evaluations[*right].1.total_cmp(&evaluations[*left].1));
            for index in order.into_iter().take(finalist_count) {
                is_finalist[index] = true;
            }
        }

        let refinement_seeds: [u64; REFINEMENT_GAME_COUNT] =
            std::array::from_fn(|_| rand::random());
        self.players
            .par_iter_mut()
            .zip(evaluations.par_iter_mut())
            .enumerate()
            .filter(|(index, _)| is_finalist[*index])
            .for_each(|(_, (player, evaluation))| {
                let (scores, refinement_behavior) = player.evaluate_with_seeds(&refinement_seeds);
                evaluation.0.extend(scores);
                evaluation.1 = robust_fitness(&evaluation.0);
                evaluation.2 = blend_behavior(
                    evaluation.2,
                    crate::player::FITNESS_GAMES,
                    refinement_behavior,
                    REFINEMENT_GAME_COUNT,
                );
            });

        let fitnesses: Vec<f32> = evaluations.iter().map(|evaluation| evaluation.1).collect();
        let average_fitness = fitnesses.iter().sum::<f32>() / fitnesses.len() as f32;
        let mut sorted_fitnesses = fitnesses.clone();
        sorted_fitnesses.sort_by(f32::total_cmp);
        let worst_fitness = sorted_fitnesses[0];
        let middle = population_size / 2;
        let median_fitness = if population_size.is_multiple_of(2) {
            (sorted_fitnesses[middle - 1] + sorted_fitnesses[middle]) / 2.0
        } else {
            sorted_fitnesses[middle]
        };

        let old_players = std::mem::take(&mut self.players);
        let mut indexed_players: Vec<Option<Player>> = old_players.into_iter().map(Some).collect();
        let mut next_population = Vec::with_capacity(population_size);
        let mut next_parent_fitnesses = Vec::with_capacity(population_size);
        let mut island_winners: Vec<(Player, f32, usize)> = Vec::with_capacity(ISLAND_COUNT);
        let ranges = island_ranges(population_size);

        for (island_index, range) in ranges.iter().cloned().enumerate() {
            let mut ranked: Vec<(Player, f32, f32, bool, f32)> = range
                .clone()
                .map(|index| {
                    let novelty = behavior_novelty(index, range.clone(), &evaluations);
                    let selection = (1.0 - NOVELTY_WEIGHT) * evaluations[index].1
                        + NOVELTY_WEIGHT * novelty * 500.0;
                    (
                        indexed_players[index].take().unwrap(),
                        evaluations[index].1,
                        selection,
                        is_finalist[index],
                        initial_fitnesses[index],
                    )
                })
                .collect();
            ranked.sort_by(|left, right| {
                right
                    .3
                    .cmp(&left.3)
                    .then_with(|| right.2.total_cmp(&left.2))
            });
            island_winners.push((ranked[0].0.clone(), ranked[0].1, island_index));
            let island_size = ranked.len();
            let elite_count = ((island_size * ELITE_PERCENT).div_ceil(100)).max(1);
            let immigrant_count = ((island_size * IMMIGRANT_PERCENT).div_ceil(100))
                .max(1)
                .min(island_size - elite_count);
            let child_count = island_size.saturating_sub(elite_count + immigrant_count);
            for elite in ranked.iter().take(elite_count) {
                next_population.push(elite.0.clone());
                next_parent_fitnesses.push(Some(elite.4));
            }

            let parent_count = (island_size / 5).max(1);
            let parent_pool: Vec<(Player, f32)> = ranked
                .iter()
                .take(parent_count)
                .map(|entry| (entry.0.clone(), entry.4))
                .collect();
            let mut rng = rand::rng();
            for child_index in 0..child_count {
                let left = &parent_pool[tournament_index(parent_pool.len(), &mut rng)];
                let right = &parent_pool[tournament_index(parent_pool.len(), &mut rng)];
                let mut child = match child_index % 4 {
                    0 | 1 => left.0.neuron_crossover(&right.0),
                    2 => left.0.arithmetic_crossover(&right.0),
                    _ => left.0.clone(),
                };
                let island = self.islands[island_index];
                child.mutate_gaussian(island.mutation_rate, island.mutation_strength);
                next_population.push(child);
                next_parent_fitnesses.push(Some(left.1.max(right.1)));
            }
            for _ in 0..immigrant_count {
                next_population.push(Player::new());
                next_parent_fitnesses.push(None);
            }
        }

        if next_generation.is_multiple_of(MIGRATION_INTERVAL) {
            let migrants: Vec<Player> = island_winners
                .iter()
                .map(|winner| winner.0.clone())
                .collect();
            migrate_ring(&mut next_population, &ranges, &migrants);
            for destination in 0..ISLAND_COUNT {
                next_parent_fitnesses[ranges[destination].end - 1] = None;
            }
            info!("generation {next_generation}: migrated champions around the island ring");
        }

        let (mut candidate, best_fitness, candidate_island) = island_winners
            .iter()
            .max_by(|left, right| left.1.total_cmp(&right.1))
            .cloned()
            .expect("islands have winners");
        let mut reporting_candidate = candidate.clone();
        let had_champion = self.champion.is_some();
        let promotion = match self.champion.as_mut() {
            None => PromotionEvidence {
                promoted: true,
                candidate_holdout_fitness: 0.0,
                champion_holdout_fitness: 0.0,
                winning_batches: HOLDOUT_BATCH_COUNT,
                mean_paired_difference: 0.0,
                standard_error: 0.0,
            },
            Some(champion) => compare_with_champion(&mut candidate, champion, next_generation),
        };
        if promotion.promoted {
            self.champion = Some(candidate.clone());
        }

        // The fixed set selects only the reporting champion; it never controls
        // the rotating-holdout selection champion used by evolution.
        let candidate_validation_fitness =
            reporting_candidate.fitness_with_seeds(&VALIDATION_SEEDS);
        let champion_validation_fitness = self
            .champion
            .as_mut()
            .expect("champion was initialized")
            .fitness_with_seeds(&VALIDATION_SEEDS);
        self.champion_validation_fitness = Some(champion_validation_fitness);

        if self.reporting_champion_validation_fitness.is_none() {
            if self.reporting_champion.is_none() {
                self.reporting_champion = self.champion.clone();
            }
            let inherited_fitness = self
                .reporting_champion
                .as_mut()
                .expect("reporting champion was initialized")
                .fitness_with_seeds(&VALIDATION_SEEDS);
            self.reporting_champion_validation_fitness = Some(inherited_fitness);
        }
        let mut reporting_record = false;
        if candidate_validation_fitness
            > self
                .reporting_champion_validation_fitness
                .unwrap_or(f32::NEG_INFINITY)
        {
            self.reporting_champion = Some(reporting_candidate.clone());
            self.reporting_champion_validation_fitness = Some(candidate_validation_fitness);
            reporting_record = true;
        }
        if champion_validation_fitness
            > self
                .reporting_champion_validation_fitness
                .unwrap_or(f32::NEG_INFINITY)
        {
            self.reporting_champion = self.champion.clone();
            self.reporting_champion_validation_fitness = Some(champion_validation_fitness);
            reporting_record = true;
        }
        let all_time_best_fitness = self
            .reporting_champion_validation_fitness
            .expect("reporting champion fitness was initialized");
        self.all_time_best_validation_fitness = Some(all_time_best_fitness);

        if promotion.promoted {
            self.try_save_checkpoint(
                "selection",
                self.champion.as_ref().expect("selection champion exists"),
                champion_validation_fitness,
                candidate_island,
                had_champion.then_some(promotion),
            );
        }
        if reporting_record {
            self.try_save_checkpoint(
                "reporting",
                self.reporting_champion
                    .as_ref()
                    .expect("reporting champion exists"),
                all_time_best_fitness,
                candidate_island,
                None,
            );
        }
        info!(
            "generation {next_generation}: selection complete; promoted={}, holdout={:.2} vs {:.2} (batches={}/{}), training_best={best_fitness:.2}, candidate_validation={candidate_validation_fitness:.2}, champion_validation={champion_validation_fitness:.2}, reporting_best={all_time_best_fitness:.2}, average={average_fitness:.2}, median={median_fitness:.2}, worst={worst_fitness:.2}",
            promotion.promoted,
            promotion.candidate_holdout_fitness,
            promotion.champion_holdout_fitness,
            promotion.winning_batches,
            HOLDOUT_BATCH_COUNT,
        );

        // Keep the holdout-selected champion genetically active instead of
        // storing it only in the hall of fame.
        let champion_slot = ranges[0].start;
        next_population[champion_slot] = self
            .champion
            .as_ref()
            .expect("selection champion exists")
            .clone();
        next_parent_fitnesses[champion_slot] = None;

        self.players = next_population;
        self.parent_fitnesses = next_parent_fitnesses;
        self.generation += 1;
        info!(
            "generation {}: evolved three islands; mutation strengths={:?}",
            self.generation,
            self.islands.map(|island| island.mutation_strength),
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
            island_mutation_strengths: self.islands.map(|island| island.mutation_strength),
            migration_performed: self.generation.is_multiple_of(MIGRATION_INTERVAL),
        }
    }

    pub fn run_generations(&mut self, count: usize) -> Vec<GenerationResult> {
        (0..count).map(|_| self.run_generation()).collect()
    }

    /// Compares both champions on the same fresh 500-game set and makes the
    /// stronger one the deployment champion saved at the end of training.
    pub fn final_test_fitness(&mut self) -> io::Result<f32> {
        let seeds: [u64; FINAL_TEST_GAME_COUNT] = std::array::from_fn(|_| rand::random());
        let selection_fitness = self
            .champion
            .as_mut()
            .ok_or_else(|| io::Error::other("run at least one generation before final testing"))?
            .fitness_with_seeds(&seeds);
        let reporting_fitness = self
            .reporting_champion
            .as_mut()
            .ok_or_else(|| io::Error::other("reporting champion was not initialized"))?
            .fitness_with_seeds(&seeds);
        self.selection_champion_final_test_fitness = Some(selection_fitness);
        self.reporting_champion_final_test_fitness = Some(reporting_fitness);

        if reporting_fitness > selection_fitness {
            self.champion = self.reporting_champion.clone();
            self.champion_validation_fitness = self.reporting_champion_validation_fitness;
        }
        let winner = selection_fitness.max(reporting_fitness);
        self.final_unseen_test_fitness = Some(winner);
        info!(
            "final unseen comparison: selection={selection_fitness:.2}, reporting={reporting_fitness:.2}, winner={winner:.2}"
        );
        Ok(winner)
    }

    fn try_save_checkpoint(
        &self,
        kind: &'static str,
        player: &Player,
        fixed_validation_fitness: f32,
        source_island: usize,
        promotion: Option<PromotionEvidence>,
    ) {
        let Some(directory) = self.checkpoint_directory.as_ref() else {
            return;
        };
        if let Err(error) = save_checkpoint(
            directory,
            kind,
            self.generation + 1,
            player,
            fixed_validation_fitness,
            source_island,
            self.islands[source_island],
            promotion,
        ) {
            warn!("failed to save {kind} checkpoint: {error}");
        }
    }

    /// Saves the strongest survivor from the latest evaluated generation.
    /// The returned filename contains Unix time in milliseconds and its fitness.
    pub fn save_best_brain(&self, directory: impl AsRef<Path>) -> io::Result<PathBuf> {
        let fitness = self
            .final_unseen_test_fitness
            .or(self.champion_validation_fitness)
            .ok_or_else(|| {
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
            schema_version: 5,
            created_at_unix_ms: timestamp,
            population_size: self.players.len(),
            completed_generations: self.generation,
            fitness_games_per_player: crate::player::FITNESS_GAMES,
            refinement_games_per_finalist: REFINEMENT_GAME_COUNT,
            finalist_count: island_ranges(self.players.len())
                .iter()
                .map(|range| (range.len() / 5).max(1))
                .sum(),
            validation_game_count: VALIDATION_GAME_COUNT,
            holdout_batch_count: HOLDOUT_BATCH_COUNT,
            holdout_games_per_batch: HOLDOUT_GAMES_PER_BATCH,
            validation_seeds: &VALIDATION_SEEDS,
            mutation_profiles: self.mutation_profiles,
            island_count: ISLAND_COUNT,
            migration_interval: MIGRATION_INTERVAL,
            novelty_weight: NOVELTY_WEIGHT,
            island_states: self.islands,
            final_unseen_test_fitness: self.final_unseen_test_fitness,
            selection_champion_final_test_fitness: self.selection_champion_final_test_fitness,
            reporting_champion_final_test_fitness: self.reporting_champion_final_test_fitness,
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

fn tournament_index<R: rand::Rng + ?Sized>(pool_len: usize, rng: &mut R) -> usize {
    (0..TOURNAMENT_SIZE)
        .map(|_| rng.random_range(0..pool_len))
        .min()
        .expect("tournament contains competitors")
}

fn island_ranges(population_size: usize) -> Vec<std::ops::Range<usize>> {
    let base = population_size / ISLAND_COUNT;
    let remainder = population_size % ISLAND_COUNT;
    let mut start = 0;
    (0..ISLAND_COUNT)
        .map(|island| {
            let size = base + usize::from(island < remainder);
            let range = start..start + size;
            start += size;
            range
        })
        .collect()
}

fn update_island_mutation(island: &mut IslandState, offspring_success_rate: f32) {
    island.success_rate_sum += offspring_success_rate;
    island.success_rate_samples += 1;
    if island.success_rate_samples < MUTATION_ADAPTATION_WINDOW {
        return;
    }

    let average_success = island.success_rate_sum / island.success_rate_samples as f32;
    if average_success > MUTATION_SUCCESS_TARGET {
        island.mutation_strength = (island.mutation_strength * 1.2).min(island.maximum_strength);
    } else {
        island.mutation_strength = (island.mutation_strength / 1.2).max(island.minimum_strength);
    }
    island.success_rate_sum = 0.0;
    island.success_rate_samples = 0;
}

fn migrate_ring(population: &mut [Player], ranges: &[std::ops::Range<usize>], winners: &[Player]) {
    for (island, winner) in winners.iter().enumerate().take(ISLAND_COUNT) {
        let destination = (island + 1) % ISLAND_COUNT;
        let replacement = ranges[destination].end - 1;
        population[replacement] = winner.clone();
    }
}

fn blend_behavior(
    left: Behavior,
    left_count: usize,
    right: Behavior,
    right_count: usize,
) -> Behavior {
    let total = (left_count + right_count) as f32;
    let blend = |a: f32, b: f32| (a * left_count as f32 + b * right_count as f32) / total;
    Behavior {
        max_tile_rank: blend(left.max_tile_rank, right.max_tile_rank),
        moves: blend(left.moves, right.moves),
        final_empty_tiles: blend(left.final_empty_tiles, right.final_empty_tiles),
        direction_frequencies: std::array::from_fn(|index| {
            blend(
                left.direction_frequencies[index],
                right.direction_frequencies[index],
            )
        }),
    }
}

fn behavior_novelty(
    index: usize,
    peers: std::ops::Range<usize>,
    evaluations: &[(Vec<f32>, f32, Behavior)],
) -> f32 {
    let behavior = evaluations[index].2;
    let mut distances: Vec<f32> = peers
        .filter(|peer| *peer != index)
        .map(|peer| behavior_distance(behavior, evaluations[peer].2))
        .collect();
    distances.sort_by(f32::total_cmp);
    let neighbor_count = distances.len().min(3);
    if neighbor_count == 0 {
        0.0
    } else {
        distances.iter().take(neighbor_count).sum::<f32>() / neighbor_count as f32
    }
}

fn behavior_distance(left: Behavior, right: Behavior) -> f32 {
    let mut squared = ((left.max_tile_rank - right.max_tile_rank) / 12.0).powi(2)
        + ((left.moves - right.moves) / 1_000.0).powi(2)
        + ((left.final_empty_tiles - right.final_empty_tiles) / 16.0).powi(2);
    squared += left
        .direction_frequencies
        .into_iter()
        .zip(right.direction_frequencies)
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f32>();
    squared.sqrt()
}

fn compare_with_champion(
    candidate: &mut Player,
    champion: &mut Player,
    generation: u64,
) -> PromotionEvidence {
    let mut winning_batches = 0;
    let mut paired_differences = Vec::with_capacity(HOLDOUT_BATCH_COUNT * HOLDOUT_GAMES_PER_BATCH);
    let mut candidate_fitness_total = 0.0;
    let mut champion_fitness_total = 0.0;

    for batch in 0..HOLDOUT_BATCH_COUNT {
        let seeds: Vec<u64> = rotating_holdout_seeds(generation, batch as u64).collect();
        let candidate_scores = candidate.scores_with_seeds(&seeds);
        let champion_scores = champion.scores_with_seeds(&seeds);
        let candidate_fitness = robust_fitness(&candidate_scores);
        let champion_fitness = robust_fitness(&champion_scores);
        candidate_fitness_total += candidate_fitness;
        champion_fitness_total += champion_fitness;
        if candidate_fitness > champion_fitness {
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
    PromotionEvidence {
        promoted: winning_batches >= HOLDOUT_BATCH_COUNT.div_ceil(2) && mean > standard_error,
        candidate_holdout_fitness: candidate_fitness_total / HOLDOUT_BATCH_COUNT as f32,
        champion_holdout_fitness: champion_fitness_total / HOLDOUT_BATCH_COUNT as f32,
        winning_batches,
        mean_paired_difference: mean,
        standard_error,
    }
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

#[allow(clippy::too_many_arguments)]
fn save_checkpoint(
    directory: &Path,
    kind: &'static str,
    generation: u64,
    player: &Player,
    fixed_validation_fitness: f32,
    source_island: usize,
    island_state: IslandState,
    promotion: Option<PromotionEvidence>,
) -> io::Result<()> {
    fs::create_dir_all(directory)?;
    let timestamp = current_unix_time_ms()?;
    let stem =
        format!("{timestamp}_generation_{generation}_{kind}_score_{fixed_validation_fitness:.2}");
    let brain_filename = format!("{stem}.brain.json");
    player.brain().save(directory.join(&brain_filename))?;

    let metadata = CheckpointMetadata {
        schema_version: 1,
        champion_kind: kind,
        generation,
        fixed_validation_fitness,
        candidate_holdout_fitness: promotion.map(|value| value.candidate_holdout_fitness),
        previous_champion_holdout_fitness: promotion.map(|value| value.champion_holdout_fitness),
        winning_holdout_batches: promotion.map(|value| value.winning_batches),
        mean_paired_difference: promotion.map(|value| value.mean_paired_difference),
        standard_error: promotion.map(|value| value.standard_error),
        source_island,
        island_state,
        brain_file: brain_filename,
    };
    let writer = io::BufWriter::new(fs::File::create(
        directory.join(format!("{stem}.metadata.json")),
    )?);
    serde_json::to_writer_pretty(writer, &metadata).map_err(io::Error::other)
}

fn current_unix_time_ms() -> io::Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)
        .map(|duration| duration.as_millis())
}

fn validate_population_size(population_size: usize) {
    assert!(
        population_size >= 9,
        "three islands require at least nine players"
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
    fn resumed_brain_is_protected_in_every_island_and_as_champion() {
        let brain = NeuralNetwork::new(&[crate::game_2048::CELL_COUNT, 16, 4]);
        let expected = brain.forward(&[0.0; crate::game_2048::CELL_COUNT]);
        let god = God::from_brain(10, brain);

        for range in island_ranges(10) {
            assert_eq!(
                god.players[range.start].brain().forward(&[0.0; 16]),
                expected
            );
        }
        assert!(god.champion.is_some());
        assert!(god.reporting_champion.is_some());
    }

    #[test]
    fn runs_a_generation_without_changing_population_size() {
        let mut god = God::new(10);

        let first = god.run_generation();
        let second = god.run_generation();

        assert_eq!(second.generation, 2);
        assert_eq!(god.generation(), 2);
        assert_eq!(god.players().len(), 10);
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
    fn splits_population_across_three_balanced_islands() {
        let ranges = island_ranges(10);

        assert_eq!(ranges, vec![0..4, 4..7, 7..10]);
    }

    #[test]
    fn high_offspring_success_increases_mutation_after_full_window() {
        let mut island = INITIAL_ISLANDS[0];
        let initial = island.mutation_strength;
        for _ in 0..MUTATION_ADAPTATION_WINDOW - 1 {
            update_island_mutation(&mut island, 0.5);
        }
        assert_eq!(island.mutation_strength, initial);
        update_island_mutation(&mut island, 0.5);

        assert!(island.mutation_strength > initial);
    }

    #[test]
    fn islands_keep_distinct_mutation_ceiling_during_long_stagnation() {
        let mut islands = INITIAL_ISLANDS;
        for island in &mut islands {
            for _ in 0..500 {
                update_island_mutation(island, 1.0);
            }
        }

        assert_eq!(islands[0].mutation_strength, 0.10);
        assert_eq!(islands[1].mutation_strength, 0.30);
        assert_eq!(islands[2].mutation_strength, 0.75);
    }

    #[test]
    fn low_offspring_success_reduces_mutation_strength() {
        let mut island = INITIAL_ISLANDS[1];
        let before = island.mutation_strength;
        for _ in 0..MUTATION_ADAPTATION_WINDOW {
            update_island_mutation(&mut island, 0.0);
        }

        assert!(island.mutation_strength < before);
        assert_eq!(island.success_rate_samples, 0);
    }

    #[test]
    #[should_panic(expected = "three islands require at least nine players")]
    fn rejects_populations_too_small_for_three_islands() {
        God::new(6);
    }
}
