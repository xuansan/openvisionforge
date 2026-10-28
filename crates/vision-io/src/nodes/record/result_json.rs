use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, ParamKind, ParamSchema, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema_with_params(
        "record.result.json",
        "Save Inspection Result",
        "Record",
        &[],
        &[
            "final",
            "payload",
            "data",
            "stats",
            "status",
            "image",
            "annotated",
            "measurements",
            "detections",
            "rules",
            "signal",
            "files",
            "trace",
        ],
        &["result", "files"],
        RESULT_PARAMS,
        result_json,
    ));
}

const RESULT_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "output",
        "Output File",
        ParamKind::FilePath,
        false,
        None,
        "Inspection result JSON path",
        &[],
    )
    .group("File")
    .order(10)
    .accept(&["json"]),
    ParamSchema::new(
        "product_id",
        "Product ID",
        ParamKind::String,
        false,
        Some(""),
        "Product or serial identifier",
        &[],
    )
    .group("Traceability")
    .order(20),
    ParamSchema::new(
        "batch_id",
        "Batch ID",
        ParamKind::String,
        false,
        Some(""),
        "Production batch identifier",
        &[],
    )
    .group("Traceability")
    .order(30),
    ParamSchema::new(
        "model_version",
        "Model Version",
        ParamKind::String,
        false,
        Some(""),
        "Inference model version",
        &[],
    )
    .group("Traceability")
    .order(40),
    ParamSchema::new(
        "calibration_version",
        "Calibration Version",
        ParamKind::String,
        false,
        Some(""),
        "Camera or scale calibration version",
        &[],
    )
    .group("Traceability")
    .order(50),
];

fn result_json(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let output = crate::util::output_path(ctx, "output", "record", "json");
    let final_result = match ctx.inputs.get("final") {
        Some(Value::Final(result)) => serde_json::to_value(result)?,
        _ => serde_json::Value::Null,
    };
    let status = match ctx.inputs.get("final") {
        Some(Value::Final(result)) => result.status.clone(),
        _ => match ctx.inputs.get("status") {
            Some(Value::Text(status)) => status.clone(),
            _ => "UNKNOWN".to_string(),
        },
    };
    let value = serde_json::json!({
        "schema_version": "1.0",
        "run_id": ctx.execution.run_id,
        "node_id": ctx.node.id,
        "status": status,
        "traceability": {
            "product_id": text(ctx, "product_id"),
            "batch_id": text(ctx, "batch_id"),
            "model_version": text(ctx, "model_version"),
            "calibration_version": text(ctx, "calibration_version"),
        },
        "final_result": final_result,
        "payload": crate::util::payload(ctx),
        "inputs": crate::util::input_json(ctx),
    });
    let path = crate::util::write_json(output, &value)?;
    Ok(BTreeMap::from([
        ("result".to_string(), Value::Json(value)),
        ("files".to_string(), crate::util::file_value(path)),
    ]))
}

fn text(ctx: &NodeContext<'_>, key: &str) -> String {
    ctx.node
        .params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string()
}
