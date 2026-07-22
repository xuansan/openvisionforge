use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema(
        "record.result.json",
        "Save Result Json",
        "Record",
        &[],
        &["payload", "data", "stats", "status", "image", "signal"],
        &["files"],
        result_json,
    ));
}

fn result_json(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let output = crate::util::output_path(ctx, "output", "record", "json");
    let value = serde_json::json!({
        "run_id": ctx.execution.run_id,
        "node_id": ctx.node.id,
        "payload": crate::util::payload(ctx),
        "inputs": crate::util::input_json(ctx),
    });
    let path = crate::util::write_json(output, &value)?;
    Ok(crate::util::one("files", crate::util::file_value(path)))
}
