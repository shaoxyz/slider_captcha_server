extern crate slider_captcha_server;
use actix_web::{
    get, middleware, post, rt,
    web::{self, Data, Query},
    App, HttpResponse, HttpServer, Responder,
};
use dashmap::DashMap;
use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};
use serde_json::json;
use slider_captcha_server::{verify_puzzle, SliderPuzzle};
use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = State::new();

    // Start background cleanup task
    let cleanup_state = app_state.clone();
    rt::spawn(async move {
        cleanup_task(cleanup_state).await;
    });

    println!("\n🚀 Production Slider Captcha Server Started");
    println!("   Listen address: 0.0.0.0:8080");
    println!("   Worker threads: 4");
    println!("   Cache expiration: 10 minutes");
    println!("   Background cleanup: every 60 seconds\n");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_state.clone()))
            // Add logging middleware
            .wrap(middleware::Logger::default())
            // Add compression middleware
            .wrap(middleware::Compress::default())
            .service(generate_handler)
            .service(verify_handler)
            .service(health_check)
    })
    .bind("0.0.0.0:8080")?
    .workers(4) // 4 worker processes, can adjust based on CPU cores
    .run()
    .await
}

// Background cleanup task, clears expired cache every 60 seconds
async fn cleanup_task(state: State) {
    use actix_web::rt::time;
    let mut interval = time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Clean up expired entries
        state.solutions.retain(|_, entry| entry.expires_at > now);

        let count = state.solutions.len();
        println!("🧹 Background cleanup completed, current cache count: {count}");
    }
}

#[derive(Deserialize)]
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

#[get("/puzzle")]
async fn generate_handler(state: Data<State>, query: Query<PuzzleQuery>) -> impl Responder {
    // Generate random image
    let slider_puzzle: SliderPuzzle = match SliderPuzzle::from_dimensions(query.w, query.h) {
        Ok(puzzle) => puzzle,
        Err(err) => {
            eprintln!("❌ Failed to generate image: {err}");
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to generate puzzle"
            }));
        }
    };

    // Generate unique ID and store solution
    let request_id = uuid::Uuid::new_v4().to_string();
    let solution = slider_puzzle.x;
    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 600; // 10 minutes

    state.solutions.insert(
        request_id.clone(),
        CacheEntry {
            solution,
            expires_at,
        },
    );

    let response = json!({
        "puzzle_image": image_to_base64(slider_puzzle.cropped_puzzle),
        "piece_image": image_to_base64(slider_puzzle.puzzle_piece),
        "id": request_id,
        "y": slider_puzzle.y,
    });

    HttpResponse::Ok().json(response)
}

#[post("/puzzle/solution")]
async fn verify_handler(state: Data<State>, solution: web::Json<Solution>) -> impl Responder {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Get cached solution
    let entry = match state.solutions.get(&solution.id) {
        Some(entry) => entry,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid request ID"
            }));
        }
    };

    // Check if expired
    if entry.expires_at <= now {
        state.solutions.remove(&solution.id);
        return HttpResponse::BadRequest().json(json!({
            "error": "Captcha expired"
        }));
    }

    let correct_solution = entry.solution;

    // Delete immediately after use (one-time captcha)
    state.solutions.remove(&solution.id);

    if verify_puzzle(correct_solution, solution.x, 0.01) {
        HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Verification successful"
        }))
    } else {
        HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "Verification failed"
        }))
    }
}

// Health check endpoint
#[get("/health")]
async fn health_check(state: Data<State>) -> impl Responder {
    let cache_size = state.solutions.len();
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "cache_size": cache_size,
        "uptime": "running"
    }))
}

// Use DashMap for lock-free concurrent cache
#[derive(Clone)]
struct CacheEntry {
    solution: f64,
    expires_at: u64,
}

#[derive(Clone)]
struct State {
    solutions: Arc<DashMap<String, CacheEntry>>,
}

impl State {
    fn new() -> Self {
        State {
            solutions: Arc::new(DashMap::new()),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Solution {
    pub id: String,
    pub x: f64,
}

fn image_to_base64(image: DynamicImage) -> String {
    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
    use image::ColorType;

    let mut buffer = Vec::new();

    let encoder = PngEncoder::new_with_quality(&mut buffer, CompressionType::Best, FilterType::Sub);

    let (width, height) = image.dimensions();
    let color_type = match &image {
        DynamicImage::ImageRgb8(_) => ColorType::Rgb8,
        DynamicImage::ImageRgba8(_) => ColorType::Rgba8,
        _ => ColorType::Rgba8,
    };

    encoder
        .encode(image.as_bytes(), width, height, color_type)
        .unwrap();

    base64::encode(&buffer)
}
