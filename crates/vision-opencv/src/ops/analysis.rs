use std::collections::BTreeMap;

use vision_core::{
    ContourResult, ContourSet, MeasurementSet, MeasurementValue, NodeContext, OperatorRegistry,
    ParamKind, ParamSchema, Rect, Region, Result, RuleResult, ScaleContext, Value,
};

use crate::{
    backend::{bridge_json, image, image_path, integer, number, one},
    schema::{schema, schema_with_params},
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema_with_params(
        "opencv.analysis.connected_components",
        "Connected Components",
        "OpenCv/BlobContour",
        &["image"],
        &[],
        &["contours", "stats"],
        CONTOUR_PARAMS,
        connected_components,
    ));
    registry.register(schema_with_params(
        "opencv.analysis.find_contours",
        "Find Contours",
        "OpenCv/BlobContour",
        &["image"],
        &[],
        &["contours", "stats"],
        CONTOUR_PARAMS,
        find_contours,
    ));
    registry.register(schema_with_params(
        "opencv.analysis.contour_filter",
        "Contour Filter",
        "OpenCv/BlobContour",
        &["contours"],
        &[],
        &["contours", "measurements"],
        FILTER_PARAMS,
        contour_filter,
    ));
    registry.register(schema_with_params(
        "opencv.analysis.area_filter",
        "Area Filter",
        "OpenCv/BlobContour",
        &["contours"],
        &[],
        &["contours", "measurements"],
        FILTER_PARAMS,
        contour_filter,
    ));
    registry.register(schema(
        "opencv.analysis.roundness",
        "Roundness",
        "OpenCv/BlobContour",
        &["contours"],
        &[],
        &["measurements"],
        roundness,
    ));
    registry.register(schema(
        "opencv.analysis.aspect_ratio",
        "Aspect Ratio",
        "OpenCv/BlobContour",
        &["contours"],
        &[],
        &["measurements"],
        aspect_ratio,
    ));
    registry.register(schema(
        "opencv.analysis.bounding_rect",
        "Bounding Rect",
        "OpenCv/BlobContour",
        &["contours"],
        &[],
        &["measurements", "contours"],
        bounding_rect,
    ));
    registry.register(schema(
        "opencv.analysis.convex_hull",
        "Convex Hull",
        "OpenCv/BlobContour",
        &["contours"],
        &[],
        &["contours", "measurements"],
        passthrough_with_note,
    ));
    registry.register(schema(
        "opencv.analysis.defect_points",
        "Defect Point Detect",
        "OpenCv/Defect",
        &["contours"],
        &[],
        &["regions", "measurements"],
        defect_points,
    ));
    registry.register(schema_with_params(
        "opencv.analysis.holes",
        "Hole Detect",
        "OpenCv/Defect",
        &["image"],
        &[],
        &["contours", "stats"],
        CONTOUR_PARAMS,
        find_contours,
    ));
    registry.register(schema_with_params(
        "opencv.analysis.spots",
        "Spot Detect",
        "OpenCv/Defect",
        &["image"],
        &[],
        &["contours", "stats"],
        CONTOUR_PARAMS,
        find_contours,
    ));
    registry.register(schema(
        "opencv.analysis.burrs",
        "Burr Detect",
        "OpenCv/Defect",
        &["contours"],
        &[],
        &["rule", "measurements"],
        burrs,
    ));
    registry.register(schema(
        "opencv.analysis.standard_diff",
        "Standard Part Diff",
        "OpenCv/Defect",
        &["image"],
        &["template"],
        &["measurements", "rule"],
        standard_diff,
    ));
}

const CONTOUR_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "min_area",
        "Min Area",
        ParamKind::Number,
        false,
        Some("1"),
        "Ignore regions below this area",
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
        Some("100"),
        "Maximum contours returned",
        &[],
    )
    .group("Contour")
    .order(20)
    .range(1.0, 100000.0, 1.0),
];

const FILTER_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "min_area",
        "Min Area",
        ParamKind::Number,
        false,
        Some("0"),
        "Minimum contour area",
        &[],
    )
    .group("Filter")
    .order(10)
    .unit("px^2"),
    ParamSchema::new(
        "max_area",
        "Max Area",
        ParamKind::Number,
        false,
        Some("999999999"),
        "Maximum contour area",
        &[],
    )
    .group("Filter")
    .order(20)
    .unit("px^2"),
    ParamSchema::new(
        "min_roundness",
        "Min Roundness",
        ParamKind::Number,
        false,
        Some("0"),
        "Minimum roundness",
        &[],
    )
    .group("Filter")
    .order(30)
    .range(0.0, 1.0, 0.01),
    ParamSchema::new(
        "max_aspect_ratio",
        "Max Aspect Ratio",
        ParamKind::Number,
        false,
        Some("999999"),
        "Maximum aspect ratio",
        &[],
    )
    .group("Filter")
    .order(40),
];

