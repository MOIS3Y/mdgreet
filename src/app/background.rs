use crate::config::BackgroundConfig;
use crate::utils;
use slint::Image;
use std::path::Path;

pub struct Background {
    pub original: Image,
    pub blurred: Image,
}

impl Background {
    pub fn load(config: &BackgroundConfig) -> Self {
        let path_str = config.path.as_deref().unwrap_or("ui/images/background.png");
        let blur = config.blur.unwrap_or(10.0);

        let original = Image::load_from_path(Path::new(path_str)).unwrap_or_else(|e| {
            eprintln!("background: failed to load original image: {}", e);
            Image::default()
        });

        let blurred = match utils::image::prepare_background(path_str, blur) {
            Ok(cached_path) => Image::load_from_path(&cached_path).unwrap_or_else(|e| {
                eprintln!("background: failed to load blurred image from cache: {}", e);
                Image::default()
            }),
            Err(e) => {
                eprintln!("background: failed to prepare blurred background: {}", e);
                Image::default()
            }
        };

        Self { original, blurred }
    }
}
