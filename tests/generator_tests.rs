use std::sync::Arc;

use slider_captcha_server::{config::AppConfig, generator::PuzzleGenerator};

fn test_config() -> AppConfig {
    AppConfig {
        host: "127.0.0.1".into(),
        port: 0,
        workers: 2,
        solution_ttl: std::time::Duration::from_secs(1),
        puzzle_ttl: std::time::Duration::from_secs(1),
        cache_prefill_per_size: 2,
        cache_max_per_size: 4,
        generator_concurrency: 2,
        cleanup_interval: std::time::Duration::from_secs(60),
        prefill_dimensions: vec![(200, 200)],
        log_level: "info".into(),
        immediate_cache_cleanup: false,
    }
}

#[tokio::test(flavor = "current_thread")]
async fn generator_get_puzzle_returns_value() {
    let config = Arc::new(test_config());
    let generator = PuzzleGenerator::new(config.clone());

    let puzzle = generator.get_puzzle(200, 200).await;
    assert!(puzzle.is_some());
}

