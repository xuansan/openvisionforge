use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, ParamKind, ParamSchema, Result, Value};

use crate::{
    backend::{bridge_json, image, image_path, one},
    schema::{schema, schema_with_params},
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema(
        "opencv.measure.binary_ratio",
        "OpenCV Binary Ratio",
        "OpenCv/Measure",
        &["image"],
        &[],
        &["stats"],
        binary_ratio,
    ));
    registry.register(schema(
        "opencv.measure.pixel_count",
        "OpenCV Pixel Count",
        "OpenCv/Measure",
        &["image"],
        &[],
        &["stats"],
        binary_ratio,
    ));
    registry.register(schema_with_params(
        "opencv.measure.contours",
        "OpenCV Contour Measure",
        "OpenCv/Measure",
        &["image"],
        &[],
        &["stats"],
        CONTOUR_PARAMS,
        contours,
    ));
}

const CONTOUR_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "min_area",
        "Min Area",
        ParamKind::Number,
        false,
        Some("1"),
        "Ignore contours smaller than this pixel area",
        &[],
    )
    .group("Measure")
    .order(10)
    .unit("px^2")
    .range(0.0, 100000000.0, 1.0),
    ParamSchema::new(
        "max_contours",
        "Max Contours",
        ParamKind::Number,
        false,
        Some("20"),
        "Maximum number of contours returned",
        &[],
    )
    .group("Measure")
    .order(20)
    .range(1.0, 10000.0, 1.0),
];

fn binary_ratio(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let stats = bridge_json(&["ratio", input.as_str()])?;
    Ok(one("stats", Value::Json(stats)))
}

fn contours(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let min_area = crate::backend::number(&ctx.node.params, "min_area", 1.0).to_string();
    let max_contours = crate::backend::integer(&ctx.node.params, "max_contours", 20).to_string();
    let stats = bridge_json(&[
        "contours",
        input.as_str(),
        min_area.as_str(),
        max_contours.as_str(),
    ])?;
    Ok(one("stats", Value::Json(stats)))
}
