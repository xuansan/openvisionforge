use std::collections::BTreeMap;

use vision_core::{
    Classification, Detection, NodeContext, OrientedBox, PoseKeypoint, PoseResult, Rect, Result,
    SegmentationMask, Value,
};

use crate::{
    backend::onnx::{InferenceBackend, OnnxRunOutput, OnnxRunRequest},
    decode::yolo::YoloDecodeConfig,
};

#[derive(Debug, Clone, Copy)]
enum Task {
    Classify,
    Segment,
    Pose,
    Obb,
}

pub(crate) fn classify(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    run_task(ctx, Task::Classify)
}

pub(crate) fn segment(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    run_task(ctx, Task::Segment)
}

pub(crate) fn pose(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    run_task(ctx, Task::Pose)
}

pub(crate) fn obb(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    run_task(ctx, Task::Obb)
}

fn run_task(ctx: &NodeContext<'_>, task: Task) -> Result<BTreeMap<String, Value>> {
    let image = crate::util::image(ctx, "image")?;
    let backend_name = crate::util::text(&ctx.node.params, "backend", "onnxruntime");
    let model_path = crate::util::text(&ctx.node.params, "model_path", "");
    let labels = labels(ctx);
    let score_threshold = crate::util::number(&ctx.node.params, "score_threshold", 0.5);
    let request = OnnxRunRequest {
        model_path: model_path.clone(),
        backend: InferenceBackend::from_param(&backend_name),
        input_width: crate::util::integer(&ctx.node.params, "input_width", 640),
        input_height: crate::util::integer(&ctx.node.params, "input_height", 640),
        mean: crate::util::f32_triplet(&ctx.node.params, "mean", [0.0, 0.0, 0.0]),
        std: crate::util::f32_triplet(&ctx.node.params, "std", [1.0, 1.0, 1.0]),
        letterbox: crate::util::boolean(&ctx.node.params, "letterbox", true),
        image: image.clone(),
    };
    let backend_output = crate::backend::onnx::run_onnx(&request)?;
    let decode = YoloDecodeConfig {
        score_threshold,
        nms_threshold: crate::util::number(&ctx.node.params, "nms_threshold", 0.45),
        labels: labels.clone(),
    };
    let detections = crate::decode::yolo::decode(&backend_output, &image, &decode);
    let raw = raw(
        ctx,
        task,
        &request,
        &backend_name,
        &model_path,
        &labels,
        &backend_output,
    );

    Ok(match task {
        Task::Classify => BTreeMap::from([
            (
                "classes".to_string(),
                Value::Classifications(classify_output(&labels)),
            ),
            ("raw".to_string(), Value::Json(raw)),
        ]),
        Task::Segment => BTreeMap::from([
            (
                "detections".to_string(),
                Value::Detections(detections.clone()),
            ),
            (
                "masks".to_string(),
                Value::Segmentations(mask_output(&detections)),
            ),
            ("raw".to_string(), Value::Json(raw)),
        ]),
        Task::Pose => BTreeMap::from([
            ("poses".to_string(), Value::Poses(pose_output(&detections))),
            ("raw".to_string(), Value::Json(raw)),
        ]),
        Task::Obb => BTreeMap::from([
            (
                "oriented_boxes".to_string(),
                Value::OrientedBoxes(obb_output(&detections)),
            ),
            ("raw".to_string(), Value::Json(raw)),
        ]),
    })
}

fn labels(ctx: &NodeContext<'_>) -> Vec<String> {
    let labels = crate::util::string_array(&ctx.node.params, "labels");
    if labels.is_empty() {
        vec!["part".to_string(), "defect".to_string()]
    } else {
        labels
    }
}

fn classify_output(labels: &[String]) -> Vec<Classification> {
    labels
        .iter()
        .take(5)
        .enumerate()
        .map(|(index, label)| Classification {
            label: label.clone(),
            score: (0.88_f64 - index as f64 * 0.08).max(0.1),
            rank: index as u32 + 1,
        })
        .collect()
}

fn mask_output(detections: &[Detection]) -> Vec<SegmentationMask> {
    detections
        .iter()
        .enumerate()
        .map(|(index, detection)| SegmentationMask {
            id: format!("mask_{index}"),
            label: detection.label.clone(),
            score: detection.score,
            bbox: detection.bbox.clone(),
            area_px: detection.bbox.width * detection.bbox.height,
            format: "bbox_mask_mvp".to_string(),
        })
        .collect()
}

fn pose_output(detections: &[Detection]) -> Vec<PoseResult> {
    detections
        .iter()
        .enumerate()
        .map(|(index, detection)| {
            let bbox = &detection.bbox;
            PoseResult {
                id: format!("pose_{index}"),
                label: detection.label.clone(),
                score: detection.score,
                bbox: bbox.clone(),
                keypoints: vec![
                    point(
                        "center",
                        bbox.x + bbox.width * 0.5,
                        bbox.y + bbox.height * 0.5,
                        detection.score,
                    ),
                    point(
                        "left",
                        bbox.x,
                        bbox.y + bbox.height * 0.5,
                        detection.score * 0.95,
                    ),
                    point(
                        "right",
                        bbox.x + bbox.width,
                        bbox.y + bbox.height * 0.5,
                        detection.score * 0.95,
                    ),
                ],
            }
        })
        .collect()
}

fn obb_output(detections: &[Detection]) -> Vec<OrientedBox> {
    detections
        .iter()
        .enumerate()
        .map(|(index, detection)| oriented_rect(index, detection))
        .collect()
}

fn point(name: &str, x: f64, y: f64, score: f64) -> PoseKeypoint {
    PoseKeypoint {
        name: name.to_string(),
        x,
        y,
        score,
    }
}

fn oriented_rect(index: usize, detection: &Detection) -> OrientedBox {
    let bbox: &Rect = &detection.bbox;
    OrientedBox {
        id: format!("obb_{index}"),
        label: detection.label.clone(),
        score: detection.score,
        cx: bbox.x + bbox.width * 0.5,
        cy: bbox.y + bbox.height * 0.5,
        width: bbox.width,
        height: bbox.height,
        angle_deg: 0.0,
    }
}

fn raw(
    ctx: &NodeContext<'_>,
    task: Task,
    request: &OnnxRunRequest,
    backend_name: &str,
    model_path: &str,
    labels: &[String],
    output: &OnnxRunOutput,
) -> serde_json::Value {
    serde_json::json!({
        "node": ctx.node.operator,
        "model_ecosystem": "ultralytics",
        "task": task_name(task),
        "backend": request.backend.as_str(),
        "backend_param": backend_name,
        "model_path": model_path,
        "labels": labels,
        "input": {
            "width": request.input_width,
            "height": request.input_height,
            "letterbox": request.letterbox,
            "mean": request.mean,
            "std": request.std
        },
        "mode": output.metadata.get("mode").and_then(|value| value.as_str()).unwrap_or("unknown"),
        "tensors": output.tensors.iter().map(|tensor| {
            serde_json::json!({
                "name": tensor.name,
                "shape": tensor.shape,
                "len": tensor.data.len()
            })
        }).collect::<Vec<_>>(),
        "metadata": output.metadata,
        "mvp": true
    })
}

fn task_name(task: Task) -> &'static str {
    match task {
        Task::Classify => "classify",
        Task::Segment => "segment",
        Task::Pose => "pose",
        Task::Obb => "obb",
    }
}
