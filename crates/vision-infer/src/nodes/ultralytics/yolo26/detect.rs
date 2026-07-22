use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    crate::nodes::ultralytics::common::register_detect_node(
        registry,
        "infer.ultralytics.yolo26.detect",
        "Ultralytics YOLO26 Detect",
    );
}
