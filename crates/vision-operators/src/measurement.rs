use std::collections::BTreeMap;

use vision_core::{
    MeasurementSet, NodeContext, OperatorRegistry, ParamKind, ParamSchema, Result, ScaleContext,
    Value,
};

use crate::schema::schema_with_params;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema_with_params(
        "measure.scale.manual",
        "Manual Scale",
        "Measure/Calibration",
        &[],
        &[],
        &["scale"],
        MANUAL_SCALE_PARAMS,
        manual_scale,
    ));
    registry.register(schema_with_params(
        "measure.units.convert",
        "Convert Measurement Units",
        "Measure/Calibration",
        &["data"],
        &["scale"],
        &["data", "stats"],
        CONVERT_UNITS_PARAMS,
        convert_units,
    ));
    registry.register(schema_with_params(
        "measure.scale.reference",
        "Reference Length Calibration",
        "Measure/Calibration",
        &["measurements"],
        &[],
        &["scale", "stats"],
        REFERENCE_SCALE_PARAMS,
        reference_scale,
    ));
}

const MANUAL_SCALE_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "px_per_mm",
        "Pixels Per Mm",
        ParamKind::Number,
        true,
        Some("10"),
        "How many image pixels represent 1 millimeter",
        &[],
    )
    .group("Scale")
    .order(10)
    .unit("px/mm")
    .range(0.000001, 1000000.0, 0.001),
    ParamSchema::new(
        "unit",
        "Unit",
        ParamKind::Enum,
        false,
        Some("mm"),
        "Physical output unit",
        &["mm"],
    )
    .group("Scale")
    .order(20),
];

const CONVERT_UNITS_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "px_per_mm",
        "Pixels Per Mm",
        ParamKind::Number,
        false,
        Some(""),
        "Explicit scale used when no scale input is connected",
        &[],
    )
    .group("Scale")
    .order(10)
    .unit("px/mm")
    .range(0.000001, 1000000.0, 0.001),
    ParamSchema::new(
        "unit",
        "Unit",
        ParamKind::Enum,
        false,
        Some("mm"),
        "Physical output unit",
        &["mm"],
    )
    .group("Scale")
    .order(20),
];

const REFERENCE_SCALE_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "reference_mm",
        "Reference Length",
        ParamKind::Number,
        true,
        Some("10"),
        "Known physical length represented by the selected pixel measurement",
        &[],
    )
    .group("Reference")
    .order(10)
    .unit("mm")
    .range(0.000001, 1000000.0, 0.001),
    ParamSchema::new(
        "measurement_name",
        "Measurement Name",
        ParamKind::String,
        false,
        Some(""),
        "Optional measurement name. Uses the first pixel measurement when empty.",
        &[],
    )
    .group("Reference")
    .order(20),
];

fn manual_scale(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let px_per_mm = number(&ctx.node.params, "px_per_mm", 10.0);
    let unit = text(&ctx.node.params, "unit", "mm");
    Ok(BTreeMap::from([(
        "scale".to_string(),
        Value::Scale(ScaleContext {
            px_per_mm: Some(px_per_mm),
            unit,
        }),
    )]))
}

fn convert_units(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let px_per_mm = scale_from_input(ctx)
        .or_else(|| optional_number(&ctx.node.params, "px_per_mm"))
        .filter(|value| *value > 0.0)
        .ok_or_else(|| vision_core::VisionError::Operator {
            node: ctx.node.id.clone(),
            message: "measure.units.convert requires a positive scale input or px_per_mm parameter"
                .to_string(),
        })?;
    let unit = text(&ctx.node.params, "unit", "mm");
    match ctx.inputs.get("data") {
        Some(Value::Measurements(set)) => {
            let measurements = convert_measurements(set, px_per_mm, &unit);
            let stats = serde_json::to_value(&measurements)?;
            Ok(BTreeMap::from([
                ("data".to_string(), Value::Measurements(measurements)),
                ("stats".to_string(), Value::Json(stats)),
            ]))
        }
        Some(Value::Json(json)) => {
            let data = convert_json(json, px_per_mm, &unit);
            Ok(BTreeMap::from([
                ("data".to_string(), Value::Json(data.clone())),
                ("stats".to_string(), Value::Json(data)),
            ]))
        }
        Some(value) => {
            let data = serde_json::json!({ "summary": value.summary() });
            Ok(BTreeMap::from([
                ("data".to_string(), Value::Json(data.clone())),
                ("stats".to_string(), Value::Json(data)),
            ]))
        }
        None => Ok(BTreeMap::from([
            ("data".to_string(), Value::Json(serde_json::json!({}))),
            ("stats".to_string(), Value::Json(serde_json::json!({}))),
        ])),
    }
}

