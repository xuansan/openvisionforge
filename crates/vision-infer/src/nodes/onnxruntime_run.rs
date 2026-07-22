use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

use crate::schema::{
    bool_param, enum_param, file_param, json_param, number_param, schema_with_params,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema_with_params(
        "infer.onnxruntime.run",
        "ONNX Runtime Run",
        "Infer/ONNXRuntime",
        &["image"],
        &[],
        &["raw", "metadata"],
        PARAMS,
        run,
    ));
}

fn run(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = crate::util::image(ctx, "image")?;
    let backend = crate::util::text(&ctx.node.params, "backend", "cpu");
    let model_path = crate::util::text(&ctx.node.params, "model_path", "");
    let input_width = crate::util::integer(&ctx.node.params, "input_width", 640);
    let input_height = crate::util::integer(&ctx.node.params, "input_height", 640);
    let mean = crate::util::f32_triplet(&ctx.node.params, "mean", [0.0, 0.0, 0.0]);
    let std = crate::util::f32_triplet(&ctx.node.params, "std", [1.0, 1.0, 1.0]);
    let letterbox = crate::util::boolean(&ctx.node.params, "letterbox", true);
    let request = crate::backend::onnx::OnnxRunRequest {
        model_path,
        backend: crate::backend::onnx::InferenceBackend::from_param(&backend),
        input_width,
        input_height,
        mean,
        std,
        letterbox,
        image,
    };
    let output = crate::backend::onnx::run_onnx(&request)?;
    let raw = serde_json::json!({
        "tensors": output.tensors.iter().map(|tensor| {
            serde_json::json!({
                "name": tensor.name,
                "shape": tensor.shape,
                "len": tensor.data.len()
            })
        }).collect::<Vec<_>>(),
        "metadata": output.metadata,
    });
    Ok(BTreeMap::from([
        ("raw".to_string(), Value::Json(raw)),
        ("metadata".to_string(), Value::Json(output.metadata)),
    ]))
}

const PARAMS: &[vision_core::ParamSchema] = &[
    file_param(
        "model_path",
        "Model Path",
        true,
        None,
        "ONNX model file path",
    )
    .group("Model")
    .order(10)
    .accept(&[".onnx"])
    .placeholder("models/model.onnx"),
    enum_param(
        "backend",
        "Backend",
        false,
        Some("cpu"),
        "Execution backend",
        &["cpu", "cuda", "tensorrt"],
    )
    .group("Model")
    .order(20),
    number_param(
        "input_width",
        "Input Width",
        false,
        Some("640"),
        "Model input width",
    )
    .group("Preprocess")
    .order(30)
    .unit("px")
    .range(32.0, 4096.0, 32.0),
    number_param(
        "input_height",
        "Input Height",
        false,
        Some("640"),
        "Model input height",
    )
    .group("Preprocess")
    .order(40)
    .unit("px")
    .range(32.0, 4096.0, 32.0),
    json_param(
        "mean",
        "Mean",
        false,
        Some("[0,0,0]"),
        "Input normalization mean",
    )
    .group("Preprocess")
    .order(50)
    .advanced(true)
    .placeholder("[0,0,0]"),
    json_param(
        "std",
        "Std",
        false,
        Some("[1,1,1]"),
        "Input normalization std",
    )
    .group("Preprocess")
    .order(60)
    .advanced(true)
    .placeholder("[1,1,1]"),
    bool_param(
        "letterbox",
        "Letterbox",
        false,
        Some("true"),
        "Use YOLO style letterbox resize",
    )
    .group("Preprocess")
    .order(70),
];
