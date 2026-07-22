use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_action(registry, "action.robot.call", "Robot Call", robot_call);
}

fn robot_call(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_action(ctx, "robot.call")
}
