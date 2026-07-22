use std::path::Path;

use image::{imageops::FilterType, ImageBuffer, Rgb, RgbImage};
use vision_core::{ImageFrame, Result, VisionError};

#[derive(Debug, Clone)]
pub(crate) struct ImageTensor {
    pub shape: [usize; 4],
    pub data: Vec<f32>,
    pub scale: f64,
    pub pad_x: f64,
    pub pad_y: f64,
    pub resized_width: u32,
    pub resized_height: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct PreprocessConfig {
    pub input_width: i64,
    pub input_height: i64,
    pub mean: [f32; 3],
    pub std: [f32; 3],
    pub letterbox: bool,
}

pub(crate) fn image_to_nchw(image: &ImageFrame, config: &PreprocessConfig) -> Result<ImageTensor> {
    let source_uri = image
        .source_uri
        .as_ref()
        .ok_or_else(|| VisionError::Operator {
            node: "infer.preprocess".to_string(),
            message: "image source_uri is required for ONNX Runtime preprocessing".to_string(),
        })?;
    let path = Path::new(source_uri);
    let dyn_image = image::open(path).map_err(|err| VisionError::Operator {
        node: "infer.preprocess".to_string(),
        message: format!("failed to read image {}: {err}", path.display()),
    })?;

    let rgb = dyn_image.to_rgb8();
    let (src_w, src_h) = rgb.dimensions();
    let dst_w = config.input_width.max(1) as u32;
    let dst_h = config.input_height.max(1) as u32;

    let (canvas, scale, pad_x, pad_y, resized_width, resized_height) = if config.letterbox {
        let scale = (dst_w as f64 / src_w as f64).min(dst_h as f64 / src_h as f64);
        let resized_width = (src_w as f64 * scale).round().max(1.0) as u32;
        let resized_height = (src_h as f64 * scale).round().max(1.0) as u32;
        let resized =
            image::imageops::resize(&rgb, resized_width, resized_height, FilterType::Triangle);
        let mut canvas: RgbImage = ImageBuffer::from_pixel(dst_w, dst_h, Rgb([114, 114, 114]));
        let pad_x = ((dst_w - resized_width) / 2) as u32;
        let pad_y = ((dst_h - resized_height) / 2) as u32;
        image::imageops::overlay(&mut canvas, &resized, pad_x.into(), pad_y.into());
        (
            canvas,
            scale,
            pad_x as f64,
            pad_y as f64,
            resized_width,
            resized_height,
        )
    } else {
        let resized = image::imageops::resize(&rgb, dst_w, dst_h, FilterType::Triangle);
        let scale = dst_w as f64 / src_w as f64;
        (resized, scale, 0.0, 0.0, dst_w, dst_h)
    };

    let plane = (dst_w * dst_h) as usize;
    let mut data = vec![0.0f32; plane * 3];
    for (x, y, pixel) in canvas.enumerate_pixels() {
        let index = (y * dst_w + x) as usize;
        for channel in 0..3 {
            let value = pixel[channel] as f32 / 255.0;
            data[channel * plane + index] = (value - config.mean[channel]) / config.std[channel];
        }
    }

    Ok(ImageTensor {
        shape: [1, 3, dst_h as usize, dst_w as usize],
        data,
        scale,
        pad_x,
        pad_y,
        resized_width,
        resized_height,
    })
}
