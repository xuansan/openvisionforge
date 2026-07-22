use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    crate::nodes::ultralytics::common::register_task_node(
        registry,
        "infer.ultralytics.yolo26.pose",
        "Ultralytics YOLO26 Pose",
        &["poses", "raw"],
        crate::nodes::ultralytics::task_pipeline::pose,
    );
}
