mod backend;
mod ops;
mod schema;

use vision_core::OperatorRegistry;

pub fn register_all(registry: &mut OperatorRegistry) {
    ops::image::register(registry);
    ops::color::register(registry);
    ops::threshold::register(registry);
    ops::filter::register(registry);
    ops::enhance::register(registry);
    ops::morph::register(registry);
    ops::analysis::register(registry);
    ops::geometry::register(registry);
    ops::location::register(registry);
    ops::measure::register(registry);
    ops::transform::register(registry);
    ops::edge::register(registry);
    ops::draw::register(registry);
}
