use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_action(
        registry,
        "action.io.analog_write",
        "Analog IO Write",
        io_analog_write,
    );
}

fn io_analog_write(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_action(ctx, "io.analog_write")
}
