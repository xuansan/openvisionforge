use vision_core::OperatorRegistry;

use crate::{
    nodes::ultralytics::{common, detect_pipeline},
    schema::schema_with_params,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema_with_params(
        "infer.yolo.detect",
        "YOLO Detect",
        "Infer/YOLO",
        &["image"],
        &[],
        &["detections", "raw"],
        common::DETECT_PARAMS,
        detect_pipeline::run,
    ));
    registry.register(schema_with_params(
        "infer.yolo26.detect",
        "YOLO26 Detect",
        "Infer/YOLO",
        &["image"],
        &[],
        &["detections", "raw"],
        common::DETECT_PARAMS,
        detect_pipeline::run,
    ));
}
