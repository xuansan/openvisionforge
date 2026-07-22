use std::{collections::BTreeMap, fs, path::Path};

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

use crate::{
    backend::{bridge_image, image, image_path, integer, one, output_path, text},
    schema::schema,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema(
        "opencv.image.read",
        "OpenCV Image Read",
        "OpenCv/Image",
        &[],
        &[],
        &["image"],
        image_read,
    ));
    registry.register(schema(
        "opencv.image.write",
        "OpenCV Image Write",
        "OpenCv/Image",
        &["image"],
        &["stats"],
        &["image", "files"],
        image_write,
    ));
    registry.register(schema(
        "opencv.image.clone",
        "OpenCV Image Clone",
        "OpenCv/Image",
        &["image"],
        &[],
        &["image"],
        image_clone,
    ));
    registry.register(schema(
        "opencv.image.resize",
        "OpenCV Image Resize",
        "OpenCv/Image",
        &["image"],
        &[],
        &["image"],
        image_resize,
    ));
    registry.register(schema(
        "opencv.image.crop",
        "OpenCV Image Crop",
        "OpenCv/Image",
        &["image"],
        &[],
        &["image"],
        image_crop,
    ));
}

fn image_read(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let path = text(&ctx.node.params, "path", "");
    if path.is_empty() {
        return crate::backend::fail("opencv.image.read requires params.path");
    }
    let image = bridge_image(&["metadata", path.as_str()])?;
    Ok(one("image", Value::Image(image)))
}

fn image_write(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let output = output_path(ctx, "output", "image.png");
    ensure_parent_dir(&output)?;
    let written = bridge_image(&["write", input.as_str(), output.as_str()])?;
    let result_path = ctx
        .execution
        .records_dir
        .join(format!("{}_opencv_result.json", ctx.node.id));
    fs::write(
        &result_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "run_id": ctx.execution.run_id,
            "output": output,
            "stats": ctx.inputs.get("stats"),
        }))?,
    )?;

    Ok(BTreeMap::from([
        ("image".to_string(), Value::Image(written)),
        (
            "files".to_string(),
            Value::Json(serde_json::json!({
                "output": output,
                "result": result_path,
                "stats": ctx.inputs.get("stats"),
            })),
        ),
    ]))
}

fn ensure_parent_dir(path: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn image_clone(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let output = crate::backend::artifact_path(ctx, "clone.png");
    let cloned = bridge_image(&["write", input.as_str(), output.as_str()])?;
    Ok(one("image", Value::Image(cloned)))
}

fn image_resize(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let output = crate::backend::artifact_path(ctx, "resize.png");
    let width = integer(&ctx.node.params, "width", image.width as i64).to_string();
    let height = integer(&ctx.node.params, "height", image.height as i64).to_string();
    let resized = bridge_image(&[
        "resize",
        input.as_str(),
        output.as_str(),
        width.as_str(),
        height.as_str(),
    ])?;
    Ok(one("image", Value::Image(resized)))
}

fn image_crop(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let output = crate::backend::artifact_path(ctx, "crop.png");
    let x = integer(&ctx.node.params, "x", 0).to_string();
    let y = integer(&ctx.node.params, "y", 0).to_string();
    let width = integer(&ctx.node.params, "width", image.width as i64).to_string();
    let height = integer(&ctx.node.params, "height", image.height as i64).to_string();
    let cropped = bridge_image(&[
        "crop",
        input.as_str(),
        output.as_str(),
        x.as_str(),
        y.as_str(),
        width.as_str(),
        height.as_str(),
    ])?;
    Ok(one("image", Value::Image(cropped)))
}
