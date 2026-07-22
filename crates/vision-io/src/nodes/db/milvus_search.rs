use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_db(
        registry,
        "db.milvus.search",
        "Milvus Search",
        &[],
        &["embedding"],
        milvus_search,
    );
}

fn milvus_search(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_operation(ctx, "milvus", "search", super::vector_document(ctx))
}
