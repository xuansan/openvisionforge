use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_action(
        registry,
        "action.io.digital_write",
        "Digital IO Write",
        io_digital_write,
    );
}

fn io_digital_write(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_action(ctx, "io.digital_write")
}
