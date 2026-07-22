use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    crate::nodes::ultralytics::common::register_task_node(
        registry,
        "infer.ultralytics.yolov8.pose",
        "Ultralytics YOLOv8 Pose",
        &["poses", "raw"],
        crate::nodes::ultralytics::task_pipeline::pose,
    );
}
