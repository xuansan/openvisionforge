use std::{
    env, fs,
    path::{Path, PathBuf},
};

use image::{GrayImage, Luma};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<_>>();
    let input_dir = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or("demo/input".into());
    let output_dir = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or("demo/output".into());
    let threshold = args
        .get(3)
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(128);

    fs::create_dir_all(&output_dir)?;

    let mut lines = vec![format!(
        "file,width,height,black_pixels,white_pixels,black_ratio,white_ratio,output"
    )];

    for entry in fs::read_dir(&input_dir)? {
        let path = entry?.path();
        if !is_image(&path) {
            continue;
        }

        let image = image::open(&path)?.to_luma8();
        let (bw, black, white) = threshold_image(&image, threshold);
        let file_stem = path.file_stem().and_then(|v| v.to_str()).unwrap_or("image");
        let output_path = output_dir.join(format!("{file_stem}_bw.png"));
        bw.save(&output_path)?;

        let total = black + white;
        let black_ratio = black as f64 / total as f64;
        let white_ratio = white as f64 / total as f64;

        println!(
            "{}: {}x{}, black {:.2}%, white {:.2}% -> {}",
            path.display(),
            image.width(),
            image.height(),
            black_ratio * 100.0,
            white_ratio * 100.0,
            output_path.display()
        );
        lines.push(format!(
            "{},{},{},{},{},{:.6},{:.6},{}",
            path.display(),
            image.width(),
            image.height(),
            black,
            white,
            black_ratio,
            white_ratio,
            output_path.display()
        ));
    }

    fs::write(output_dir.join("report.csv"), lines.join("\n"))?;
    Ok(())
}

fn threshold_image(image: &GrayImage, threshold: u8) -> (GrayImage, u64, u64) {
    let mut out = GrayImage::new(image.width(), image.height());
    let mut black = 0;
    let mut white = 0;

    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel[0] < threshold {
            out.put_pixel(x, y, Luma([0]));
            black += 1;
        } else {
            out.put_pixel(x, y, Luma([255]));
            white += 1;
        }
    }

    (out, black, white)
}

fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|v| v.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "jpg" | "jpeg" | "png"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use image::{GrayImage, Luma};

    use super::threshold_image;

    #[test]
    fn thresholds_black_and_white() {
        let mut image = GrayImage::new(2, 1);
        image.put_pixel(0, 0, Luma([10]));
        image.put_pixel(1, 0, Luma([200]));

        let (_out, black, white) = threshold_image(&image, 128);

        assert_eq!((black, white), (1, 1));
    }
}
