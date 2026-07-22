use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema(
        "record.trace",
        "Save Trace Snapshot",
        "Record",
        &[],
        &["payload", "data", "stats", "status"],
        &["files"],
        trace_snapshot,
    ));
}

fn trace_snapshot(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let output = crate::util::output_path(ctx, "output", "record", "trace.json");
    let value = serde_json::json!({
        "run_id": ctx.execution.run_id,
        "node_id": ctx.node.id,
        "inputs": crate::util::input_json(ctx),
    });
    let path = crate::util::write_json(output, &value)?;
    Ok(crate::util::one("files", crate::util::file_value(path)))
}
