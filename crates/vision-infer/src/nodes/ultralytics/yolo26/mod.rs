pub(crate) mod classify;
pub(crate) mod detect;
pub(crate) mod obb;
pub(crate) mod pose;
pub(crate) mod segment;

use vision_core::OperatorRegistry;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    detect::register(registry);
    segment::register(registry);
    classify::register(registry);
    pose::register(registry);
    obb::register(registry);
}
