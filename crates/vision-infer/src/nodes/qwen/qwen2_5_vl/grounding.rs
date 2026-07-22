use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    crate::nodes::qwen::common::register_vlm_node(
        registry,
        "infer.qwen.qwen2_5_vl.grounding",
        "Qwen2.5-VL Grounding",
        "grounding",
    );
}
