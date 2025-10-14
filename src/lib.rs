use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use rand::Rng;
// Import the slider captcha library

#[derive(Debug, Clone)]
pub struct SliderPuzzle {
    pub cropped_puzzle: image::DynamicImage,
    pub puzzle_piece: image::DynamicImage,
    pub x: f64,
    pub y: f64,
}

impl SliderPuzzle {
    // Generate random gradient image (optimized for storage size)
    pub fn generate_random_image(width: u32, height: u32) -> DynamicImage {
        let mut rng = rand::thread_rng();
        let mut image = DynamicImage::new_rgba8(width, height);

        // Randomly generate 2-3 colors for gradient
        let r1 = rng.gen_range(100..=255);
        let g1 = rng.gen_range(100..=255);
        let b1 = rng.gen_range(100..=255);

        let r2 = rng.gen_range(50..=200);
        let g2 = rng.gen_range(50..=200);
        let b2 = rng.gen_range(50..=200);

        // Gradient direction: 0=horizontal, 1=vertical, 2=diagonal
        let gradient_type = rng.gen_range(0..3);

        for y in 0..height {
            for x in 0..width {
                let ratio = match gradient_type {
                    0 => x as f32 / width as f32,  // Horizontal gradient
                    1 => y as f32 / height as f32, // Vertical gradient
                    _ => ((x as f32 / width as f32) + (y as f32 / height as f32)) / 2.0, // Diagonal gradient
                };

                let r = (r1 as f32 * (1.0 - ratio) + r2 as f32 * ratio) as u8;
                let g = (g1 as f32 * (1.0 - ratio) + g2 as f32 * ratio) as u8;
                let b = (b1 as f32 * (1.0 - ratio) + b2 as f32 * ratio) as u8;

                image.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }

        // Add 2-4 simple geometric shape decorations
        let num_shapes = rng.gen_range(2..=4);
        for _ in 0..num_shapes {
            let shape_r = rng.gen_range(0..=255);
            let shape_g = rng.gen_range(0..=255);
            let shape_b = rng.gen_range(0..=255);
            let alpha = rng.gen_range(100..=200); // Semi-transparent

            let cx = rng.gen_range(0..width);
            let cy = rng.gen_range(0..height);
            let radius = rng.gen_range(20..60);

            // Draw circle
            for y in 0..height {
                for x in 0..width {
                    let dx = (x as i32 - cx as i32).abs();
                    let dy = (y as i32 - cy as i32).abs();
                    let dist_sq = (dx * dx + dy * dy) as f32;
                    let radius_sq = (radius * radius) as f32;

                    if dist_sq <= radius_sq {
                        // Alpha blending
                        let old_pixel = image.get_pixel(x, y);
                        let blend = alpha as f32 / 255.0;
                        let r = ((shape_r as f32 * blend) + (old_pixel[0] as f32 * (1.0 - blend)))
                            as u8;
                        let g = ((shape_g as f32 * blend) + (old_pixel[1] as f32 * (1.0 - blend)))
                            as u8;
                        let b = ((shape_b as f32 * blend) + (old_pixel[2] as f32 * (1.0 - blend)))
                            as u8;

                        image.put_pixel(x, y, Rgba([r, g, b, 255]));
                    }
                }
            }
        }

        image
    }

    pub fn new(image_path: &str) -> Result<SliderPuzzle, String> {
        let input_image = match image::open(image_path) {
            Ok(image) => image,
            Err(e) => return Err(e.to_string()),
        };
        let (width, height) = input_image.dimensions();
        // Define the size of the puzzle piece.
        let piece_width = width / 5;
        let piece_height = height / 5;
        // Choose a random starting position for the puzzle piece.
        let mut rng = rand::thread_rng();
        let start_x = rng.gen_range(0..(width - piece_width));
        let start_y = rng.gen_range(piece_height..(2 * piece_height));

        // Crop the puzzle piece out of the input image.
        let mut puzzle_piece = DynamicImage::new_rgb8(piece_width, piece_height);
        for y in 0..piece_height {
            for x in 0..piece_width {
                let pixel = input_image.get_pixel(start_x + x, start_y + y);
                let rgba_pixel = Rgba([pixel[0], pixel[1], pixel[2], pixel[3]]);
                puzzle_piece.put_pixel(x, y, rgba_pixel);
            }
        }
        // Create a blank canvas for the cropped image.
        let mut cropped_image = DynamicImage::new_rgba8(width, height);
        for y in 0..height {
            for x in 0..width {
                let pixel = input_image.get_pixel(x, y);
                let mut rgba_pixel = Rgba([pixel[0], pixel[1], pixel[2], pixel[3]]);
                if x >= start_x
                    && x < start_x + piece_width
                    && y >= start_y
                    && y < start_y + piece_height
                {
                    rgba_pixel[3] = 0;
                }
                cropped_image.put_pixel(x, y, rgba_pixel);
            }
        }

        Ok(SliderPuzzle {
            cropped_puzzle: cropped_image,
            puzzle_piece,
            y: (start_y as f64 / height as f64),
            x: (start_x as f64 / width as f64),
        })
    }
    // Create puzzle from randomly generated image
    pub fn from_dimensions(width: u32, height: u32) -> Result<SliderPuzzle, String> {
        let input_image = Self::generate_random_image(width, height);

        // Define the size of the puzzle piece.
        let piece_width = width / 5;
        let piece_height = height / 5;
        // Choose a random starting position for the puzzle piece.
        let mut rng = rand::thread_rng();
        let start_x = rng.gen_range(0..(width - piece_width));
        let start_y = rng.gen_range(piece_height..(2 * piece_height));

        // Crop the puzzle piece out of the input image.
        let mut puzzle_piece = DynamicImage::new_rgb8(piece_width, piece_height);
        for y in 0..piece_height {
            for x in 0..piece_width {
                let pixel = input_image.get_pixel(start_x + x, start_y + y);
                let rgba_pixel = Rgba([pixel[0], pixel[1], pixel[2], pixel[3]]);
                puzzle_piece.put_pixel(x, y, rgba_pixel);
            }
        }
        // Create a blank canvas for the cropped image.
        let mut cropped_image = DynamicImage::new_rgba8(width, height);
        for y in 0..height {
            for x in 0..width {
                let pixel = input_image.get_pixel(x, y);
                let mut rgba_pixel = Rgba([pixel[0], pixel[1], pixel[2], pixel[3]]);
                if x >= start_x
                    && x < start_x + piece_width
                    && y >= start_y
                    && y < start_y + piece_height
                {
                    rgba_pixel[3] = 0;
                }
                cropped_image.put_pixel(x, y, rgba_pixel);
            }
        }

        Ok(SliderPuzzle {
            cropped_puzzle: cropped_image,
            puzzle_piece,
            y: (start_y as f64 / height as f64),
            x: (start_x as f64 / width as f64),
        })
    }

    pub fn update_x(&mut self, x: f64) -> &mut SliderPuzzle {
        self.x = x;
        self
    }
    pub fn get_x(&self) -> f64 {
        self.x
    }
}

pub fn verify_puzzle(solution: f64, submission: f64, error_margin: f64) -> bool {
    (solution - submission).abs() < error_margin
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generate_puzzle() {
        let binding = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test")
            .join("archworkout.png");
        let input_path = binding.to_str().unwrap();
        let slider_puzzle = SliderPuzzle::new(input_path).unwrap();
        let puzzle_piece_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test")
            .join("piece.png")
            .to_str()
            .unwrap()
            .to_owned();
        let cropped_puzzle_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test")
            .join("puzzle.png")
            .to_str()
            .unwrap()
            .to_owned();
        slider_puzzle
            .puzzle_piece
            .save(puzzle_piece_path.clone())
            .unwrap();
        slider_puzzle
            .cropped_puzzle
            .save(cropped_puzzle_path.clone())
            .unwrap();
        println!("SOLUTION:\nx: {:?}\ny: {:?}\n", slider_puzzle.x, slider_puzzle.y);
        println!("Check images at:\n{cropped_puzzle_path:?}\n{puzzle_piece_path:?}");
    }
    #[test]
    fn playground() {
        let x: u32 = 100;
        let y: f32 = 1.23;

        let z: f32 = x as f32 + y;
        println!("{z}");

        assert!(z == 101.23)
    }
}
