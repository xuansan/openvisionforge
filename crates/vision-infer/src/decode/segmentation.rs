use crate::{backend::onnx::OnnxRunOutput, decode::task_common as common};
use std::{fs, path::Path};
use vision_core::{Detection, ImageFrame, Result, SegmentationMask};

struct Candidate {
    detection: Detection,
    coefficients: Vec<f32>,
}

pub(crate) fn decode(
    output: &OnnxRunOutput,
    image: &ImageFrame,
    labels: &[String],
    threshold: f64,
    nms: f64,
    mask_threshold: f64,
    mask_dir: Option<&Path>,
) -> Result<(Vec<Detection>, Vec<SegmentationMask>)> {
    if output.tensors.len() < 2 {
        return Err(common::error(
            "segmentation model must output predictions and prototypes",
        ));
    }
    let predictions = &output.tensors[0];
    let prototypes = &output.tensors[1];
    let rows = common::transposed_rows(predictions)?;
    let [_, channels, mh, mw] = prototypes.shape.as_slice() else {
        return Err(common::error(format!(
            "unsupported mask prototype shape {:?}",
            prototypes.shape
        )));
    };
    let (channels, mh, mw) = (*channels as usize, *mh as usize, *mw as usize);
    let attrs = *predictions.shape.get(1).unwrap_or(&0) as usize;
    if attrs <= 4 + channels {
        return Err(common::error(format!(
            "unsupported segmentation prediction shape {:?}",
            predictions.shape
        )));
    }
    let classes = attrs - 4 - channels;
    let mut candidates = Vec::new();
    for row in rows {
        let Some((class, score)) = common::best_class(&row[4..4 + classes]) else {
            continue;
        };
        if score < threshold {
            continue;
        }
        let bbox = common::restore_box(&row, image, output);
        candidates.push(Candidate {
            detection: Detection {
                label: common::label(labels, class),
                score,
                bbox,
            },
            coefficients: row[4 + classes..].to_vec(),
        });
    }
    candidates.sort_by(|a, b| b.detection.score.total_cmp(&a.detection.score));
    let mut kept: Vec<Candidate> = Vec::new();
    for candidate in candidates {
        if kept.iter().all(|other| {
            candidate.detection.label != other.detection.label
                || common::iou(&candidate.detection.bbox, &other.detection.bbox) < nms
        }) {
            kept.push(candidate)
        }
    }
    let mut masks = Vec::new();
    for (id, candidate) in kept.iter().enumerate() {
        let bbox = &candidate.detection.bbox;
        let x1 = bbox.x.floor().clamp(0.0, image.width as f64) as usize;
        let y1 = bbox.y.floor().clamp(0.0, image.height as f64) as usize;
        let x2 = (bbox.x + bbox.width).ceil().clamp(0.0, image.width as f64) as usize;
        let y2 = (bbox.y + bbox.height)
            .ceil()
            .clamp(0.0, image.height as f64) as usize;
        let mut active = 0usize;
        let mut mask = vec![0u8; image.width as usize * image.height as usize];
        for y in y1..y2 {
            for x in x1..x2 {
                let proto_x = (x * mw / image.width as usize).min(mw - 1);
                let proto_y = (y * mh / image.height as usize).min(mh - 1);
                let pixel = proto_y * mw + proto_x;
                let logit = (0..channels)
                    .map(|ch| candidate.coefficients[ch] * prototypes.data[ch * mh * mw + pixel])
                    .sum::<f32>();
                if 1.0 / (1.0 + (-logit).exp()) >= mask_threshold as f32 {
                    active += 1;
                    mask[y * image.width as usize + x] = 255;
                }
            }
        }
        let area = active as f64;
        let source_uri = mask_dir.and_then(|dir| {
            let path = dir.join(format!("mask-{id}.png"));
            fs::create_dir_all(dir).ok()?;
            image::GrayImage::from_raw(image.width, image.height, mask)?
                .save(&path)
                .ok()?;
            Some(path.to_string_lossy().into_owned())
        });
        masks.push(SegmentationMask {
            id: format!("mask-{id}"),
            label: candidate.detection.label.clone(),
            score: candidate.detection.score,
            bbox: bbox.clone(),
            area_px: area,
            format: "prototype_logits".into(),
            source_uri,
        });
    }
    Ok((kept.into_iter().map(|c| c.detection).collect(), masks))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::onnx::TensorOutput;
    #[test]
    fn decodes_segmentation_tensors() {
        let out = OnnxRunOutput {
            tensors: vec![
                TensorOutput {
                    name: "pred".into(),
                    shape: vec![1, 7, 1],
                    data: vec![50.0, 50.0, 40.0, 40.0, 0.9, 10.0, 10.0],
                },
                TensorOutput {
                    name: "proto".into(),
                    shape: vec![1, 2, 2, 2],
                    data: vec![1.0; 8],
                },
            ],
            metadata: serde_json::json!({"preprocess":{"scale":1.0}}),
        };
        let image = ImageFrame {
            width: 100,
            height: 100,
            channels: 3,
            pixel_format: "RGB8".into(),
            source_uri: None,
        };
        let (d, m) = decode(&out, &image, &["part".into()], 0.25, 0.45, 0.5, None).unwrap();
        assert_eq!(d.len(), 1);
        assert_eq!(m.len(), 1);
        assert!(m[0].area_px > 0.0);
    }
}
