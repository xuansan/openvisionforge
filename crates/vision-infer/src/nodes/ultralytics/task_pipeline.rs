use std::collections::BTreeMap;

use vision_core::{NodeContext, Result, Value};

use crate::backend::onnx::{InferenceBackend, OnnxRunOutput, OnnxRunRequest};

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
    let model_id = crate::util::text(&ctx.node.params, "model_id", "");
    let model = crate::model_assets::resolve(
        &model_id,
        &crate::util::text(&ctx.node.params, "model_path", ""),
        task_name(task),
    )?;
    let model_path = model.path;
    let mut labels = labels(ctx);
    if labels.is_empty() {
        labels = model.labels;
    }
    let score_threshold = crate::util::number(&ctx.node.params, "score_threshold", 0.5);
    let request = OnnxRunRequest {
        model_path: model_path.clone(),
        backend: InferenceBackend::from_param(&backend_name),
        input_width: ctx
            .node
            .params
            .get("input_width")
            .and_then(|value| value.as_i64())
            .or(model.input_width)
            .unwrap_or(640),
        input_height: ctx
            .node
            .params
            .get("input_height")
            .and_then(|value| value.as_i64())
            .or(model.input_height)
            .unwrap_or(640),
        mean: crate::util::f32_triplet(&ctx.node.params, "mean", [0.0, 0.0, 0.0]),
        std: crate::util::f32_triplet(&ctx.node.params, "std", [1.0, 1.0, 1.0]),
        letterbox: crate::util::boolean(&ctx.node.params, "letterbox", true),
        image: image.clone(),
    };
    let backend_output = crate::backend::onnx::run_onnx(&request)?;
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
                Value::Classifications(crate::decode::classification::decode(
                    &backend_output,
                    &labels,
                    score_threshold,
                    crate::util::integer(&ctx.node.params, "top_k", 5).max(1) as usize,
                )?),
            ),
            ("raw".to_string(), Value::Json(raw)),
        ]),
        Task::Segment => {
            let (detections, masks) = crate::decode::segmentation::decode(
                &backend_output,
                &image,
                &labels,
                score_threshold,
                crate::util::number(&ctx.node.params, "nms_threshold", 0.45),
                crate::util::number(&ctx.node.params, "mask_threshold", 0.5),
                Some(&ctx.execution.records_dir.join("artifacts")),
            )?;
            BTreeMap::from([
                ("detections".into(), Value::Detections(detections)),
                ("masks".into(), Value::Segmentations(masks)),
                ("raw".into(), Value::Json(raw)),
            ])
        }
        Task::Pose => BTreeMap::from([
            (
                "poses".into(),
                Value::Poses(crate::decode::pose::decode(
                    &backend_output,
                    &image,
                    score_threshold,
                    crate::util::number(&ctx.node.params, "nms_threshold", 0.45),
                )?),
            ),
            ("raw".into(), Value::Json(raw)),
        ]),
        Task::Obb => BTreeMap::from([
            (
                "oriented_boxes".into(),
                Value::OrientedBoxes(crate::decode::obb::decode(
                    &backend_output,
                    &image,
                    &labels,
                    score_threshold,
                    crate::util::number(&ctx.node.params, "nms_threshold", 0.45),
                )?),
            ),
            ("raw".into(), Value::Json(raw)),
        ]),
    })
}

fn labels(ctx: &NodeContext<'_>) -> Vec<String> {
    crate::util::string_array(&ctx.node.params, "labels")
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
        "decoder": "real"
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
