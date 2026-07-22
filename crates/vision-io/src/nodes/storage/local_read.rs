use std::{collections::BTreeMap, fs};

use vision_core::{NodeContext, OperatorRegistry, ParamKind, ParamSchema, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema_with_params(
        "storage.local.read",
        "Local Storage Read",
        "Storage",
        &[],
        &[],
        &["data"],
        LOCAL_READ_PARAMS,
        local_read,
    ));
}

const LOCAL_READ_PARAMS: &[ParamSchema] = &[ParamSchema::new(
    "path",
    "Path",
    ParamKind::FilePath,
    true,
    None,
    "Input file path",
    &[],
)
.group("Storage")
.order(10)
.accept(&[".json", ".txt"])];

fn local_read(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let path = crate::util::text(&ctx.node.params, "path", "");
    let text = fs::read_to_string(path)?;
    let data = serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!(text));
    Ok(crate::util::one("data", Value::Json(data)))
}
