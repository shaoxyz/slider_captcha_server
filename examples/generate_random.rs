extern crate slider_captcha_server;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ColorType, GenericImageView};
use slider_captcha_server::SliderPuzzle;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("Starting to generate random gradient puzzle...\n");

    // Use default dimensions 500x300
    let width = 500;
    let height = 300;

    println!("Image dimensions: {width}x{height}");

    // Generate puzzle
    let slider_puzzle = match SliderPuzzle::from_dimensions(width, height) {
        Ok(puzzle) => puzzle,
        Err(err) => {
            eprintln!("Failed to generate puzzle: {err}");
            return;
        }
    };

    // Save generated images (using highest compression level)
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test");

    let puzzle_path = output_dir.join("generated_puzzle.png");
    let piece_path = output_dir.join("generated_piece.png");

    // Save using optimized PNG encoder
    save_optimized_png(&slider_puzzle.cropped_puzzle, &puzzle_path);
    save_optimized_png(&slider_puzzle.puzzle_piece, &piece_path);

    // Get file sizes
    let puzzle_size = fs::metadata(&puzzle_path).map(|m| m.len()).unwrap_or(0);
    let piece_size = fs::metadata(&piece_path).map(|m| m.len()).unwrap_or(0);

    println!(
        "✓ Puzzle background saved: {:?} ({}KB)",
        puzzle_path,
        puzzle_size / 1024
    );
    println!(
        "✓ Puzzle piece saved: {:?} ({}KB)",
        piece_path,
        piece_size / 1024
    );

    println!("\nSolution information:");
    println!("- X position (relative): {:.4}", slider_puzzle.x);
    println!("- Y position (relative): {:.4}", slider_puzzle.y);
    println!("- X position (pixels): {:.0}px", slider_puzzle.x * width as f64);
    println!("- Y position (pixels): {:.0}px", slider_puzzle.y * height as f64);

    println!("\nFile size statistics:");
    println!("- Total size: {}KB", (puzzle_size + piece_size) / 1024);
    println!(
        "- Approx. after Base64 encoding: {}KB",
        (puzzle_size + piece_size) * 4 / 3 / 1024
    );

    println!("\nGeneration completed! Please check the image files in the test directory.");
}

fn save_optimized_png(image: &image::DynamicImage, path: &PathBuf) {
    let file = fs::File::create(path).unwrap();
    let encoder = PngEncoder::new_with_quality(
        file,
        CompressionType::Best,
        FilterType::Sub, // Sub filter usually works well for gradient images
    );

    let (width, height) = image.dimensions();
    let color_type = match image {
        image::DynamicImage::ImageRgb8(_) => ColorType::Rgb8,
        image::DynamicImage::ImageRgba8(_) => ColorType::Rgba8,
        _ => ColorType::Rgba8,
    };

    encoder
        .encode(image.as_bytes(), width, height, color_type)
        .unwrap();
}