fn reference_scale(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let set = match ctx.inputs.get("measurements") {
        Some(Value::Measurements(set)) => set,
        _ => {
            return Err(vision_core::VisionError::Operator {
                node: ctx.node.id.clone(),
                message: "measure.scale.reference requires measurements input".to_string(),
            })
        }
    };
    let selected = text(&ctx.node.params, "measurement_name", "");
    let measurement = if selected.is_empty() {
        set.measurements.iter().find(|item| item.value_px.is_some())
    } else {
        set.measurements
            .iter()
            .find(|item| item.name == selected && item.value_px.is_some())
    }
    .ok_or_else(|| vision_core::VisionError::Operator {
        node: ctx.node.id.clone(),
        message: "reference measurement with value_px was not found".to_string(),
    })?;
    let reference_mm = number(&ctx.node.params, "reference_mm", 10.0);
    let measured_px = measurement.value_px.unwrap_or_default();
    if measured_px <= 0.0 || reference_mm <= 0.0 {
        return Err(vision_core::VisionError::Operator {
            node: ctx.node.id.clone(),
            message: "reference pixel length and reference_mm must be greater than zero"
                .to_string(),
        });
    }
    let px_per_mm = measured_px / reference_mm;
    let scale = ScaleContext {
        px_per_mm: Some(px_per_mm),
        unit: "mm".to_string(),
    };
    Ok(BTreeMap::from([
        ("scale".to_string(), Value::Scale(scale)),
        (
            "stats".to_string(),
            Value::Json(serde_json::json!({
                "measurement_name": measurement.name,
                "measured_px": measured_px,
                "reference_mm": reference_mm,
                "px_per_mm": px_per_mm,
                "mm_per_px": 1.0 / px_per_mm,
            })),
        ),
    ]))
}

fn convert_measurements(set: &MeasurementSet, px_per_mm: f64, unit: &str) -> MeasurementSet {
    let mut converted = set.clone();
    for item in &mut converted.measurements {
        let Some(value_px) = item.value_px else {
            continue;
        };
        let is_area = item.unit == "area" || item.name.ends_with("_px2");
        let is_length = item.unit == "length"
            || item.unit == "px"
            || item.name.ends_with("_px")
            || item.name.contains("perimeter");
        if is_area {
            item.value_mm = Some(value_px / (px_per_mm * px_per_mm));
            item.unit = format!("{unit}2");
        } else if is_length {
            item.value_mm = Some(value_px / px_per_mm);
            item.unit = unit.to_string();
        }
    }
    converted
}

fn convert_json(json: &serde_json::Value, px_per_mm: f64, unit: &str) -> serde_json::Value {
    let mut out = json.clone();
    if let Some(object) = out.as_object_mut() {
        object.insert(
            "scale".to_string(),
            serde_json::json!({
                "px_per_mm": px_per_mm,
                "mm_per_px": 1.0 / px_per_mm,
                "unit": unit,
            }),
        );
        let keys = object.keys().cloned().collect::<Vec<_>>();
        for key in keys {
            let Some(value_px) = object.get(&key).and_then(serde_json::Value::as_f64) else {
                continue;
            };
            if key.ends_with("_px") {
                let physical_key = key.trim_end_matches("_px").to_string() + "_" + unit;
                object.insert(physical_key, serde_json::Value::from(value_px / px_per_mm));
            }
            if key.ends_with("_px2") {
                let physical_key = key.trim_end_matches("_px2").to_string() + "_" + unit + "2";
                object.insert(
                    physical_key,
                    serde_json::Value::from(value_px / (px_per_mm * px_per_mm)),
                );
            }
        }
    }
    out
}

fn scale_from_input(ctx: &NodeContext<'_>) -> Option<f64> {
    match ctx.inputs.get("scale")? {
        Value::Scale(scale) => scale.px_per_mm,
        Value::Json(json) => json.get("px_per_mm").and_then(serde_json::Value::as_f64),
        _ => None,
    }
}

fn optional_number(params: &serde_json::Value, key: &str) -> Option<f64> {
    params.get(key).and_then(|value| {
        value
            .as_f64()
            .or_else(|| value.as_str().and_then(|text| text.parse::<f64>().ok()))
    })
}

fn number(params: &serde_json::Value, key: &str, default: f64) -> f64 {
    optional_number(params, key).unwrap_or(default)
}

fn text(params: &serde_json::Value, key: &str, default: &str) -> String {
    params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or(default)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use vision_core::{ExecutionContext, MeasurementValue, WorkflowNode};

    #[test]
    fn converts_length_and_area_measurements() {
        let input = MeasurementSet {
            measurements: vec![
                MeasurementValue {
                    name: "width_px".to_string(),
                    value_px: Some(200.0),
                    value_mm: None,
                    unit: "length".to_string(),
                    confidence: None,
                    quality_score: None,
                    ok: None,
                    reason: None,
                },
                MeasurementValue {
                    name: "area_px".to_string(),
                    value_px: Some(40000.0),
                    value_mm: None,
                    unit: "area".to_string(),
                    confidence: None,
                    quality_score: None,
                    ok: None,
                    reason: None,
                },
            ],
        };

        let converted = convert_measurements(&input, 20.0, "mm");

        assert_eq!(converted.measurements[0].value_mm, Some(10.0));
        assert_eq!(converted.measurements[0].unit, "mm");
        assert_eq!(converted.measurements[1].value_mm, Some(100.0));
        assert_eq!(converted.measurements[1].unit, "mm2");
    }

    #[test]
    fn rejects_unit_conversion_without_scale() {
        let node = WorkflowNode {
            id: "convert".to_string(),
            operator: "measure.units.convert".to_string(),
            params: serde_json::json!({}),
            execution: Default::default(),
        };
        let inputs = BTreeMap::from([(
            "data".to_string(),
            Value::Measurements(MeasurementSet {
                measurements: vec![],
            }),
        )]);
        let execution = ExecutionContext {
            run_id: "test".to_string(),
            records_dir: std::path::PathBuf::from("records/test"),
        };
        let ctx = NodeContext {
            node: &node,
            inputs,
            execution: &execution,
        };

        let err = convert_units(&ctx).expect_err("missing scale must be rejected");

        assert!(err.to_string().contains("requires a positive scale"));
    }
}
