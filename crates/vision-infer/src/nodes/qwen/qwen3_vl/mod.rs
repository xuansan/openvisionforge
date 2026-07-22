pub(crate) mod describe;
pub(crate) mod grounding;
pub(crate) mod inspect;
pub(crate) mod ocr;

use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    describe::register(registry);
    ocr::register(registry);
    grounding::register(registry);
    inspect::register(registry);
}
