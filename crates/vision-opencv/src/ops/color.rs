use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

use crate::{
    backend::{image, image_path, one, text},
    schema::schema,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema(
        "opencv.color.to_gray",
        "OpenCV To Gray",
        "OpenCv/Color",
        &["image"],
        &[],
        &["image"],
        to_gray,
    ));
    registry.register(schema(
        "opencv.color.convert",
        "OpenCV Color Convert",
        "OpenCv/Color",
        &["image"],
        &[],
        &["image"],
        convert,
    ));
}

fn to_gray(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let output = crate::backend::artifact_path(ctx, "gray.png");
    let gray = crate::backend::bridge_image(&["gray", input.as_str(), output.as_str()])?;
    Ok(one("image", Value::Image(gray)))
}

fn convert(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let output = crate::backend::artifact_path(ctx, "color_convert.png");
    let mode = text(&ctx.node.params, "mode", "bgr_to_hsv");
    let converted = crate::backend::bridge_image(&[
        "color_convert",
        input.as_str(),
        output.as_str(),
        mode.as_str(),
    ])?;
    Ok(one("image", Value::Image(converted)))
}
