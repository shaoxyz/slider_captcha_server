use std::{
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use actix_web::{get, rt::{spawn, time}, web, App, HttpResponse, HttpServer, Responder};
use tokio::task::spawn_blocking;
use tracing::{info, warn};

use crate::{config::AppConfig, generator::PuzzleGenerator, puzzle::verify_puzzle};

#[derive(Clone)]
pub struct AppState {
    pub generator: Arc<PuzzleGenerator>,
    pub config: Arc<AppConfig>,
}

#[derive(serde::Deserialize)]
struct PuzzleQuery {
    #[serde(default = "default_height")]
    h: u32,
    #[serde(default = "default_width")]
    w: u32,
}

fn default_height() -> u32 {
    300
}

fn default_width() -> u32 {
    500
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SolutionPayload {
    id: String,
    x: f64,
}

#[get("/puzzle")]
async fn puzzle_handler(
    state: web::Data<AppState>,
    query: web::Query<PuzzleQuery>,
) -> impl Responder {
    let request_start = Instant::now();

    let width = query.w.max(100);
    let height = query.h.max(100);
    info!(%width, %height, "Incoming puzzle request");

    match state.generator.get_puzzle(width, height).await {
        Some(images) => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let id = uuid::Uuid::new_v4().to_string();
            let expires_at = now + state.config.solution_ttl.as_secs();

            let solution = images.slider.x;

            state
                .generator
                .cache_solution(id.clone(), solution, expires_at);

            let response = serde_json::json!({
                "puzzle_image": &*images.puzzle_b64,
                "piece_image": &*images.piece_b64,
                "id": &id,
                "y": images.slider.y,
            });

            info!(
                %width,
                %height,
                elapsed_ms = request_start.elapsed().as_millis(),
                cache_size = state.generator.cache_len(&(width, height)),
                "Puzzle served"
            );

            HttpResponse::Ok().json(response)
        }
        None => {
            warn!(%width, %height, "No puzzle available");
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "Puzzle generation busy, try again later"
            }))
        }
    }
}

#[actix_web::post("/puzzle/solution")]
async fn verify_handler(
    state: web::Data<AppState>,
    payload: web::Json<SolutionPayload>,
) -> impl Responder {
    let request_start = Instant::now();
    let id = payload.id.clone();
    let x = payload.x;

    match state.generator.get_solution(&id) {
        Some(entry) => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if entry.expires_at <= now {
                warn!(%id, "Solution expired");
                state.generator.remove_solution(&id);
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Captcha expired"
                }))
            } else if verify_puzzle(entry.solution, x, 0.015) {
                info!(id = %id, elapsed_ms = request_start.elapsed().as_millis(), "Captcha solved");
                
                // 根据配置决定是否立即删除缓存
                if state.config.immediate_cache_cleanup {
                    state.generator.remove_solution(&id);
                }
                
                HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": "Verification successful"
                }))
            } else {
                // 验证失败，增加尝试次数
                let attempts = state.generator.increment_attempts(&id).unwrap_or(0);
                warn!(id = %id, submitted = %x, expected = %entry.solution, attempts = %attempts, "Incorrect solution");
                
                if attempts >= 5 {
                    // 5次失败后删除solution
                    state.generator.remove_solution(&id);
                    HttpResponse::BadRequest().json(serde_json::json!({
                        "success": false,
                        "error": "Too many failed attempts, please request a new captcha",
                        "attempts": attempts
                    }))
                } else {
                    HttpResponse::BadRequest().json(serde_json::json!({
                        "success": false,
                        "error": "Verification failed",
                        "attempts": attempts,
                        "remaining": 5 - attempts
                    }))
                }
            }
        }
        None => {
            warn!(%id, "Unknown solution id");
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid request ID"
            }))
        }
    }
}

#[get("/health")]
async fn health_handler(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "prefill_sizes": state.config.prefill_dimensions,
    }))
}

pub async fn run_server(config: Arc<AppConfig>) -> std::io::Result<()> {
    let generator = Arc::new(PuzzleGenerator::new(config.clone()));
    generator.fill_cache(&config);

    let state = AppState {
        generator: generator.clone(),
        config: config.clone(),
    };

    let cleanup_config = config.clone();
    let cleanup_generator = generator.clone();

    spawn(async move {
        let mut interval = time::interval(cleanup_config.cleanup_interval);
        loop {
            interval.tick().await;
            let cleanup_generator = cleanup_generator.clone();
            if let Err(err) = spawn_blocking(move || {
                let (removed, remaining) = cleanup_generator.cleanup();
                tracing::info!(removed, remaining, "Cache cleanup completed");
            })
            .await
            {
                tracing::error!(error=?err, "Cleanup task panic");
            }
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(puzzle_handler)
            .service(verify_handler)
            .service(health_handler)
    })
    .bind((config.host.clone(), config.port))?
    .workers(config.workers)
    .run()
    .await
}
