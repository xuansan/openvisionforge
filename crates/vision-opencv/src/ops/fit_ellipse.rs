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
        "opencv.measure.fit_ellipse",
        "Fit Ellipse",
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
        "fit_ellipse",
        min_area.as_str(),
        max_contours.as_str(),
    ])?;
    let px_per_mm = number(&ctx.node.params, "px_per_mm", 0.0);
    let measurements = raw
        .get("items")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .flat_map(|item| ellipse_measurements(item, px_per_mm))
        .collect::<Vec<_>>();
    Ok(BTreeMap::from([
        (
            "measurements".to_string(),
            Value::Measurements(MeasurementSet { measurements }),
        ),
        ("raw".to_string(), Value::Json(raw)),
    ]))
}

fn ellipse_measurements(item: &serde_json::Value, px_per_mm: f64) -> Vec<MeasurementValue> {
    let index = item
        .get("index")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let quality = item.get("quality").and_then(|value| value.as_f64());
    [
        ("center_x", "px"),
        ("center_y", "px"),
        ("major_axis", "length"),
        ("minor_axis", "length"),
        ("angle_deg", "deg"),
    ]
    .iter()
    .filter_map(|(field, unit)| {
        item.get(*field)
            .and_then(|value| value.as_f64())
            .map(|value| MeasurementValue {
                name: format!("fit_ellipse.{index}.{field}"),
                value_px: Some(value),
                value_mm: (*unit == "length" && px_per_mm > 0.0).then_some(value / px_per_mm),
                unit: unit.to_string(),
                confidence: quality,
                quality_score: quality,
                ok: quality.map(|score| score > 0.0),
                reason: item
                    .get("error")
                    .and_then(|value| value.as_str())
                    .map(str::to_string),
            })
    })
    .collect()
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