fn connected_components(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    contour_from_image(ctx)
}

fn find_contours(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    contour_from_image(ctx)
}

fn contour_from_image(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let min_area = number(&ctx.node.params, "min_area", 1.0).to_string();
    let max_contours = integer(&ctx.node.params, "max_contours", 100).to_string();
    let stats = bridge_json(&[
        "contours",
        input.as_str(),
        min_area.as_str(),
        max_contours.as_str(),
    ])?;
    let contours = contour_set(&stats, ScaleContext::default());
    Ok(BTreeMap::from([
        ("contours".to_string(), Value::Contours(contours)),
        ("stats".to_string(), Value::Json(stats)),
    ]))
}

fn contour_filter(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let min_area = number(&ctx.node.params, "min_area", 0.0);
    let max_area = number(&ctx.node.params, "max_area", 999999999.0);
    let min_roundness = number(&ctx.node.params, "min_roundness", 0.0);
    let max_aspect_ratio = number(&ctx.node.params, "max_aspect_ratio", 999999.0);
    let mut set = contours(ctx)?;
    set.contours.retain(|item| {
        item.area_px >= min_area
            && item.area_px <= max_area
            && item.roundness.unwrap_or(0.0) >= min_roundness
            && item.aspect_ratio.unwrap_or(0.0) <= max_aspect_ratio
    });
    set.count = set.contours.len();
    let measurements = contour_measurements(&set, "filtered_count");
    Ok(BTreeMap::from([
        ("contours".to_string(), Value::Contours(set)),
        (
            "measurements".to_string(),
            Value::Measurements(measurements),
        ),
    ]))
}

fn roundness(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let set = contours(ctx)?;
    Ok(one(
        "measurements",
        Value::Measurements(named_measurements(&set, "roundness", |item| item.roundness)),
    ))
}

fn aspect_ratio(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let set = contours(ctx)?;
    Ok(one(
        "measurements",
        Value::Measurements(named_measurements(&set, "aspect_ratio", |item| {
            item.aspect_ratio
        })),
    ))
}

fn bounding_rect(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let set = contours(ctx)?;
    Ok(BTreeMap::from([
        (
            "measurements".to_string(),
            Value::Measurements(contour_measurements(&set, "bbox")),
        ),
        ("contours".to_string(), Value::Contours(set)),
    ]))
}

fn passthrough_with_note(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let set = contours(ctx)?;
    Ok(BTreeMap::from([
        ("contours".to_string(), Value::Contours(set.clone())),
        (
            "measurements".to_string(),
            Value::Measurements(contour_measurements(&set, "convex_hull_placeholder")),
        ),
    ]))
}

fn defect_points(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let set = contours(ctx)?;
    let regions = set
        .contours
        .iter()
        .filter(|item| {
            item.quality_score.unwrap_or(1.0) < 0.8 || item.roundness.unwrap_or(1.0) < 0.6
        })
        .map(|item| Region {
            id: item.id.clone(),
            bbox: item.bbox.clone(),
            area_px: item.area_px,
            score: item.quality_score,
            label: Some("defect_point".to_string()),
        })
        .collect::<Vec<_>>();
    Ok(BTreeMap::from([
        ("regions".to_string(), Value::Regions(regions)),
        (
            "measurements".to_string(),
            Value::Measurements(contour_measurements(&set, "defect_candidates")),
        ),
    ]))
}

fn burrs(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let set = contours(ctx)?;
    let suspicious = set
        .contours
        .iter()
        .filter(|item| {
            item.aspect_ratio.unwrap_or(1.0) > 6.0 || item.roundness.unwrap_or(1.0) < 0.25
        })
        .count();
    let ok = suspicious == 0;
    let rule = RuleResult {
        rule_id: "burr_candidates".to_string(),
        ok,
        level: if ok {
            vision_core::QualityLevel::Ok
        } else {
            vision_core::QualityLevel::Ng
        },
        reason: if ok {
            "未发现毛刺候选"
        } else {
            "发现疑似毛刺候选"
        }
        .to_string(),
        actual: serde_json::Value::from(suspicious),
        expected: serde_json::json!({ "max": 0 }),
        source_node: Some(ctx.node.id.clone()),
        source_field: Some("aspect_ratio/roundness".to_string()),
    };
    Ok(BTreeMap::from([
        ("rule".to_string(), Value::Rule(rule)),
        (
            "measurements".to_string(),
            Value::Measurements(contour_measurements(&set, "burr_candidates")),
        ),
    ]))
}

