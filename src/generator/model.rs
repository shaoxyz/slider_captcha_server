use std::sync::Arc;

use image::GenericImageView;
use base64::Engine;

use crate::puzzle::SliderPuzzle;

#[derive(Clone)]
pub struct PuzzleImages {
    pub puzzle_b64: Arc<String>,
    pub piece_b64: Arc<String>,
    pub slider: Arc<SliderPuzzle>,
}

#[derive(Clone, Debug)]
pub struct CachedSolution {
    pub solution: f64,
    pub expires_at: u64,
    pub attempts: u32,  // 尝试次数
}

pub fn image_to_base64(image: image::DynamicImage) -> String {
    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
    use image::ColorType;

    let mut buffer = Vec::new();

    let encoder =
        PngEncoder::new_with_quality(&mut buffer, CompressionType::Default, FilterType::Sub);

    let (width, height) = image.dimensions();
    let color_type = match &image {
        image::DynamicImage::ImageRgb8(_) => ColorType::Rgb8,
        image::DynamicImage::ImageRgba8(_) => ColorType::Rgba8,
        _ => ColorType::Rgba8,
    };

    encoder
        .encode(image.as_bytes(), width, height, color_type)
        .expect("Failed to encode PNG");

    base64::engine::general_purpose::STANDARD.encode(&buffer)
}
