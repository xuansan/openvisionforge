use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_action(
        registry,
        "action.alarm.trigger",
        "Alarm Trigger",
        alarm_trigger,
    );
}

fn alarm_trigger(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_action(ctx, "alarm.trigger")
}
