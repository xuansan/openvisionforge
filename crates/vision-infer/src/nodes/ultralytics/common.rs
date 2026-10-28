use vision_core::{OperatorFn, OperatorRegistry};

use crate::schema::{
    bool_param, enum_param, file_param, json_param, number_param, schema_with_params, string_param,
};

pub(crate) fn register_detect_node(
    registry: &mut OperatorRegistry,
    id: &'static str,
    name: &'static str,
) {
    registry.register(schema_with_params(
        id,
        name,
        "Infer/Ultralytics",
        &["image"],
        &[],
        &["detections", "raw"],
        DETECT_PARAMS,
        crate::nodes::ultralytics::detect_pipeline::run,
    ));
}

pub(crate) fn register_task_node(
    registry: &mut OperatorRegistry,
    id: &'static str,
    name: &'static str,
    outputs: &'static [&'static str],
    run: OperatorFn,
) {
    registry.register(schema_with_params(
        id,
        name,
        "Infer/Ultralytics",
        &["image"],
        &[],
        outputs,
        DETECT_PARAMS,
        run,
    ));
}

pub(crate) const DETECT_PARAMS: &[vision_core::ParamSchema] = &[
    file_param(
        "model_path",
        "Model Path",
        false,
        None,
        "Exported model path; takes precedence over model_id",
    )
    .group("Model")
    .order(10)
    .placeholder("models/yolo26.onnx")
    .accept(&[".onnx", ".engine", ".rknn", ".bmodel"]),
    string_param(
        "model_id",
        "Model ID",
        false,
        None,
        "Model key from models/registry.json",
    )
    .group("Model")
    .order(15)
    .placeholder("yolo11n-coco"),
    enum_param(
        "backend",
        "Backend",
        false,
        Some("onnxruntime"),
        "Execution backend",
        &[
            "onnxruntime",
            "cpu",
            "cuda",
            "tensorrt",
            "rknn",
            "sophon",
            "mock",
        ],
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
    number_param(
        "score_threshold",
        "Score Threshold",
        false,
        Some("0.5"),
        "Minimum confidence score",
    )
    .group("Postprocess")
    .order(50)
    .range(0.0, 1.0, 0.01),
    number_param(
        "nms_threshold",
        "NMS Threshold",
        false,
        Some("0.45"),
        "Non-maximum suppression threshold",
    )
    .group("Postprocess")
    .order(60)
    .range(0.0, 1.0, 0.01),
    enum_param(
        "decode_format",
        "Decode Format",
        false,
        Some("auto"),
        "YOLO output layout; auto uses label count to distinguish objectness layouts",
        &["auto", "ultralytics", "ultralytics_objectness"],
    )
    .group("Postprocess")
    .order(65)
    .advanced(true),
    json_param("labels", "Labels", false, Some("[]"), "Class label list")
        .group("Postprocess")
        .order(70)
        .placeholder("[\"ok\",\"defect\"]"),
    number_param(
        "top_k",
        "Top K",
        false,
        Some("5"),
        "Maximum classification results; ignored by non-classification tasks",
    )
    .group("Postprocess")
    .order(75)
    .range(1.0, 1000.0, 1.0)
    .advanced(true),
    number_param(
        "mask_threshold",
        "Mask Threshold",
        false,
        Some("0.5"),
        "Prototype mask probability threshold; ignored by non-segmentation tasks",
    )
    .group("Postprocess")
    .order(77)
    .range(0.0, 1.0, 0.01)
    .advanced(true),
    json_param(
        "mean",
        "Mean",
        false,
        Some("[0,0,0]"),
        "Input normalization mean",
    )
    .group("Preprocess")
    .order(80)
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
    .order(90)
    .advanced(true)
    .placeholder("[1,1,1]"),
    bool_param(
        "letterbox",
        "Letterbox",
        false,
        Some("true"),
        "Use Ultralytics letterbox resize",
    )
    .group("Preprocess")
    .order(100),
];
