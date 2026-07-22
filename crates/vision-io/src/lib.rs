mod nodes;
mod schema;
mod util;

use vision_core::OperatorRegistry;

pub fn register_all(registry: &mut OperatorRegistry) {
    nodes::display::register(registry);
    nodes::record::register(registry);
    nodes::storage::register(registry);
    nodes::comm::register(registry);
    nodes::action::register(registry);
    nodes::db::register(registry);
}
