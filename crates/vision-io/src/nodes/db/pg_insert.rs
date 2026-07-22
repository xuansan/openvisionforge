use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_db(
        registry,
        "db.pg.insert",
        "PostgreSQL Insert",
        &[],
        &["payload", "data", "stats"],
        pg_insert,
    );
}

fn pg_insert(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_operation(ctx, "pg", "insert", crate::util::payload(ctx))
}
