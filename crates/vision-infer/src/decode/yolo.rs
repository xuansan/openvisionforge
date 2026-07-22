use vision_core::{Detection, ImageFrame, Rect};

use crate::backend::onnx::OnnxRunOutput;

#[derive(Debug, Clone)]
pub(crate) struct YoloDecodeConfig {
    pub score_threshold: f64,
    pub nms_threshold: f64,
    pub labels: Vec<String>,
}

pub(crate) fn decode(
    output: &OnnxRunOutput,
    image: &ImageFrame,
    config: &YoloDecodeConfig,
) -> Vec<Detection> {
    let Some(tensor) = output.tensors.first() else {
        return Vec::new();
    };
    let mut detections = decode_tensor(tensor, image, output, config);
    detections.sort_by(|a, b| b.score.total_cmp(&a.score));
    nms(detections, config.nms_threshold)
}

fn decode_tensor(
    tensor: &crate::backend::onnx::TensorOutput,
    image: &ImageFrame,
    output: &OnnxRunOutput,
    config: &YoloDecodeConfig,
) -> Vec<Detection> {
    if tensor.shape.len() == 3 {
        let a = tensor.shape[1].max(0) as usize;
        let b = tensor.shape[2].max(0) as usize;
        if a >= 5 && b > a {
            return decode_rows_transposed(&tensor.data, a, b, image, output, config);
        }
        return decode_rows(&tensor.data, a, b, image, output, config);
    }
    if tensor.shape.len() == 2 {
        return decode_rows(
            &tensor.data,
            tensor.shape[0].max(0) as usize,
            tensor.shape[1].max(0) as usize,
            image,
            output,
            config,
        );
    }
    Vec::new()
}

fn decode_rows(
    data: &[f32],
    rows: usize,
    cols: usize,
    image: &ImageFrame,
    output: &OnnxRunOutput,
    config: &YoloDecodeConfig,
) -> Vec<Detection> {
    if rows == 0 || cols < 6 || data.len() < rows * cols {
        return Vec::new();
    }
    (0..rows)
        .filter_map(|row| {
            decode_prediction(&data[row * cols..row * cols + cols], image, output, config)
        })
        .collect()
}

fn decode_rows_transposed(
    data: &[f32],
    attrs: usize,
    rows: usize,
    image: &ImageFrame,
    output: &OnnxRunOutput,
    config: &YoloDecodeConfig,
) -> Vec<Detection> {
    if attrs < 6 || rows == 0 || data.len() < attrs * rows {
        return Vec::new();
    }
    let mut row = vec![0.0f32; attrs];
    let mut detections = Vec::new();
    for item in 0..rows {
        for attr in 0..attrs {
            row[attr] = data[attr * rows + item];
        }
        if let Some(detection) = decode_prediction(&row, image, output, config) {
            detections.push(detection);
        }
    }
    detections
}

fn decode_prediction(
    row: &[f32],
    image: &ImageFrame,
    output: &OnnxRunOutput,
    config: &YoloDecodeConfig,
) -> Option<Detection> {
    let (x, y, w, h, score, class_index, xyxy) = if row.len() == 6 {
        (
            row[0] as f64,
            row[1] as f64,
            row[2] as f64,
            row[3] as f64,
            row[4] as f64,
            row[5].max(0.0) as usize,
            true,
        )
    } else {
        let class_offset = if row.len() > 84 { 5 } else { 4 };
        let (class_index, class_score) = row[class_offset..]
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))?;
        let objectness = if class_offset == 5 {
            row[4] as f64
        } else {
            1.0
        };
        (
            row[0] as f64,
            row[1] as f64,
            row[2] as f64,
            row[3] as f64,
            objectness * *class_score as f64,
            class_index,
            false,
        )
    };
    if score < config.score_threshold {
        return None;
    }
    let image_width = image.width as f64;
    let image_height = image.height as f64;
    let (x1, y1, x2, y2) = if normalized([x, y, w, h]) {
        if xyxy {
            (
                x * image_width,
                y * image_height,
                w * image_width,
                h * image_height,
            )
        } else {
            cxcywh_to_xyxy(
                x * image_width,
                y * image_height,
                w * image_width,
                h * image_height,
            )
        }
    } else {
        let (x1, y1, x2, y2) = if xyxy {
            (x, y, w, h)
        } else {
            cxcywh_to_xyxy(x, y, w, h)
        };
        undo_letterbox(x1, y1, x2, y2, output)
    };

    let label = config
        .labels
        .get(class_index)
        .cloned()
        .unwrap_or_else(|| format!("class_{class_index}"));
    Some(Detection {
        label,
        score,
        bbox: Rect {
            x: x1.clamp(0.0, image_width),
            y: y1.clamp(0.0, image_height),
            width: (x2 - x1).max(0.0).min(image_width),
            height: (y2 - y1).max(0.0).min(image_height),
        },
    })
}

fn cxcywh_to_xyxy(cx: f64, cy: f64, w: f64, h: f64) -> (f64, f64, f64, f64) {
    (cx - w / 2.0, cy - h / 2.0, cx + w / 2.0, cy + h / 2.0)
}

fn normalized(values: [f64; 4]) -> bool {
    values.iter().all(|value| *value >= 0.0 && *value <= 1.01)
}

fn undo_letterbox(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    output: &OnnxRunOutput,
) -> (f64, f64, f64, f64) {
    let preprocess = output
        .metadata
        .get("preprocess")
        .unwrap_or(&serde_json::Value::Null);
    let scale = preprocess
        .get("scale")
        .and_then(|value| value.as_f64())
        .unwrap_or(1.0)
        .max(0.000001);
    let pad_x = preprocess
        .get("pad_x")
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    let pad_y = preprocess
        .get("pad_y")
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    (
        (x1 - pad_x) / scale,
        (y1 - pad_y) / scale,
        (x2 - pad_x) / scale,
        (y2 - pad_y) / scale,
    )
}

fn nms(mut detections: Vec<Detection>, threshold: f64) -> Vec<Detection> {
    let mut kept: Vec<Detection> = Vec::new();
    while let Some(candidate) = detections.first().cloned() {
        detections.remove(0);
        detections.retain(|other| {
            candidate.label != other.label || iou(&candidate.bbox, &other.bbox) < threshold
        });
        kept.push(candidate);
    }
    kept
}

fn iou(a: &Rect, b: &Rect) -> f64 {
    let ax2 = a.x + a.width;
    let ay2 = a.y + a.height;
    let bx2 = b.x + b.width;
    let by2 = b.y + b.height;
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = ax2.min(bx2);
    let bottom = ay2.min(by2);
    let intersection = (right - left).max(0.0) * (bottom - top).max(0.0);
    let union = a.width * a.height + b.width * b.height - intersection;
    if union <= 0.0 {
        0.0
    } else {
        intersection / union
    }
}
