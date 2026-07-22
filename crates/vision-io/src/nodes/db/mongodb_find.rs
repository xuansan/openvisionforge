use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_db(
        registry,
        "db.mongodb.find",
        "MongoDB Find",
        &[],
        &[],
        mongodb_find,
    );
}

fn mongodb_find(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let filter = ctx
        .node
        .params
        .get("filter")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    super::write_operation(ctx, "mongodb", "find", filter)
}
