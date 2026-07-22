pub(crate) mod common;
pub(crate) mod detect_pipeline;
pub(crate) mod task_pipeline;
pub(crate) mod yolo11;
pub(crate) mod yolo26;
pub(crate) mod yolov8;

use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    yolo26::register(registry);
    yolo11::register(registry);
    yolov8::register(registry);
}
