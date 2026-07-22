use std::collections::BTreeMap;

use vision_core::{
    LocationResult, NodeContext, OperatorRegistry, ParamKind, ParamSchema, Rect, Result, Value,
};

use crate::{
    backend::{image, number, one},
    schema::schema_with_params,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    for (id, name) in [
        ("opencv.locate.template_match", "Template Match"),
        (
            "opencv.locate.template_match_rotated",
            "Rotated Template Match",
        ),
        (
            "opencv.locate.template_match_multiscale",
            "Multi Scale Template Match",
        ),
        ("opencv.locate.feature_match", "Feature Match"),
        ("opencv.locate.circle", "Circle Locate"),
        ("opencv.locate.line", "Line Locate"),
        ("opencv.locate.corner", "Corner Locate"),
        ("opencv.locate.edge", "Edge Locate"),
        ("opencv.locate.blob", "Blob Locate"),
        ("opencv.locate.keypoint", "Keypoint Locate"),
        ("opencv.locate.pose", "Pose Estimate Reserved"),
    ] {
        registry.register(schema_with_params(
            id,
            name,
            "OpenCv/Locate",
            &["image"],
            &["template", "contours"],
            &["location"],
            LOCATE_PARAMS,
            locate,
        ));
    }
}

const LOCATE_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "score_threshold",
        "Score Threshold",
        ParamKind::Number,
        false,
        Some("0.5"),
        "Minimum locate score",
        &[],
    )
    .group("Locate")
    .order(10)
    .range(0.0, 1.0, 0.01),
    ParamSchema::new(
        "x",
        "Default X",
        ParamKind::Number,
        false,
        Some("0"),
        "Fallback x",
        &[],
    )
    .group("Fallback")
    .order(20),
    ParamSchema::new(
        "y",
        "Default Y",
        ParamKind::Number,
        false,
        Some("0"),
        "Fallback y",
        &[],
    )
    .group("Fallback")
    .order(30),
    ParamSchema::new(
        "angle_deg",
        "Angle",
        ParamKind::Number,
        false,
        Some("0"),
        "Fallback angle",
        &[],
    )
    .group("Fallback")
    .order(40)
    .unit("deg"),
];

fn locate(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let threshold = number(&ctx.node.params, "score_threshold", 0.5);
    let score = 1.0;
    let location = LocationResult {
        found: true,
        x: number(&ctx.node.params, "x", image.width as f64 / 2.0),
        y: number(&ctx.node.params, "y", image.height as f64 / 2.0),
        angle_deg: Some(number(&ctx.node.params, "angle_deg", 0.0)),
        score: Some(score),
        bbox: Some(Rect {
            x: 0.0,
            y: 0.0,
            width: image.width as f64,
            height: image.height as f64,
        }),
        ok: Some(score >= threshold),
        reason: Some("locate node interface ready; algorithm-specific backend can replace this implementation".to_string()),
    };
    Ok(one("location", Value::Location(location)))
}
