use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

use crate::{
    backend::{bridge_image, image, image_path, number, one},
    schema::schema,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema(
        "opencv.edge.canny",
        "OpenCV Canny Edge",
        "OpenCv/Edge",
        &["image"],
        &[],
        &["image"],
        canny,
    ));
}

fn canny(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let output = crate::backend::artifact_path(ctx, "canny.png");
    let low = number(&ctx.node.params, "low", 50.0).to_string();
    let high = number(&ctx.node.params, "high", 150.0).to_string();
    let edges = bridge_image(&[
        "canny",
        input.as_str(),
        output.as_str(),
        low.as_str(),
        high.as_str(),
    ])?;
    Ok(one("image", Value::Image(edges)))
}
