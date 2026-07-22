use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    crate::nodes::ultralytics::common::register_task_node(
        registry,
        "infer.ultralytics.yolo11.pose",
        "Ultralytics YOLO11 Pose",
        &["poses", "raw"],
        crate::nodes::ultralytics::task_pipeline::pose,
    );
}
