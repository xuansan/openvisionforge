use vision_core::{ImageFrame, PoseKeypoint, PoseResult, Result};

use crate::{backend::onnx::OnnxRunOutput, decode::task_common as common};

const COCO_KEYPOINTS: [&str; 17] = [
    "nose",
    "left_eye",
    "right_eye",
    "left_ear",
    "right_ear",
    "left_shoulder",
    "right_shoulder",
    "left_elbow",
    "right_elbow",
    "left_wrist",
    "right_wrist",
    "left_hip",
    "right_hip",
    "left_knee",
    "right_knee",
    "left_ankle",
    "right_ankle",
];

pub(crate) fn decode(
    output: &OnnxRunOutput,
    image: &ImageFrame,
    threshold: f64,
    nms: f64,
) -> Result<Vec<PoseResult>> {
    let tensor = output
        .tensors
        .first()
        .ok_or_else(|| common::error("pose model produced no tensors"))?;
    let rows = common::transposed_rows(tensor)?;
    if tensor.shape.get(1) != Some(&56) {
        return Err(common::error(format!(
            "unsupported pose shape {:?}; expected 56 attributes",
            tensor.shape
        )));
    }
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
    let mut candidates = Vec::new();
    for row in rows {
        let score = row[4] as f64;
        if score < threshold {
            continue;
        }
        let bbox = common::restore_box(&row, image, output);
        let keypoints = COCO_KEYPOINTS
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let at = 5 + i * 3;
                PoseKeypoint {
                    name: (*name).into(),
                    x: ((row[at] as f64 - px) / scale).clamp(0.0, image.width as f64),
                    y: ((row[at + 1] as f64 - py) / scale).clamp(0.0, image.height as f64),
                    score: row[at + 2] as f64,
                }
            })
            .collect();
        candidates.push(PoseResult {
            id: format!("pose-{}", candidates.len()),
            label: "person".into(),
            score,
            bbox,
            keypoints,
        });
    }
    candidates.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut kept: Vec<PoseResult> = Vec::new();
    for candidate in candidates {
        if kept
            .iter()
            .all(|other| common::iou(&candidate.bbox, &other.bbox) < nms)
        {
            kept.push(candidate);
        }
    }
    for (index, pose) in kept.iter_mut().enumerate() {
        pose.id = format!("pose-{index}");
    }
    Ok(kept)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::onnx::TensorOutput;
    #[test]
    fn decodes_pose_tensor() {
        let mut data = vec![0.0; 56];
        data[0] = 100.0;
        data[1] = 120.0;
        data[2] = 40.0;
        data[3] = 80.0;
        data[4] = 0.9;
        for i in 0..17 {
            data[5 + i * 3] = 100.0;
            data[6 + i * 3] = 120.0;
            data[7 + i * 3] = 0.8;
        }
        let out = OnnxRunOutput {
            tensors: vec![TensorOutput {
                name: "o".into(),
                shape: vec![1, 56, 1],
                data,
            }],
            metadata: serde_json::json!({"preprocess":{"scale":1.0,"pad_x":0.0,"pad_y":0.0}}),
        };
        let image = ImageFrame {
            width: 640,
            height: 480,
            channels: 3,
            pixel_format: "RGB8".into(),
            source_uri: None,
        };
        assert_eq!(
            decode(&out, &image, 0.25, 0.45).unwrap()[0].keypoints.len(),
            17
        );
    }
}
