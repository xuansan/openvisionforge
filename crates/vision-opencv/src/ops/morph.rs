use std::collections::BTreeMap;

use vision_core::{ImageMask, NodeContext, OperatorRegistry, Result, Value};

use crate::{
    backend::{bridge_image, image, image_path, integer},
    schema::schema,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    register_morph(registry, "opencv.morph.erode", "OpenCV Erode", erode);
    register_morph(registry, "opencv.morph.dilate", "OpenCV Dilate", dilate);
    register_morph(registry, "opencv.morph.open", "OpenCV Morph Open", open);
    register_morph(registry, "opencv.morph.close", "OpenCV Morph Close", close);
    register_morph(
        registry,
        "opencv.morph.gradient",
        "OpenCV Morph Gradient",
        gradient,
    );
    register_morph(registry, "opencv.morph.tophat", "OpenCV Top Hat", tophat);
    register_morph(
        registry,
        "opencv.morph.blackhat",
        "OpenCV Black Hat",
        blackhat,
    );
}

fn register_morph(
    registry: &mut OperatorRegistry,
    id: &'static str,
    name: &'static str,
    run: vision_core::OperatorFn,
) {
    registry.register(schema(
        id,
        name,
        "OpenCv/Morphology",
        &["image"],
        &[],
        &["image", "mask"],
        run,
    ));
}

fn erode(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    morph_command(ctx, "erode")
}

fn dilate(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    morph_command(ctx, "dilate")
}

fn open(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    morph_command(ctx, "open")
}

fn close(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    morph_command(ctx, "close")
}

fn gradient(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    morph_command(ctx, "gradient")
}

fn tophat(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    morph_command(ctx, "tophat")
}

fn blackhat(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    morph_command(ctx, "blackhat")
}

fn morph_command(ctx: &NodeContext<'_>, op: &str) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let output = crate::backend::artifact_path(ctx, "morph.png");
    let kernel = integer(&ctx.node.params, "kernel", 3).to_string();
    let iterations = integer(&ctx.node.params, "iterations", 1).to_string();
    let out = bridge_image(&[
        "morph",
        input.as_str(),
        output.as_str(),
        op,
        kernel.as_str(),
        iterations.as_str(),
    ])?;
    Ok(BTreeMap::from([
        ("image".to_string(), Value::Image(out.clone())),
        (
            "mask".to_string(),
            Value::Mask(ImageMask {
                width: out.width,
                height: out.height,
                source_uri: out.source_uri.clone(),
            }),
        ),
    ]))
}
