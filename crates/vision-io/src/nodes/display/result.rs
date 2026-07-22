use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema(
        "display.result",
        "Display Result",
        "Display",
        &[],
        &["final", "rule", "rules", "status", "data", "payload"],
        &["result", "files"],
        display_result,
    ));
}

fn display_result(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let result = result_value(ctx);
    let manifest = super::display_path(ctx, "json");
    let value = serde_json::json!({
        "kind": "display.result",
        "run_id": ctx.execution.run_id,
        "node_id": ctx.node.id,
        "result": result,
        "inputs": crate::util::input_json(ctx),
    });
    let path = crate::util::write_json(manifest, &value)?;

    Ok(BTreeMap::from([
        ("result".to_string(), result),
        ("files".to_string(), crate::util::file_value(path)),
    ]))
}

fn result_value(ctx: &NodeContext<'_>) -> Value {
    ctx.inputs
        .get("final")
        .or_else(|| ctx.inputs.get("rule"))
        .or_else(|| ctx.inputs.get("rules"))
        .or_else(|| ctx.inputs.get("status"))
        .or_else(|| ctx.inputs.get("data"))
        .or_else(|| ctx.inputs.get("payload"))
        .cloned()
        .unwrap_or(Value::None)
}
