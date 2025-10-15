use std::sync::Arc;

use slider_captcha_server::{config::AppConfig, server::run_server};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Arc::new(AppConfig::from_env());

    tracing_subscriber::registry()
        .with(EnvFilter::new(config.log_level.clone()))
        .with(fmt::layer())
        .init();

    tracing::info!(
        host = %config.host,
        port = %config.port,
        workers = %config.workers,
        generator_concurrency = %config.generator_concurrency,
        cache_prefill = %config.cache_prefill_per_size,
        cache_max = %config.cache_max_per_size,
        "Slider captcha server starting"
    );

    run_server(config).await
}
