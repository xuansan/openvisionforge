use std::{collections::BTreeMap, fs};

use vision_core::{NodeContext, OperatorRegistry, ParamKind, ParamSchema, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema_with_params(
        "storage.nas.read",
        "NAS Storage Read",
        "Storage",
        &[],
        &[],
        &["data"],
        NAS_READ_PARAMS,
        nas_read,
    ));
}

const NAS_READ_PARAMS: &[ParamSchema] = &[ParamSchema::new(
    "path",
    "Path",
    ParamKind::FilePath,
    true,
    None,
    "NAS or network share input path",
    &[],
)
.group("Storage")
.order(10)
.accept(&[".json", ".txt"])];

fn nas_read(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let path = crate::util::text(&ctx.node.params, "path", "");
    let text = fs::read_to_string(path)?;
    let data = serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!(text));
    Ok(crate::util::one("data", Value::Json(data)))
}
