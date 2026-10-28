use vision_core::{Detection, ImageFrame, Rect, Result, VisionError};

use crate::backend::onnx::OnnxRunOutput;

#[derive(Debug, Clone)]
pub(crate) struct YoloDecodeConfig {
    pub score_threshold: f64,
    pub nms_threshold: f64,
    pub labels: Vec<String>,
    pub format: YoloDecodeFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum YoloDecodeFormat {
    Auto,
    Ultralytics,
    UltralyticsObjectness,
}

impl YoloDecodeFormat {
    pub(crate) fn from_param(value: &str) -> Self {
        match value {
            "ultralytics" => Self::Ultralytics,
            "ultralytics_objectness" => Self::UltralyticsObjectness,
            _ => Self::Auto,
        }
    }
}

pub(crate) fn decode(
    output: &OnnxRunOutput,
    image: &ImageFrame,
    config: &YoloDecodeConfig,
) -> Result<Vec<Detection>> {
    let Some(tensor) = output.tensors.first() else {
        return Err(decode_error("model produced no output tensors"));
    };
    let mut detections = decode_tensor(tensor, image, output, config)?;
    detections.sort_by(|a, b| b.score.total_cmp(&a.score));
    Ok(nms(detections, config.nms_threshold))
}

fn decode_tensor(
    tensor: &crate::backend::onnx::TensorOutput,
    image: &ImageFrame,
    output: &OnnxRunOutput,
    config: &YoloDecodeConfig,
) -> Result<Vec<Detection>> {
    if tensor.shape.len() == 3 {
        let a = tensor.shape[1].max(0) as usize;
        let b = tensor.shape[2].max(0) as usize;
        if a >= 5 && b > a {
            return Ok(decode_rows_transposed(
                &tensor.data,
                a,
                b,
                image,
                output,
                config,
            ));
        }
        return Ok(decode_rows(&tensor.data, a, b, image, output, config));
    }
    if tensor.shape.len() == 2 {
        return Ok(decode_rows(
            &tensor.data,
            tensor.shape[0].max(0) as usize,
            tensor.shape[1].max(0) as usize,
            image,
            output,
            config,
        ));
    }
    Err(decode_error(format!(
        "unsupported YOLO detection output shape {:?}; expected [1, attributes, predictions], [1, predictions, attributes], or [predictions, attributes]",
        tensor.shape
    )))
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
            decode_prediction(
                &data[row * cols..row * cols + cols],
                image,
                output,
                config,
                cols == 6,
            )
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
        if let Some(detection) = decode_prediction(&row, image, output, config, false) {
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
    postprocessed_xyxy: bool,
) -> Option<Detection> {
    let (x, y, w, h, score, class_index, xyxy) = if postprocessed_xyxy {
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
        let class_offset = match config.format {
            YoloDecodeFormat::Ultralytics => 4,
            YoloDecodeFormat::UltralyticsObjectness => 5,
            YoloDecodeFormat::Auto
                if !config.labels.is_empty() && row.len() == config.labels.len() + 5 =>
            {
                5
            }
            YoloDecodeFormat::Auto => 4,
        };
        if row.len() <= class_offset {
            return None;
        }
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
    let x1 = x1.clamp(0.0, image_width);
    let y1 = y1.clamp(0.0, image_height);
    let x2 = x2.clamp(x1, image_width);
    let y2 = y2.clamp(y1, image_height);
    Some(Detection {
        label,
        score,
        bbox: Rect {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        },
    })
}

fn decode_error(message: impl Into<String>) -> VisionError {
    VisionError::Operator {
        node: "infer.yolo.decode".to_string(),
        message: message.into(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::onnx::{OnnxRunOutput, TensorOutput};

    fn image() -> ImageFrame {
        ImageFrame {
            width: 640,
            height: 480,
            channels: 3,
            pixel_format: "RGB8".to_string(),
            source_uri: None,
        }
    }

    fn output(shape: Vec<i64>, data: Vec<f32>) -> OnnxRunOutput {
        OnnxRunOutput {
            tensors: vec![TensorOutput {
                name: "output0".to_string(),
                shape,
                data,
            }],
            metadata: serde_json::json!({
                "preprocess": {"scale": 1.0, "pad_x": 0.0, "pad_y": 80.0}
            }),
        }
    }

    fn config() -> YoloDecodeConfig {
        YoloDecodeConfig {
            score_threshold: 0.25,
            nms_threshold: 0.45,
            labels: vec!["part".to_string(), "defect".to_string()],
            format: YoloDecodeFormat::Auto,
        }
    }

    #[test]
    fn decodes_transposed_ultralytics_output_and_restores_letterbox() {
        let mut data = vec![0.0; 6 * 7];
        data[0] = 320.0;
        data[7] = 320.0;
        data[14] = 200.0;
        data[21] = 100.0;
        data[28] = 0.9;
        data[35] = 0.1;
        let tensor = output(vec![1, 6, 7], data);
        let detections = decode(&tensor, &image(), &config()).unwrap();
        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].label, "part");
        assert!((detections[0].bbox.y - 190.0).abs() < 0.001);
    }

    #[test]
    fn suppresses_overlapping_boxes_of_the_same_class() {
        let tensor = output(
            vec![2, 6],
            vec![
                10.0, 10.0, 110.0, 110.0, 0.9, 0.0, 12.0, 12.0, 108.0, 108.0, 0.8, 0.0,
            ],
        );
        assert_eq!(decode(&tensor, &image(), &config()).unwrap().len(), 1);
    }

    #[test]
    fn rejects_unknown_output_rank() {
        let error = decode(&output(vec![1, 2, 3, 4], vec![]), &image(), &config()).unwrap_err();
        assert!(error
            .to_string()
            .contains("unsupported YOLO detection output shape"));
    }

    #[test]
    fn auto_detects_objectness_layout_from_label_count() {
        let tensor = output(vec![1, 7], vec![320.0, 240.0, 100.0, 80.0, 0.5, 0.8, 0.2]);
        let detections = decode(&tensor, &image(), &config()).unwrap();
        assert_eq!(detections.len(), 1);
        assert!((detections[0].score - 0.4).abs() < 0.0001);
    }
}
