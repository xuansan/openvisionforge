use std::collections::BTreeMap;

use vision_core::{
    MeasurementSet, MeasurementValue, NodeContext, OperatorRegistry, ParamKind, ParamSchema,
    Result, Value,
};

use crate::{
    backend::{bridge_json, image, image_path, integer, number},
    schema::schema_with_params,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema_with_params(
        "opencv.measure.fit_circle",
        "Fit Circle",
        "OpenCv/GeometryMeasure",
        &["image"],
        &[],
        &["measurements", "raw"],
        PARAMS,
        run,
    ));
}

fn run(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let min_area = number(&ctx.node.params, "min_area", 1.0).to_string();
    let max_contours = integer(&ctx.node.params, "max_contours", 20).to_string();
    let raw = bridge_json(&[
        "fit_shape",
        input.as_str(),
        "fit_circle",
        min_area.as_str(),
        max_contours.as_str(),
    ])?;
    let px_per_mm = number(&ctx.node.params, "px_per_mm", 0.0);
    let measurements = raw
        .get("items")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .flat_map(|item| circle_measurements(item, px_per_mm))
        .collect::<Vec<_>>();
    Ok(BTreeMap::from([
        (
            "measurements".to_string(),
            Value::Measurements(MeasurementSet { measurements }),
        ),
        ("raw".to_string(), Value::Json(raw)),
    ]))
}

fn circle_measurements(item: &serde_json::Value, px_per_mm: f64) -> Vec<MeasurementValue> {
    let index = item
        .get("index")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let quality = item.get("quality").and_then(|value| value.as_f64());
    ["center_x", "center_y", "radius", "diameter"]
        .iter()
        .filter_map(|field| {
            item.get(*field)
                .and_then(|value| value.as_f64())
                .map(|value| {
                    let unit = if *field == "center_x" || *field == "center_y" {
                        "px"
                    } else {
                        "length"
                    };
                    measurement(
                        format!("fit_circle.{index}.{field}"),
                        value,
                        unit,
                        px_per_mm,
                        quality,
                    )
                })
        })
        .collect()
}

fn measurement(
    name: String,
    value_px: f64,
    unit: &str,
    px_per_mm: f64,
    quality: Option<f64>,
) -> MeasurementValue {
    MeasurementValue {
        name,
        value_px: Some(value_px),
        value_mm: (px_per_mm > 0.0).then_some(value_px / px_per_mm),
        unit: unit.to_string(),
        confidence: quality,
        quality_score: quality,
        ok: quality.map(|score| score >= 0.0),
        reason: None,
    }
}

const PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "min_area",
        "Min Area",
        ParamKind::Number,
        false,
        Some("1"),
        "Ignore contours smaller than this pixel area",
        &[],
    )
    .group("Contour")
    .order(10)
    .unit("px^2")
    .range(0.0, 100000000.0, 1.0),
    ParamSchema::new(
        "max_contours",
        "Max Contours",
        ParamKind::Number,
        false,
        Some("20"),
        "Maximum contours to fit",
        &[],
    )
    .group("Contour")
    .order(20)
    .range(1.0, 10000.0, 1.0),
    ParamSchema::new(
        "px_per_mm",
        "Px Per Mm",
        ParamKind::Number,
        false,
        None,
        "Scale for physical units",
        &[],
    )
    .group("Scale")
    .order(100)
    .unit("px/mm"),
];
