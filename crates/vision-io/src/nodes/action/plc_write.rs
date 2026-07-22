use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_action(registry, "action.plc.write", "PLC Write", plc_write);
}

fn plc_write(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_action(ctx, "plc.write")
}
