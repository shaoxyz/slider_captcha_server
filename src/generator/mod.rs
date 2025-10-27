use std::{sync::Arc, time::Instant};

use dashmap::DashMap;
use tokio::{spawn, sync::{mpsc, Semaphore}, task::spawn_blocking};

use crate::{cache::ExpiringCache, config::AppConfig, puzzle::SliderPuzzle};

mod model;

pub use model::{CachedSolution, PuzzleImages};

#[derive(Clone)]
pub struct PuzzleGenerator {
    cache: ExpiringCache<(u32, u32), PuzzleImages>,
    request_tx: mpsc::Sender<GenerateRequest>,
    solutions: DashMap<String, CachedSolution>,
}

struct GenerateRequest {
    dimensions: (u32, u32),
    response: mpsc::Sender<Arc<PuzzleImages>>,
}

impl PuzzleGenerator {
    pub fn new(config: Arc<AppConfig>) -> Self {
        let cache = ExpiringCache::new(config.puzzle_ttl, config.cache_max_per_size);
        let (tx, rx) = mpsc::channel::<GenerateRequest>(config.cache_max_per_size * 4);
        let semaphore = Arc::new(Semaphore::new(config.generator_concurrency));

        let cache_dispatch = cache.clone();
        let semaphore_dispatch = semaphore.clone();

        spawn(async move {
            let mut rx = rx;
            while let Some(GenerateRequest { dimensions, response }) = rx.recv().await {
                let cache = cache_dispatch.clone();
                let semaphore = semaphore_dispatch.clone();

                spawn(async move {
                    let start = Instant::now();
                    let permit = match semaphore.acquire_owned().await {
                        Ok(permit) => permit,
                        Err(err) => {
                            tracing::error!(error=?err, "Failed to acquire semaphore");
                            return;
                        }
                    };

                    let generation = spawn_blocking(move || generate_puzzle(dimensions));

                    match generation.await {
                        Ok(Ok(images)) => {
                            cache.insert(dimensions, images.clone());
                            let arc = Arc::new(images);
                            if let Err(err) = response.send(arc).await {
                                tracing::warn!(dimensions=?dimensions, error=?err, "Failed to deliver generated puzzle");
                            }

                            tracing::info!(
                                dimensions=?dimensions,
                                elapsed_ms = start.elapsed().as_millis(),
                                "Background puzzle generated"
                            );
                        }
                        Ok(Err(err)) => {
                            tracing::error!(dimensions=?dimensions, error=?err, "Failed to generate slider puzzle");
                        }
                        Err(err) => {
                            tracing::error!(dimensions=?dimensions, error=?err, "Generation task join error");
                        }
                    }

                    drop(permit);
                });
            }
        });

        Self {
            cache,
            request_tx: tx,
            solutions: DashMap::new(),
        }
    }

    pub async fn get_puzzle(&self, width: u32, height: u32) -> Option<Arc<PuzzleImages>> {
        if let Some(images) = self.cache.pop(&(width, height)) {
            return Some(images);
        }

        let (tx, mut rx) = mpsc::channel(1);

        let request = GenerateRequest {
            dimensions: (width, height),
            response: tx,
        };

        if let Err(err) = self.request_tx.send(request).await {
            tracing::error!(%width, %height, error=?err, "Failed to enqueue generation request");
            return None;
        }

        rx.recv().await
    }

    pub fn cache_solution(&self, id: String, solution: f64, expires_at: u64) {
        self.solutions.insert(
            id,
            CachedSolution {
                solution,
                expires_at,
                attempts: 0,
            },
        );
    }

    pub fn get_solution(&self, id: &str) -> Option<CachedSolution> {
        self.solutions.get(id).map(|entry| entry.value().clone())
    }

    pub fn increment_attempts(&self, id: &str) -> Option<u32> {
        if let Some(mut entry) = self.solutions.get_mut(id) {
            entry.attempts += 1;
            Some(entry.attempts)
        } else {
            None
        }
    }

    pub fn remove_solution(&self, id: &str) -> Option<CachedSolution> {
        self.solutions.remove(id).map(|(_, value)| value)
    }

    // 保持向后兼容
    pub fn take_solution(&self, id: &str) -> Option<CachedSolution> {
        self.remove_solution(id)
    }

    pub fn cache_len(&self, key: &(u32, u32)) -> usize {
        self.cache.len_for(key)
    }

    pub fn total_cached(&self) -> usize {
        self.cache.total_len()
    }

    pub fn cleanup(&self) -> (usize, usize) {
        self.cache.clean_expired()
    }
}

fn generate_puzzle(dimensions: (u32, u32)) -> Result<PuzzleImages, String> {
    let (width, height) = dimensions;
    let slider_puzzle = SliderPuzzle::from_dimensions(width, height)?;

    let puzzle_b64 = Arc::new(model::image_to_base64(slider_puzzle.cropped_puzzle.clone()));
    let piece_b64 = Arc::new(model::image_to_base64(slider_puzzle.puzzle_piece.clone()));

    Ok(PuzzleImages {
        puzzle_b64,
        piece_b64,
        slider: Arc::new(slider_puzzle),
    })
}
