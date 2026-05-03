use crate::utils;
use anyhow::{Context, Result};
use material_colors::color::Argb;
use material_colors::quantize::Quantizer;
use material_colors::quantize::QuantizerCelebi;
use material_colors::score::Score;
use std::fs;
use std::path::{Path, PathBuf};

pub fn prepare_background(original_path: &str, blur_sigma: f32) -> Result<PathBuf> {
    let original = Path::new(original_path);
    if !original.exists() {
        return Err(anyhow::anyhow!(
            "Original background not found: {}",
            original_path
        ));
    }

    let cache_dir = utils::cache::get_cache_dir();
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;
    }

    // Generate a unique filename based on the original path and blur value
    let file_name = original.file_name().unwrap().to_string_lossy();
    let cache_file_name = format!("{}_blur_{}.png", file_name, (blur_sigma * 10.0) as i32);
    let cache_path = cache_dir.join(cache_file_name);

    // Check if we need to re-process
    let needs_update = if cache_path.exists() {
        let original_meta = fs::metadata(original)?;
        let cache_meta = fs::metadata(&cache_path)?;
        original_meta.modified()? > cache_meta.modified()?
    } else {
        true
    };

    if needs_update {
        println!(
            "Processing background image: {} (blur: {})",
            original_path, blur_sigma
        );
        let img = image::open(original).context("Failed to open original image")?;

        // Optimization: For background blur, we don't need full resolution.
        // Resizing first makes blur calculation MUCH faster.
        let processed = if blur_sigma > 0.0 {
            // Resize to a smaller width while maintaining aspect ratio
            let target_width = 800;
            let resized = img.resize(
                target_width,
                (target_width as f32 * (img.height() as f32 / img.width() as f32)) as u32,
                image::imageops::FilterType::Triangle,
            );
            resized.blur(blur_sigma)
        } else {
            img
        };

        processed
            .save(&cache_path)
            .context("Failed to save blurred image to cache")?;
        println!("Background processed and saved to {:?}", cache_path);
    }

    Ok(cache_path)
}

/// Extracts the seed color from an image for dynamic theming.
/// Follows official material-colors optimization guidelines.
pub fn extract_seed_color(path: &str) -> Result<[u8; 4]> {
    let img = image::open(path).context("Failed to open image for color extraction")?;

    // Optimization: Resize to 128x128 with Lanczos3 filter as recommended
    let resized = img.resize_exact(128, 128, image::imageops::FilterType::Lanczos3);
    let rgb = resized.to_rgba8();

    // Convert pixels to Argb format for material-colors quantizer
    let mut pixels = Vec::with_capacity(128 * 128);
    for pixel in rgb.pixels() {
        let [r, g, b, a] = pixel.0;
        pixels.push(Argb::new(a, r, g, b));
    }

    // Quantize colors
    let colors = QuantizerCelebi::quantize(&pixels, 128);

    // Score colors and get the best one
    // Signature for 0.4.2: score(map, top_count, fallback, filter)
    let ranked = Score::score(&colors.color_to_count, Some(1), None, None);
    let best_color = ranked.get(0).cloned().unwrap_or(Argb::from_u32(0xFF445E91));

    Ok([
        best_color.red,
        best_color.green,
        best_color.blue,
        best_color.alpha,
    ])
}
