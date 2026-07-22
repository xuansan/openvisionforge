use std::{collections::BTreeMap, fs};

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema(
        "record.report.csv",
        "Save Csv Report",
        "Record",
        &[],
        &["payload", "data", "stats", "status"],
        &["files"],
        report_csv,
    ));
}

fn report_csv(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let output = crate::util::output_path(ctx, "output", "record", "csv");
    crate::util::ensure_parent(&output)?;
    let status = crate::util::text(&ctx.node.params, "status", "UNKNOWN");
    let row = format!(
        "run_id,node_id,status,payload\n{},{},{},{}\n",
        ctx.execution.run_id,
        ctx.node.id,
        status,
        serde_json::to_string(&crate::util::payload(ctx))?.replace(',', ";")
    );
    fs::write(&output, row)?;
    Ok(crate::util::one("files", crate::util::file_value(output)))
}
