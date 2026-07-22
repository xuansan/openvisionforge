use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, TriggerInfo, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema(
        "action.camera.trigger",
        "Camera Trigger",
        "Action",
        &[],
        &["payload", "data"],
        &["trigger", "files"],
        camera_trigger,
    ));
}

fn camera_trigger(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let trigger = TriggerInfo {
        trigger_id: crate::util::text(&ctx.node.params, "trigger_id", &ctx.execution.run_id),
        trigger_type: crate::util::text(&ctx.node.params, "trigger_type", "software"),
        source: crate::util::text(&ctx.node.params, "source", "workflow"),
    };
    let action = super::action_json(ctx, "camera.trigger");
    let output = crate::util::output_path(ctx, "outbox", "action", "json");
    let path = crate::util::write_json(output, &action)?;
    Ok(BTreeMap::from([
        ("trigger".to_string(), Value::Trigger(trigger)),
        ("files".to_string(), crate::util::file_value(path)),
    ]))
}
