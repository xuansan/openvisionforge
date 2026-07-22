use std::collections::BTreeMap;

use vision_core::{
    MeasurementSet, NodeContext, OperatorRegistry, ParamKind, ParamSchema, Result, Value,
};

use crate::{
    backend::{number, one},
    schema::schema_with_params,
};

use super::analysis::{contour_measurements, contours, measurement};

#[path = "fit_circle.rs"]
mod fit_circle;
#[path = "fit_ellipse.rs"]
mod fit_ellipse;
#[path = "fit_line.rs"]
mod fit_line;
#[path = "min_enclosing_circle.rs"]
mod min_enclosing_circle;

pub(crate) fn register(registry: &mut OperatorRegistry) {
    for (id, name, run) in [
        (
            "opencv.measure.point_distance",
            "Point To Point Distance",
            point_distance as vision_core::OperatorFn,
        ),
        (
            "opencv.measure.point_line_distance",
            "Point To Line Distance",
            point_line_distance,
        ),
        ("opencv.measure.line_width", "Line Width", line_width),
        (
            "opencv.measure.circle_diameter",
            "Circle Diameter",
            circle_diameter,
        ),
        (
            "opencv.measure.circle_center_distance",
            "Circle Center Distance",
            circle_center_distance,
        ),
        ("opencv.measure.angle", "Angle", angle),
        ("opencv.measure.parallelism", "Parallelism", parallelism),
        (
            "opencv.measure.perpendicularity",
            "Perpendicularity",
            perpendicularity,
        ),
        (
            "opencv.measure.concentricity",
            "Concentricity",
            concentricity,
        ),
        ("opencv.measure.gap", "Gap / Clearance", gap),
        (
            "opencv.measure.step_height",
            "Step Height Reserved",
            step_height,
        ),
        ("opencv.measure.flatness", "Flatness Reserved", flatness),
    ] {
        registry.register(schema_with_params(
            id,
            name,
            "OpenCv/GeometryMeasure",
            &[],
            &["contours"],
            &["measurements"],
            GEOMETRY_PARAMS,
            run,
        ));
    }
    registry.register(schema_with_params(
        "opencv.measure.area",
        "Area Measure",
        "OpenCv/GeometryMeasure",
        &["contours"],
        &[],
        &["measurements"],
        GEOMETRY_PARAMS,
        area,
    ));
    registry.register(schema_with_params(
        "opencv.measure.perimeter",
        "Perimeter Measure",
        "OpenCv/GeometryMeasure",
        &["contours"],
        &[],
        &["measurements"],
        GEOMETRY_PARAMS,
        perimeter,
    ));
    registry.register(schema_with_params(
        "opencv.measure.min_bounding_rect",
        "Min Bounding Rect",
        "OpenCv/GeometryMeasure",
        &["contours"],
        &[],
        &["measurements"],
        GEOMETRY_PARAMS,
        min_bounding_rect,
    ));
    registry.register(schema_with_params(
        "opencv.measure.contour_size",
        "Contour Size",
        "OpenCv/GeometryMeasure",
        &["contours"],
        &[],
        &["measurements"],
        GEOMETRY_PARAMS,
        contour_size,
    ));
    min_enclosing_circle::register(registry);
    fit_circle::register(registry);
    fit_line::register(registry);
    fit_ellipse::register(registry);
}

const GEOMETRY_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "x1",
        "X1",
        ParamKind::Number,
        false,
        Some("0"),
        "Point 1 x",
        &[],
    )
    .group("Geometry")
    .order(10),
    ParamSchema::new(
        "y1",
        "Y1",
        ParamKind::Number,
        false,
        Some("0"),
        "Point 1 y",
        &[],
    )
    .group("Geometry")
    .order(20),
    ParamSchema::new(
        "x2",
        "X2",
        ParamKind::Number,
        false,
        Some("0"),
        "Point 2 x",
        &[],
    )
    .group("Geometry")
    .order(30),
    ParamSchema::new(
        "y2",
        "Y2",
        ParamKind::Number,
        false,
        Some("0"),
        "Point 2 y",
        &[],
    )
    .group("Geometry")
    .order(40),
    ParamSchema::new(
        "x3",
        "X3",
        ParamKind::Number,
        false,
        Some("0"),
        "Point 3 x",
        &[],
    )
    .group("Geometry")
    .order(50),
    ParamSchema::new(
        "y3",
        "Y3",
        ParamKind::Number,
        false,
        Some("0"),
        "Point 3 y",
        &[],
    )
    .group("Geometry")
    .order(60),
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
    ParamSchema::new(
        "min_value",
        "Min Value",
        ParamKind::Number,
        false,
        None,
        "Optional local OK/NG minimum",
        &[],
    )
    .group("Local Judge")
    .order(110),
    ParamSchema::new(
        "max_value",
        "Max Value",
        ParamKind::Number,
        false,
        None,
        "Optional local OK/NG maximum",
        &[],
    )
    .group("Local Judge")
    .order(120),
];

fn point_distance(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let x1 = number(&ctx.node.params, "x1", 0.0);
    let y1 = number(&ctx.node.params, "y1", 0.0);
    let x2 = number(&ctx.node.params, "x2", 0.0);
    let y2 = number(&ctx.node.params, "y2", 0.0);
    out(
        ctx,
        "point_distance",
        ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt(),
        "length",
    )
}

