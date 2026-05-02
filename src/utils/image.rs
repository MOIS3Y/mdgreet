use crate::config;
use anyhow::{Context, Result};
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

    let cache_dir = get_cache_dir()?;
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

fn get_cache_dir() -> Result<PathBuf> {
    let uid = unsafe { libc::getuid() };
    if uid == 0 {
        Ok(PathBuf::from(config::CACHE_DIR))
    } else {
        Ok(PathBuf::from(".cache"))
    }
}
