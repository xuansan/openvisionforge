use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema(
        "display.image",
        "Display Image",
        "Display",
        &["image"],
        &["data", "stats", "detections", "measurements", "final"],
        &["image", "files"],
        display_image,
    ));
}

fn display_image(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = crate::util::image(ctx, "image")?;
    let input = crate::util::image_path(&image)?;
    let image_output = super::display_path(ctx, "png");
    let image_path = crate::util::copy_file(input, &image_output)?;
    let manifest = super::display_path(ctx, "json");
    let value = serde_json::json!({
        "kind": "display.image",
        "run_id": ctx.execution.run_id,
        "node_id": ctx.node.id,
        "image": image,
        "preview_image": image_path,
        "inputs": crate::util::input_json(ctx),
    });
    let path = crate::util::write_json(manifest, &value)?;

    Ok(BTreeMap::from([
        ("image".to_string(), Value::Image(image)),
        (
            "files".to_string(),
            Value::Json(serde_json::json!({
                "manifest": path,
                "preview_image": image_path,
            })),
        ),
    ]))
}