fn point_line_distance(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let x1 = number(&ctx.node.params, "x1", 0.0);
    let y1 = number(&ctx.node.params, "y1", 0.0);
    let x2 = number(&ctx.node.params, "x2", 1.0);
    let y2 = number(&ctx.node.params, "y2", 0.0);
    let x3 = number(&ctx.node.params, "x3", 0.0);
    let y3 = number(&ctx.node.params, "y3", 0.0);
    let denom = ((y2 - y1).powi(2) + (x2 - x1).powi(2)).sqrt();
    let value = if denom > 0.0 {
        ((y2 - y1) * x3 - (x2 - x1) * y3 + x2 * y1 - y2 * x1).abs() / denom
    } else {
        0.0
    };
    out(ctx, "point_line_distance", value, "length")
}

fn line_width(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    contour_dimension(ctx, "line_width", |w, h| w.min(h))
}

fn circle_diameter(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let radius = number(&ctx.node.params, "x1", 0.0).abs();
    out(ctx, "circle_diameter", radius * 2.0, "length")
}

fn circle_center_distance(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    point_distance(ctx)
}

fn angle(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let x1 = number(&ctx.node.params, "x1", 1.0);
    let y1 = number(&ctx.node.params, "y1", 0.0);
    let x2 = number(&ctx.node.params, "x2", 0.0);
    let y2 = number(&ctx.node.params, "y2", 0.0);
    let x3 = number(&ctx.node.params, "x3", 0.0);
    let y3 = number(&ctx.node.params, "y3", 1.0);
    let a = (x1 - x2, y1 - y2);
    let b = (x3 - x2, y3 - y2);
    let dot = a.0 * b.0 + a.1 * b.1;
    let mag = (a.0.powi(2) + a.1.powi(2)).sqrt() * (b.0.powi(2) + b.1.powi(2)).sqrt();
    let deg = if mag > 0.0 {
        (dot / mag).clamp(-1.0, 1.0).acos().to_degrees()
    } else {
        0.0
    };
    raw_out("angle_deg", deg, "deg")
}

fn parallelism(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    raw_out(
        "parallelism",
        number(&ctx.node.params, "x1", 0.0).abs(),
        "score",
    )
}

fn perpendicularity(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    raw_out(
        "perpendicularity",
        number(&ctx.node.params, "x1", 0.0).abs(),
        "score",
    )
}

fn concentricity(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    point_distance(ctx)
}

fn area(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let set = contours(ctx)?;
    Ok(one(
        "measurements",
        Value::Measurements(contour_measurements(&set, "area")),
    ))
}

fn perimeter(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let set = contours(ctx)?;
    Ok(one(
        "measurements",
        Value::Measurements(contour_measurements(&set, "perimeter")),
    ))
}

fn min_bounding_rect(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let set = contours(ctx)?;
    Ok(one(
        "measurements",
        Value::Measurements(contour_measurements(&set, "min_rect")),
    ))
}

fn contour_size(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let set = contours(ctx)?;
    Ok(one(
        "measurements",
        Value::Measurements(contour_measurements(&set, "contour_size")),
    ))
}

fn gap(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    contour_dimension(ctx, "gap", |w, h| w.min(h))
}

fn step_height(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    out(ctx, "step_height_reserved", 0.0, "length")
}

fn flatness(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    out(ctx, "flatness_reserved", 0.0, "length")
}

fn contour_dimension(
    ctx: &NodeContext<'_>,
    name: &str,
    select: impl Fn(f64, f64) -> f64,
) -> Result<BTreeMap<String, Value>> {
    let set = contours(ctx)?;
    let value = set
        .contours
        .first()
        .map(|item| select(item.bbox.width, item.bbox.height))
        .unwrap_or(0.0);
    out(ctx, name, value, "length")
}

fn out(
    ctx: &NodeContext<'_>,
    name: &str,
    value_px: f64,
    unit: &str,
) -> Result<BTreeMap<String, Value>> {
    let px_per_mm = ctx
        .node
        .params
        .get("px_per_mm")
        .and_then(serde_json::Value::as_f64);
    let value_mm = px_per_mm
        .filter(|scale| *scale > 0.0)
        .map(|scale| value_px / scale);
    let min = ctx
        .node
        .params
        .get("min_value")
        .and_then(serde_json::Value::as_f64);
    let max = ctx
        .node
        .params
        .get("max_value")
        .and_then(serde_json::Value::as_f64);
    let judged_value = value_mm.unwrap_or(value_px);
    let ok = min.or(max).map(|_| {
        min.is_none_or(|min| judged_value >= min) && max.is_none_or(|max| judged_value <= max)
    });
    let mut item = measurement(name.to_string(), Some(value_px), value_mm, unit);
    item.ok = ok;
    item.reason = ok.map(|ok| {
        if ok {
            "局部测量判定通过".to_string()
        } else {
            "局部测量判定未通过".to_string()
        }
    });
    Ok(one(
        "measurements",
        Value::Measurements(MeasurementSet {
            measurements: vec![item],
        }),
    ))
}

fn raw_out(name: &str, value: f64, unit: &str) -> Result<BTreeMap<String, Value>> {
    Ok(one(
        "measurements",
        Value::Measurements(MeasurementSet {
            measurements: vec![measurement(name.to_string(), Some(value), None, unit)],
        }),
    ))
}
