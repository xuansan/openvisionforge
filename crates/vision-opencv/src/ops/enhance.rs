use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, ParamKind, ParamSchema, Result, Value};

use crate::{
    backend::{bridge_image, image, image_path, integer, number, one},
    schema::schema_with_params,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema_with_params(
        "opencv.enhance.gamma",
        "OpenCV Gamma Correction",
        "OpenCv/Enhance",
        &["image"],
        &[],
        &["image"],
        GAMMA_PARAMS,
        gamma,
    ));
    registry.register(schema_with_params(
        "opencv.enhance.hist_equalize",
        "OpenCV Histogram Equalize",
        "OpenCv/Enhance",
        &["image"],
        &[],
        &["image"],
        &[],
        hist_equalize,
    ));
    registry.register(schema_with_params(
        "opencv.enhance.clahe",
        "OpenCV CLAHE",
        "OpenCv/Enhance",
        &["image"],
        &[],
        &["image"],
        CLAHE_PARAMS,
        clahe,
    ));
    registry.register(schema_with_params(
        "opencv.enhance.contrast_stretch",
        "OpenCV Contrast Stretch",
        "OpenCv/Enhance",
        &["image"],
        &[],
        &["image"],
        CONTRAST_PARAMS,
        contrast_stretch,
    ));
}

const GAMMA_PARAMS: &[ParamSchema] = &[ParamSchema::new(
    "gamma",
    "Gamma",
    ParamKind::Number,
    false,
    Some("1.2"),
    "Gamma value. Larger than 1 darkens, smaller than 1 brightens.",
    &[],
)
.group("Enhance")
.order(10)
.range(0.01, 10.0, 0.01)];

const CLAHE_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "clip_limit",
        "Clip Limit",
        ParamKind::Number,
        false,
        Some("2.0"),
        "CLAHE contrast clipping limit",
        &[],
    )
    .group("Enhance")
    .order(10)
    .range(0.1, 100.0, 0.1),
    ParamSchema::new(
        "tile_grid_size",
        "Tile Grid Size",
        ParamKind::Number,
        false,
        Some("8"),
        "CLAHE tile grid size",
        &[],
    )
    .group("Enhance")
    .order(20)
    .range(1.0, 64.0, 1.0),
];

const CONTRAST_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "low",
        "Low",
        ParamKind::Number,
        false,
        Some("0"),
        "Input value mapped to black",
        &[],
    )
    .group("Enhance")
    .order(10)
    .range(0.0, 255.0, 1.0),
    ParamSchema::new(
        "high",
        "High",
        ParamKind::Number,
        false,
        Some("255"),
        "Input value mapped to white. Use 255 for auto max.",
        &[],
    )
    .group("Enhance")
    .order(20)
    .range(0.0, 255.0, 1.0),
];

fn gamma(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let gamma = number(&ctx.node.params, "gamma", 1.2).to_string();
    enhance_command(ctx, &["gamma", gamma.as_str()], "gamma.png")
}

fn hist_equalize(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    enhance_command(ctx, &["hist_equalize"], "hist_equalize.png")
}

fn clahe(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let clip_limit = number(&ctx.node.params, "clip_limit", 2.0).to_string();
    let tile_grid_size = integer(&ctx.node.params, "tile_grid_size", 8).to_string();
    enhance_command(
        ctx,
        &["clahe", clip_limit.as_str(), tile_grid_size.as_str()],
        "clahe.png",
    )
}

fn contrast_stretch(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let low = number(&ctx.node.params, "low", 0.0).to_string();
    let high = number(&ctx.node.params, "high", 255.0).to_string();
    enhance_command(
        ctx,
        &["contrast_stretch", low.as_str(), high.as_str()],
        "contrast_stretch.png",
    )
}

fn enhance_command(
    ctx: &NodeContext<'_>,
    command: &[&str],
    suffix: &str,
) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let output = crate::backend::artifact_path(ctx, suffix);
    let mut args = Vec::with_capacity(command.len() + 2);
    args.push("enhance");
    args.push(command[0]);
    args.push(input.as_str());
    args.push(output.as_str());
    args.extend_from_slice(&command[1..]);
    let out = bridge_image(&args)?;
    Ok(one("image", Value::Image(out)))
}
