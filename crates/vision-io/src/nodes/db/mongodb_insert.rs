use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_db(
        registry,
        "db.mongodb.insert",
        "MongoDB Insert",
        &[],
        &["payload", "data", "stats"],
        mongodb_insert,
    );
}

fn mongodb_insert(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let document = crate::util::payload(ctx);
    super::write_operation(ctx, "mongodb", "insert", document)
}
