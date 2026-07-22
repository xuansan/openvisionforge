use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, SignalOutput, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema(
        "action.signal.output",
        "Signal Output",
        "Action",
        &[],
        &["payload", "data", "stats", "status"],
        &["signal"],
        signal_output,
    ));
}

fn signal_output(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let value = crate::util::text(&ctx.node.params, "value", "OK");
    Ok(crate::util::one(
        "signal",
        Value::Signal(SignalOutput {
            channel: crate::util::text(&ctx.node.params, "channel", "DO0"),
            value,
        }),
    ))
}
