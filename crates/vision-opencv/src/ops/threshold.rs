use std::collections::BTreeMap;

use vision_core::{
    ImageMask, NodeContext, OperatorRegistry, ParamKind, ParamSchema, Result, Value,
};

use crate::{
    backend::{bridge_image, image, image_path, integer, number},
    schema::{schema, schema_with_params},
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema_with_params(
        "opencv.threshold.fixed",
        "OpenCV Fixed Threshold",
        "OpenCv/Threshold",
        &["image"],
        &[],
        &["image", "mask"],
        FIXED_PARAMS,
        threshold_fixed,
    ));
    registry.register(schema(
        "opencv.threshold.otsu",
        "OpenCV Otsu Threshold",
        "OpenCv/Threshold",
        &["image"],
        &[],
        &["image", "mask"],
        threshold_otsu,
    ));
    registry.register(schema(
        "opencv.threshold.triangle",
        "OpenCV Triangle Threshold",
        "OpenCv/Threshold",
        &["image"],
        &[],
        &["image", "mask"],
        threshold_triangle,
    ));
    registry.register(schema_with_params(
        "opencv.threshold.adaptive_mean",
        "OpenCV Adaptive Mean Threshold",
        "OpenCv/Threshold",
        &["image"],
        &[],
        &["image", "mask"],
        ADAPTIVE_PARAMS,
        adaptive_mean,
    ));
    registry.register(schema_with_params(
        "opencv.threshold.adaptive_gaussian",
        "OpenCV Adaptive Gaussian Threshold",
        "OpenCv/Threshold",
        &["image"],
        &[],
        &["image", "mask"],
        ADAPTIVE_PARAMS,
        adaptive_gaussian,
    ));
    registry.register(schema_with_params(
        "opencv.threshold.range",
        "OpenCV Range Threshold",
        "OpenCv/Threshold",
        &["image"],
        &[],
        &["image", "mask"],
        RANGE_PARAMS,
        threshold_range,
    ));
}

const FIXED_PARAMS: &[ParamSchema] = &[ParamSchema::new(
    "threshold",
    "Threshold",
    ParamKind::Number,
    false,
    Some("128"),
    "Binary threshold value",
    &[],
)
.group("Threshold")
.order(10)
.range(0.0, 255.0, 1.0)];

const RANGE_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "low",
        "Low",
        ParamKind::Number,
        false,
        Some("0"),
        "Lower threshold value",
        &[],
    )
    .group("Threshold")
    .order(10)
    .range(0.0, 255.0, 1.0),
    ParamSchema::new(
        "high",
        "High",
        ParamKind::Number,
        false,
        Some("255"),
        "Upper threshold value",
        &[],
    )
    .group("Threshold")
    .order(20)
    .range(0.0, 255.0, 1.0),
];

const ADAPTIVE_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "block_size",
        "Block Size",
        ParamKind::Number,
        false,
        Some("11"),
        "Odd neighborhood size used by adaptive threshold",
        &[],
    )
    .group("Threshold")
    .order(10)
    .range(3.0, 255.0, 2.0),
    ParamSchema::new(
        "c",
        "C",
        ParamKind::Number,
        false,
        Some("2"),
        "Constant subtracted from the local mean",
        &[],
    )
    .group("Threshold")
    .order(20)
    .range(-255.0, 255.0, 0.1),
];

fn threshold_fixed(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let threshold = number(&ctx.node.params, "threshold", 128.0).to_string();
    threshold_command(ctx, &["threshold", threshold.as_str()], "binary.png")
}

fn threshold_otsu(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    threshold_command(ctx, &["threshold_otsu"], "otsu.png")
}

fn threshold_triangle(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    threshold_command(ctx, &["threshold_triangle"], "triangle.png")
}

fn adaptive_mean(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    adaptive_command(ctx, "mean")
}

fn adaptive_gaussian(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    adaptive_command(ctx, "gaussian")
}

fn threshold_range(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let low = number(&ctx.node.params, "low", 0.0).to_string();
    let high = number(&ctx.node.params, "high", 255.0).to_string();
    threshold_command(
        ctx,
        &["threshold_range", low.as_str(), high.as_str()],
        "range.png",
    )
}

fn adaptive_command(ctx: &NodeContext<'_>, method: &str) -> Result<BTreeMap<String, Value>> {
    let block_size = integer(&ctx.node.params, "block_size", 11).to_string();
    let c = number(&ctx.node.params, "c", 2.0).to_string();
    threshold_command(
        ctx,
        &[
            "threshold_adaptive",
            method,
            block_size.as_str(),
            c.as_str(),
        ],
        "adaptive.png",
    )
}

fn threshold_command(
    ctx: &NodeContext<'_>,
    command: &[&str],
    suffix: &str,
) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let output = crate::backend::artifact_path(ctx, suffix);
    let mut args = Vec::with_capacity(command.len() + 2);
    args.push(command[0]);
    args.push(input.as_str());
    args.push(output.as_str());
    args.extend_from_slice(&command[1..]);
    let binary = bridge_image(&args)?;
    Ok(BTreeMap::from([
        ("image".to_string(), Value::Image(binary.clone())),
        (
            "mask".to_string(),
            Value::Mask(ImageMask {
                width: binary.width,
                height: binary.height,
            }),
        ),
    ]))
}
