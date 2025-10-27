use std::{env, time::Duration};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub solution_ttl: Duration,
    pub puzzle_ttl: Duration,
    pub cache_prefill_per_size: usize,
    pub cache_max_per_size: usize,
    pub generator_concurrency: usize,
    pub cleanup_interval: Duration,
    pub prefill_dimensions: Vec<(u32, u32)>,
    pub log_level: String,
    pub immediate_cache_cleanup: bool,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("SERVER_PORT")
            .ok()
            .and_then(|raw| raw.parse::<u16>().ok())
            .unwrap_or(8080);

        let workers = env::var("SERVER_WORKERS")
            .ok()
            .and_then(|raw| raw.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or_else(num_cpus::get);

        let solution_ttl = env::var("PUZZLE_SOLUTION_TTL_SECS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .filter(|secs| *secs > 0)
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(600));

        let puzzle_ttl = env::var("PUZZLE_CACHE_TTL_SECS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .filter(|secs| *secs > 0)
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(300));

        let cache_prefill_per_size = env::var("PUZZLE_CACHE_PREFILL")
            .ok()
            .and_then(|raw| raw.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(8);

        let cache_max_per_size = env::var("PUZZLE_CACHE_MAX")
            .ok()
            .and_then(|raw| raw.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(32);

        let generator_concurrency = env::var("PUZZLE_GENERATOR_CONCURRENCY")
            .ok()
            .and_then(|raw| raw.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or_else(|| num_cpus::get().max(2));

        let cleanup_interval = env::var("CLEANUP_INTERVAL_SECS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .filter(|secs| *secs > 0)
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(300));

        let prefill_dimensions = parse_prefill_dimensions(
            env::var("PUZZLE_PREFILL_DIMENSIONS").unwrap_or_else(|_| "500x300".to_string()),
        );

        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        let immediate_cache_cleanup = env::var("IMMEDIATE_CACHE_CLEANUP")
            .ok()
            .and_then(|raw| raw.parse::<bool>().ok())
            .unwrap_or(true);

        AppConfig {
            host,
            port,
            workers,
            solution_ttl,
            puzzle_ttl,
            cache_prefill_per_size,
            cache_max_per_size,
            generator_concurrency,
            cleanup_interval,
            prefill_dimensions,
            log_level,
            immediate_cache_cleanup,
        }
    }
}

fn parse_prefill_dimensions(raw: String) -> Vec<(u32, u32)> {
    raw.split(',')
        .filter_map(|part| {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                return None;
            }
            let mut pieces = trimmed.split('x');
            let width = pieces.next()?.trim().parse::<u32>().ok()?;
            let height = pieces.next()?.trim().parse::<u32>().ok()?;
            Some((width.max(1), height.max(1)))
        })
        .collect()
}
