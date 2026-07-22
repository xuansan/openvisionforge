use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    crate::nodes::qwen::common::register_vlm_node(
        registry,
        "infer.qwen.qwen2_5_vl.ocr",
        "Qwen2.5-VL OCR",
        "ocr",
    );
}
