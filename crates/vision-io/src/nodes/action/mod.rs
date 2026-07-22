mod alarm_trigger;
mod camera_trigger;
mod io_analog_write;
mod io_digital_write;
mod plc_write;
mod robot_call;
mod signal_output;

use vision_core::{NodeContext, OperatorRegistry};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    signal_output::register(registry);
    io_digital_write::register(registry);
    io_analog_write::register(registry);
    plc_write::register(registry);
    alarm_trigger::register(registry);
    robot_call::register(registry);
    camera_trigger::register(registry);
}

fn register_action(
    registry: &mut OperatorRegistry,
    id: &'static str,
    name: &'static str,
    run: vision_core::OperatorFn,
) {
    registry.register(crate::schema::schema(
        id,
        name,
        "Action",
        &[],
        &["payload", "data", "stats", "status", "signal"],
        &["action", "files"],
        run,
    ));
}

fn write_action(
    ctx: &NodeContext<'_>,
    action_type: &str,
) -> vision_core::Result<std::collections::BTreeMap<String, vision_core::Value>> {
    let action = action_json(ctx, action_type);
    let output = crate::util::output_path(ctx, "outbox", "action", "json");
    let path = crate::util::write_json(output, &action)?;
    Ok(std::collections::BTreeMap::from([
        ("action".to_string(), vision_core::Value::Json(action)),
        ("files".to_string(), crate::util::file_value(path)),
    ]))
}

fn action_json(ctx: &NodeContext<'_>, action_type: &str) -> serde_json::Value {
    serde_json::json!({
        "run_id": ctx.execution.run_id,
        "node_id": ctx.node.id,
        "action_type": action_type,
        "target": crate::util::text(&ctx.node.params, "target", ""),
        "address": crate::util::text(&ctx.node.params, "address", ""),
        "channel": crate::util::text(&ctx.node.params, "channel", ""),
        "value": crate::util::text(&ctx.node.params, "value", ""),
        "payload": crate::util::payload(ctx),
        "inputs": crate::util::input_json(ctx),
        "mode": "outbox",
    })
}
