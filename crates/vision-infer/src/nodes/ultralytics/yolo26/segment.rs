use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    crate::nodes::ultralytics::common::register_task_node(
        registry,
        "infer.ultralytics.yolo26.segment",
        "Ultralytics YOLO26 Segment",
        &["detections", "masks", "raw"],
        crate::nodes::ultralytics::task_pipeline::segment,
    );
}
