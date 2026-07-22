use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    crate::nodes::ultralytics::common::register_task_node(
        registry,
        "infer.ultralytics.yolo26.obb",
        "Ultralytics YOLO26 OBB",
        &["oriented_boxes", "raw"],
        crate::nodes::ultralytics::task_pipeline::obb,
    );
}
