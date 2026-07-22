use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_db(
        registry,
        "db.mysql.query",
        "MySQL Query",
        &[],
        &[],
        mysql_query,
    );
}

fn mysql_query(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_operation(ctx, "mysql", "query", super::sql_document(ctx))
}
