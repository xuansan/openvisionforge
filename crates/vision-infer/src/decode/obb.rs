use crate::{backend::onnx::OnnxRunOutput, decode::task_common as common};
use vision_core::{ImageFrame, OrientedBox, Result};

pub(crate) fn decode(
    output: &OnnxRunOutput,
    image: &ImageFrame,
    labels: &[String],
    threshold: f64,
    nms: f64,
) -> Result<Vec<OrientedBox>> {
    let tensor = output
        .tensors
        .first()
        .ok_or_else(|| common::error("OBB model produced no tensors"))?;
    let rows = common::transposed_rows(tensor)?;
    let attrs = *tensor.shape.get(1).unwrap_or(&0) as usize;
    if attrs < 6 {
        return Err(common::error(format!(
            "unsupported OBB shape {:?}",
            tensor.shape
        )));
    }
    let classes = attrs - 5;
    let preprocess = output
        .metadata
        .get("preprocess")
        .unwrap_or(&serde_json::Value::Null);
    let scale = preprocess
        .get("scale")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0)
        .max(1e-9);
    let px = preprocess
        .get("pad_x")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let py = preprocess
        .get("pad_y")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let mut boxes = Vec::new();
    for row in rows {
        let Some((class, score)) = common::best_class(&row[4..4 + classes]) else {
            continue;
        };
        if score < threshold {
            continue;
        }
        let cx = ((row[0] as f64 - px) / scale).clamp(0.0, image.width as f64);
        let cy = ((row[1] as f64 - py) / scale).clamp(0.0, image.height as f64);
        boxes.push(OrientedBox {
            id: format!("obb-{}", boxes.len()),
            label: common::label(labels, class),
            score,
            cx,
            cy,
            width: row[2] as f64 / scale,
            height: row[3] as f64 / scale,
            angle_deg: (row[attrs - 1] as f64).to_degrees(),
        });
    }
    boxes.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut kept: Vec<OrientedBox> = Vec::new();
    for candidate in boxes {
        let rect = vision_core::Rect {
            x: candidate.cx - candidate.width / 2.0,
            y: candidate.cy - candidate.height / 2.0,
            width: candidate.width,
            height: candidate.height,
        };
        if kept.iter().all(|other| {
            let r = vision_core::Rect {
                x: other.cx - other.width / 2.0,
                y: other.cy - other.height / 2.0,
                width: other.width,
                height: other.height,
            };
            candidate.label != other.label || common::iou(&rect, &r) < nms
        }) {
            kept.push(candidate);
        }
    }
    for (index, oriented_box) in kept.iter_mut().enumerate() {
        oriented_box.id = format!("obb-{index}");
    }
    Ok(kept)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::onnx::TensorOutput;
    #[test]
    fn decodes_obb_tensor() {
        let out = OnnxRunOutput {
            tensors: vec![TensorOutput {
                name: "o".into(),
                shape: vec![1, 7, 1],
                data: vec![100.0, 100.0, 40.0, 20.0, 0.1, 0.9, 1.5707964],
            }],
            metadata: serde_json::json!({"preprocess":{"scale":1.0}}),
        };
        let image = ImageFrame {
            width: 640,
            height: 480,
            channels: 3,
            pixel_format: "RGB8".into(),
            source_uri: None,
        };
        let b = decode(&out, &image, &["a".into(), "b".into()], 0.25, 0.45).unwrap();
        assert_eq!(b[0].label, "b");
        assert!((b[0].angle_deg - 90.0).abs() < 0.01);
    }
}
