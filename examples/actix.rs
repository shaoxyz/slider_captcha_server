extern crate slider_captcha_server;
use actix_web::{
    get, post,
    web::{self, Data, Query},
    App, HttpResponse, HttpServer, Responder,
};
use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};
use serde_json::json;
use slider_captcha_server::{verify_puzzle, SliderPuzzle};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = State::default();

    println!("\nStarted slider_captcha_server on port 18080.\n");
    HttpServer::new(move || {
        App::new()
            .data(app_state.clone())
            .service(generate_handler)
            .service(verify_handler)
    })
    .bind("127.0.0.1:18080")?
    .run()
    .await
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
async fn generate_handler(state: web::Data<State>, query: Query<PuzzleQuery>) -> impl Responder {
    // Clean up expired cache first
    state.cleanup_expired();

    // Generate random image with provided width and height
    let slider_puzzle: SliderPuzzle = match SliderPuzzle::from_dimensions(query.w, query.h) {
        Ok(puzzle) => puzzle,
        Err(err) => {
            print!("!!! Failed to generate image !!!! \n{err}");
            return HttpResponse::InternalServerError().body("Contact Admin.");
        }
    };

    // Generate unique request ID and store solution with 10-minute expiration
    let request_id = uuid::Uuid::new_v4().to_string();
    let solution = slider_puzzle.x;
    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 600; // 10 minutes = 600 seconds

    state.solutions.lock().unwrap().insert(
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

    println!(
        "\nSOLUTION:\nid:{:?},\nx:{:?},y:{:?}",
        request_id, slider_puzzle.x, slider_puzzle.y
    );
    HttpResponse::Ok().json(response)
}

#[post("/puzzle/solution")]
async fn verify_handler(state: Data<State>, solution: web::Json<Solution>) -> impl Responder {
    // Clean up expired cache first
    state.cleanup_expired();

    // Check if solution matches
    let mut locked_state = state.solutions.lock().unwrap();

    let correct_solution = match locked_state.get(&solution.id) {
        Some(entry) => {
            // Check if expired
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if entry.expires_at <= now {
                locked_state.remove(&solution.id);
                return HttpResponse::BadRequest().body("Captcha expired");
            }

            println!(
                "SOLUTION:\nRequestID:{:?}\nx:{:?}\n",
                solution.id, entry.solution
            );
            entry.solution
        }
        _ => return HttpResponse::BadRequest().body("Invalid request ID"),
    };
    locked_state.remove(&solution.id);
    if verify_puzzle(correct_solution, solution.x, 0.01) {
        HttpResponse::Ok().body("VERIFIED!")
    } else {
        HttpResponse::BadRequest().body("Incorrect solution")
    }
}

// Cache entry containing solution and expiration time
#[derive(Clone)]
struct CacheEntry {
    solution: f64,
    expires_at: u64, // Unix timestamp (seconds)
}

// A struct to store the global state of the application
#[derive(Clone)]
struct State {
    solutions: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl State {
    // Clean up expired cache entries
    fn cleanup_expired(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut solutions = self.solutions.lock().unwrap();
        solutions.retain(|_, entry| entry.expires_at > now);
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            solutions: Arc::new(Mutex::new(HashMap::new())),
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

    // Use PNG encoder with highest compression level
    let encoder = PngEncoder::new_with_quality(
        &mut buffer,
        CompressionType::Best, // Highest compression level
        FilterType::Sub,       // Sub filter works well for gradient images
    );

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
