use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    crate::nodes::qwen::common::register_vlm_node(
        registry,
        "infer.qwen.qwen3_vl.ocr",
        "Qwen3-VL OCR",
        "ocr",
    );
}
