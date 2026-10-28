use vision_core::{ImageFrame, Rect, VisionError};

use crate::backend::onnx::{OnnxRunOutput, TensorOutput};

pub(crate) fn transposed_rows(tensor: &TensorOutput) -> Result<Vec<Vec<f32>>, VisionError> {
    let [_, attrs, count] = tensor.shape.as_slice() else {
        return Err(error(format!(
            "expected [1, attributes, predictions], got {:?}",
            tensor.shape
        )));
    };
    let (attrs, count) = (*attrs as usize, *count as usize);
    if attrs == 0 || count == 0 || tensor.data.len() < attrs * count {
        return Err(error("output tensor is empty or truncated"));
    }
    Ok((0..count)
        .map(|item| {
            (0..attrs)
                .map(|attr| tensor.data[attr * count + item])
                .collect()
        })
        .collect())
}

pub(crate) fn restore_box(row: &[f32], image: &ImageFrame, output: &OnnxRunOutput) -> Rect {
    let preprocess = output
        .metadata
        .get("preprocess")
        .unwrap_or(&serde_json::Value::Null);
    let scale = preprocess
        .get("scale")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0)
        .max(1e-9);
    let pad_x = preprocess
        .get("pad_x")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let pad_y = preprocess
        .get("pad_y")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let cx = (row[0] as f64 - pad_x) / scale;
    let cy = (row[1] as f64 - pad_y) / scale;
    let width = row[2] as f64 / scale;
    let height = row[3] as f64 / scale;
    let x = (cx - width / 2.0).clamp(0.0, image.width as f64);
    let y = (cy - height / 2.0).clamp(0.0, image.height as f64);
    Rect {
        x,
        y,
        width: width.min(image.width as f64 - x).max(0.0),
        height: height.min(image.height as f64 - y).max(0.0),
    }
}

pub(crate) fn best_class(scores: &[f32]) -> Option<(usize, f64)> {
    scores
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, s)| (i, *s as f64))
}

pub(crate) fn label(labels: &[String], index: usize) -> String {
    labels
        .get(index)
        .cloned()
        .unwrap_or_else(|| format!("class_{index}"))
}

pub(crate) fn iou(a: &Rect, b: &Rect) -> f64 {
    let intersection = ((a.x + a.width).min(b.x + b.width) - a.x.max(b.x)).max(0.0)
        * ((a.y + a.height).min(b.y + b.height) - a.y.max(b.y)).max(0.0);
    let union = a.width * a.height + b.width * b.height - intersection;
    if union <= 0.0 {
        0.0
    } else {
        intersection / union
    }
}

pub(crate) fn error(message: impl Into<String>) -> VisionError {
    VisionError::Operator {
        node: "infer.ultralytics.decode".into(),
        message: message.into(),
    }
}
