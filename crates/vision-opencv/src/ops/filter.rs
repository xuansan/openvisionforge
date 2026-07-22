use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

use crate::{
    backend::{bridge_image, image, image_path, integer, number, one},
    schema::schema,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    register_filter(registry, "opencv.filter.blur", "OpenCV Blur", blur);
    register_filter(
        registry,
        "opencv.filter.gaussian",
        "OpenCV Gaussian Blur",
        gaussian,
    );
    register_filter(
        registry,
        "opencv.filter.median",
        "OpenCV Median Blur",
        median,
    );
    register_filter(registry, "opencv.filter.sharpen", "OpenCV Sharpen", sharpen);
}

fn register_filter(
    registry: &mut OperatorRegistry,
    id: &'static str,
    name: &'static str,
    run: vision_core::OperatorFn,
) {
    registry.register(schema(
        id,
        name,
        "OpenCv/Filter",
        &["image"],
        &[],
        &["image"],
        run,
    ));
}

fn blur(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let kernel = integer(&ctx.node.params, "kernel", 3).to_string();
    filter_command(ctx, &["blur", kernel.as_str()], "blur.png")
}

fn gaussian(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let kernel = integer(&ctx.node.params, "kernel", 5).to_string();
    let sigma = number(&ctx.node.params, "sigma", 0.0).to_string();
    filter_command(
        ctx,
        &["gaussian", kernel.as_str(), sigma.as_str()],
        "gaussian.png",
    )
}

fn median(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let kernel = integer(&ctx.node.params, "kernel", 3).to_string();
    filter_command(ctx, &["median", kernel.as_str()], "median.png")
}

fn sharpen(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    filter_command(ctx, &["sharpen"], "sharpen.png")
}

fn filter_command(
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
    let out = bridge_image(&args)?;
    Ok(one("image", Value::Image(out)))
}
