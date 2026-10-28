use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    super::register_db(
        registry,
        "db.mysql.insert",
        "MySQL Insert",
        &[],
        &["payload", "result", "final", "data", "stats"],
        mysql_insert,
    );
}

fn mysql_insert(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    super::write_operation(ctx, "mysql", "insert", crate::util::payload(ctx))
}
