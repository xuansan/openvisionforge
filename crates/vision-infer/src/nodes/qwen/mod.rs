pub(crate) mod common;
pub(crate) mod qwen2_5_vl;
pub(crate) mod qwen3_vl;

use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    qwen3_vl::register(registry);
    qwen2_5_vl::register(registry);
}