fn standard_diff(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let value = MeasurementValue {
        name: "standard_diff_score".to_string(),
        value_px: Some(0.0),
        value_mm: None,
        unit: "score".to_string(),
        confidence: Some(1.0),
        quality_score: Some(1.0),
        ok: Some(true),
        reason: Some(format!(
            "standard diff placeholder for {}x{}",
            image.width, image.height
        )),
    };
    let rule = RuleResult {
        rule_id: "standard_diff".to_string(),
        ok: true,
        level: vision_core::QualityLevel::Ok,
        reason: "标准件差异比对接口已预留".to_string(),
        actual: serde_json::Value::from(0.0),
        expected: serde_json::Value::from(0.0),
        source_node: Some(ctx.node.id.clone()),
        source_field: Some("standard_diff_score".to_string()),
    };
    Ok(BTreeMap::from([
        (
            "measurements".to_string(),
            Value::Measurements(MeasurementSet {
                measurements: vec![value],
            }),
        ),
        ("rule".to_string(), Value::Rule(rule)),
    ]))
}

pub(crate) fn contours(ctx: &NodeContext<'_>) -> Result<ContourSet> {
    match ctx.inputs.get("contours") {
        Some(Value::Contours(set)) => Ok(set.clone()),
        Some(Value::Json(stats)) => Ok(contour_set(stats, ScaleContext::default())),
        _ => Ok(ContourSet {
            width: 0,
            height: 0,
            count: 0,
            contours: Vec::new(),
        }),
    }
}

pub(crate) fn contour_measurements(set: &ContourSet, prefix: &str) -> MeasurementSet {
    let mut measurements = Vec::new();
    measurements.push(measurement(
        format!("{prefix}.count"),
        Some(set.count as f64),
        None,
        "count",
    ));
    if let Some(first) = set.contours.first() {
        measurements.extend([
            measurement(
                format!("{prefix}.area_px"),
                Some(first.area_px),
                first.area_mm2,
                "area",
            ),
            measurement(
                format!("{prefix}.perimeter_px"),
                first.perimeter_px,
                first.perimeter_mm,
                "length",
            ),
            measurement(
                format!("{prefix}.width_px"),
                Some(first.bbox.width),
                None,
                "length",
            ),
            measurement(
                format!("{prefix}.height_px"),
                Some(first.bbox.height),
                None,
                "length",
            ),
            measurement(
                format!("{prefix}.roundness"),
                first.roundness,
                None,
                "score",
            ),
            measurement(
                format!("{prefix}.aspect_ratio"),
                first.aspect_ratio,
                None,
                "ratio",
            ),
        ]);
    }
    MeasurementSet { measurements }
}

fn named_measurements(
    set: &ContourSet,
    name: &str,
    select: impl Fn(&ContourResult) -> Option<f64>,
) -> MeasurementSet {
    MeasurementSet {
        measurements: set
            .contours
            .iter()
            .enumerate()
            .map(|(index, item)| {
                measurement(format!("{name}.{index}"), select(item), None, "score")
            })
            .collect(),
    }
}

pub(crate) fn measurement(
    name: String,
    value_px: Option<f64>,
    value_mm: Option<f64>,
    unit: &str,
) -> MeasurementValue {
    MeasurementValue {
        name,
        value_px,
        value_mm,
        unit: unit.to_string(),
        confidence: Some(1.0),
        quality_score: Some(1.0),
        ok: None,
        reason: None,
    }
}

fn contour_set(stats: &serde_json::Value, scale: ScaleContext) -> ContourSet {
    let width = stats
        .get("width")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as u32;
    let height = stats
        .get("height")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as u32;
    let contours = stats
        .get("contours")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, item)| {
            let x = f64_field(item, "x");
            let y = f64_field(item, "y");
            let w = f64_field(item, "width");
            let h = f64_field(item, "height");
            let area = f64_field(item, "area");
            let perimeter = f64_field(item, "perimeter");
            let roundness = if perimeter > 0.0 {
                Some((4.0 * std::f64::consts::PI * area / (perimeter * perimeter)).clamp(0.0, 1.0))
            } else {
                None
            };
            let aspect_ratio = if h > 0.0 { Some(w / h) } else { None };
            ContourResult {
                id: format!("contour_{index}"),
                bbox: Rect {
                    x,
                    y,
                    width: w,
                    height: h,
                },
                area_px: area,
                area_mm2: scale
                    .px_per_mm
                    .filter(|px_per_mm| *px_per_mm > 0.0)
                    .map(|px_per_mm| area / (px_per_mm * px_per_mm)),
                perimeter_px: Some(perimeter),
                perimeter_mm: scale.px_to_mm(perimeter),
                roundness,
                aspect_ratio,
                center_x: x + w / 2.0,
                center_y: y + h / 2.0,
                confidence: Some(1.0),
                quality_score: roundness,
                ok: None,
                reason: None,
            }
        })
        .collect::<Vec<_>>();
    ContourSet {
        width,
        height,
        count: contours.len(),
        contours,
    }
}

fn f64_field(value: &serde_json::Value, key: &str) -> f64 {
    value
        .get(key)
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0)
}
