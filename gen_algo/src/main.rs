#[path = "2048-game.rs"]
pub mod game_2048;
pub mod god;
pub mod neural_network;
pub mod player;

use std::{env, error::Error, fs, io, path::PathBuf};

use god::God;

const DEFAULT_POPULATION_SIZE: usize = 10;
const DEFAULT_GENERATIONS: usize = 10;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut arguments = env::args().skip(1);
    let population_size =
        parse_argument(arguments.next(), DEFAULT_POPULATION_SIZE, "population size")?;
    let generation_count =
        parse_argument(arguments.next(), DEFAULT_GENERATIONS, "generation count")?;
    // Some package runners forward their `--` separator as a literal argument.
    let saved_brain_argument = arguments
        .find(|argument| argument != "--")
        .map(PathBuf::from);
    let saved_brain_path = resolve_brain_path(saved_brain_argument)?;

    let mut god = if let Some(path) = saved_brain_path {
        log::info!("resuming evolution from {}", path.display());
        let brain = God::load_brain(path)?;
        God::from_brain(population_size, brain)
    } else {
        God::new(population_size)
    };
    log::info!("evolving {population_size} players for {generation_count} generations");

    let results = god.run_generations(generation_count);
    for result in &results {
        log::info!(
            "generation {:>4}: best = {:>10.2}, average = {:>10.2}",
            result.generation,
            result.best_fitness,
            result.average_fitness
        );
    }

    if generation_count > 0 {
        let save_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("saved-brains");
        let saved_path = god.save_best_brain(save_directory)?;
        log::info!("saved best brain to {}", saved_path.display());
    }

    Ok(())
}

fn resolve_brain_path(argument: Option<PathBuf>) -> Result<Option<PathBuf>, Box<dyn Error>> {
    match argument.as_deref() {
        Some(path) if path == std::path::Path::new("--latest") => {
            let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("saved-brains");
            let latest = fs::read_dir(&directory)?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| {
                    path.extension()
                        .is_some_and(|extension| extension == "json")
                })
                .max()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("no saved brains found in {}", directory.display()),
                    )
                })?;

            log::info!("selected latest saved brain: {}", latest.display());
            Ok(Some(latest))
        }
        Some(path) => Ok(Some(path.to_path_buf())),
        None => Ok(None),
    }
}

fn parse_argument(
    argument: Option<String>,
    default: usize,
    name: &str,
) -> Result<usize, Box<dyn Error>> {
    match argument {
        Some(value) => value
            .parse::<usize>()
            .map_err(|error| format!("invalid {name} '{value}': {error}").into()),
        None => Ok(default),
    }
}
