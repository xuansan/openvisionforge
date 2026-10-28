use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_db(
        registry,
        "db.milvus.insert",
        "Milvus Insert",
        &[],
        &["payload", "result", "final", "data", "stats", "embedding"],
        milvus_insert,
    );
}

fn milvus_insert(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_operation(ctx, "milvus", "insert", super::vector_document(ctx))
}
