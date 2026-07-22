use std::collections::BTreeMap;

use vision_core::{Detection, ImageFrame, NodeContext, Result, Value};

use crate::{
    backend::onnx::{InferenceBackend, OnnxRunOutput, OnnxRunRequest},
    decode::yolo::YoloDecodeConfig,
};

#[derive(Debug, Clone)]
pub(crate) struct DetectParams {
    pub model_path: String,
    pub backend_name: String,
    pub backend: InferenceBackend,
    pub input_width: i64,
    pub input_height: i64,
    pub mean: [f32; 3],
    pub std: [f32; 3],
    pub letterbox: bool,
    pub decode: YoloDecodeConfig,
}

impl DetectParams {
    pub(crate) fn from_context(ctx: &NodeContext<'_>) -> Self {
        let backend_name = crate::util::text(&ctx.node.params, "backend", "onnxruntime");
        Self {
            model_path: crate::util::text(&ctx.node.params, "model_path", ""),
            backend: InferenceBackend::from_param(&backend_name),
            backend_name,
            input_width: crate::util::integer(&ctx.node.params, "input_width", 640),
            input_height: crate::util::integer(&ctx.node.params, "input_height", 640),
            mean: crate::util::f32_triplet(&ctx.node.params, "mean", [0.0, 0.0, 0.0]),
            std: crate::util::f32_triplet(&ctx.node.params, "std", [1.0, 1.0, 1.0]),
            letterbox: crate::util::boolean(&ctx.node.params, "letterbox", true),
            decode: YoloDecodeConfig {
                score_threshold: crate::util::number(&ctx.node.params, "score_threshold", 0.5),
                nms_threshold: crate::util::number(&ctx.node.params, "nms_threshold", 0.45),
                labels: crate::util::string_array(&ctx.node.params, "labels"),
            },
        }
    }

    fn to_backend_request(&self, image: &ImageFrame) -> OnnxRunRequest {
        OnnxRunRequest {
            model_path: self.model_path.clone(),
            backend: self.backend.clone(),
            input_width: self.input_width,
            input_height: self.input_height,
            mean: self.mean,
            std: self.std,
            letterbox: self.letterbox,
            image: image.clone(),
        }
    }
}

pub(crate) fn run(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = crate::util::image(ctx, "image")?;
    let params = DetectParams::from_context(ctx);
    let backend_output = run_backend(&params, &image)?;
    let detections = postprocess(&backend_output, &image, &params);
    let raw = build_raw(ctx, &params, &backend_output, detections.len());
    Ok(BTreeMap::from([
        ("detections".to_string(), Value::Detections(detections)),
        ("raw".to_string(), Value::Json(raw)),
    ]))
}

fn run_backend(params: &DetectParams, image: &ImageFrame) -> Result<OnnxRunOutput> {
    let request = params.to_backend_request(image);
    crate::backend::onnx::run_onnx(&request)
}

fn postprocess(
    output: &OnnxRunOutput,
    image: &ImageFrame,
    params: &DetectParams,
) -> Vec<Detection> {
    crate::decode::yolo::decode(output, image, &params.decode)
}

fn build_raw(
    ctx: &NodeContext<'_>,
    params: &DetectParams,
    output: &OnnxRunOutput,
    detection_count: usize,
) -> serde_json::Value {
    serde_json::json!({
        "node": ctx.node.operator,
        "model_ecosystem": "ultralytics",
        "task": "detect",
        "backend": params.backend.as_str(),
        "backend_param": params.backend_name,
        "model_path": params.model_path,
        "input": {
            "width": params.input_width,
            "height": params.input_height,
            "letterbox": params.letterbox,
            "mean": params.mean,
            "std": params.std
        },
        "postprocess": {
            "score_threshold": params.decode.score_threshold,
            "nms_threshold": params.decode.nms_threshold,
            "labels": params.decode.labels
        },
        "mode": output.metadata.get("mode").and_then(|value| value.as_str()).unwrap_or("unknown"),
        "detection_count": detection_count,
        "tensors": output.tensors.iter().map(|tensor| {
            serde_json::json!({
                "name": tensor.name,
                "shape": tensor.shape,
                "len": tensor.data.len()
            })
        }).collect::<Vec<_>>(),
        "metadata": output.metadata
    })
}
