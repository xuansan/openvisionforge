use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_db(
        registry,
        "db.pg.query",
        "PostgreSQL Query",
        &[],
        &[],
        pg_query,
    );
}

fn pg_query(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_operation(ctx, "pg", "query", super::sql_document(ctx))
}
