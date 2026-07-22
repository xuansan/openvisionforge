use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema(
        "display.json",
        "Display Json",
        "Display",
        &["data"],
        &[],
        &["data", "files"],
        display_json,
    ));
}

fn display_json(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let data = ctx.inputs.get("data").cloned().unwrap_or(Value::None);
    let manifest = super::display_path(ctx, "json");
    let value = serde_json::json!({
        "kind": "display.json",
        "run_id": ctx.execution.run_id,
        "node_id": ctx.node.id,
        "data": data,
    });
    let path = crate::util::write_json(manifest, &value)?;

    Ok(BTreeMap::from([
        ("data".to_string(), data),
        ("files".to_string(), crate::util::file_value(path)),
    ]))
}
