use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    crate::nodes::ultralytics::common::register_task_node(
        registry,
        "infer.ultralytics.yolo11.obb",
        "Ultralytics YOLO11 OBB",
        &["oriented_boxes", "raw"],
        crate::nodes::ultralytics::task_pipeline::obb,
    );
}
