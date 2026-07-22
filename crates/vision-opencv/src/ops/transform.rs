use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

use crate::{
    backend::{bridge_image, image, image_path, one, text},
    schema::schema,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema(
        "opencv.transform.rotate",
        "OpenCV Rotate",
        "OpenCv/Transform",
        &["image"],
        &[],
        &["image"],
        rotate,
    ));
    registry.register(schema(
        "opencv.transform.flip",
        "OpenCV Flip",
        "OpenCv/Transform",
        &["image"],
        &[],
        &["image"],
        flip,
    ));
}

fn rotate(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let output = crate::backend::artifact_path(ctx, "rotate.png");
    let direction = text(&ctx.node.params, "direction", "right");
    let rotated = bridge_image(&[
        "rotate",
        input.as_str(),
        output.as_str(),
        direction.as_str(),
    ])?;
    Ok(one("image", Value::Image(rotated)))
}

fn flip(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let output = crate::backend::artifact_path(ctx, "flip.png");
    let mode = text(&ctx.node.params, "mode", "horizontal");
    let flipped = bridge_image(&["flip", input.as_str(), output.as_str(), mode.as_str()])?;
    Ok(one("image", Value::Image(flipped)))
}
