use std::io::Cursor;
use std::sync::Arc;

use bytes::Bytes;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use log::info;

use crate::config::app_config::{AppConfig, Format};

#[derive(Copy, Clone)]
pub struct UpscalerConfig {
    threshold_enabled: bool,
    threshold: u32,
    threshold_png: u32,
    return_format: Format,
}

pub trait Upscaler: Send {
    fn upscale(&self, input: Bytes, image_format: ImageFormat) -> (Bytes, ImageFormat) {
        let config = self.get_config();
        if config.threshold_enabled {
            let input_kb = (input.len() / 1024) as u32;
            let threshold = if image_format == ImageFormat::Png { config.threshold_png } else { config.threshold };
            if input_kb > threshold {
                info!("image size {} is bigger than threshold {}. skipping upscale", input_kb, threshold);
                return (input, image_format);
            }
        }

        let mut reader = image::io::Reader::new(Cursor::new(input.clone()));
        reader.set_format(image_format);
        let image = reader.decode()
            .or(image::io::Reader::new(Cursor::new(input))
                .with_guessed_format().unwrap().decode()
            ).unwrap();

        let upscaled = self.upscale_image(image);
        let mut buf = Cursor::new(Vec::new());

        let format_to = match config.return_format {
            Format::Png => ImageFormat::Png,
            Format::Jpeg => ImageFormat::Jpeg,
            Format::WebP => ImageFormat::WebP,
            Format::Avif => ImageFormat::Avif,
            Format::Original => image_format,
        };

        upscaled.write_to(&mut buf, format_to).expect("can't write image");
        (Bytes::from(buf.into_inner()), format_to)
    }

    fn upscale_image(&self, image: DynamicImage) -> DynamicImage;

    fn get_config(&self) -> UpscalerConfig;
}

pub struct Lanczos3Upscaler {
    config: UpscalerConfig,
    scale: u32,
}

impl Lanczos3Upscaler {
    pub fn new(config: Arc<AppConfig>) -> Self {
        let upscaler_config = UpscalerConfig {
            threshold_enabled: config.size_threshold_enabled,
            threshold: config.size_threshold,
            threshold_png: config.size_threshold_png,
            return_format: config.return_format,
        };

        Self {
            config: upscaler_config,
            scale: 2,
        }
    }
}

impl Upscaler for Lanczos3Upscaler {
    fn upscale_image(&self, image: DynamicImage) -> DynamicImage {
        let width = image.width() * self.scale;
        let height = image.height() * self.scale;
        image.resize_exact(width, height, FilterType::Lanczos3)
    }

    fn get_config(&self) -> UpscalerConfig {
        self.config
    }
}
