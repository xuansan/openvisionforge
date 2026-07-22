use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, ParamKind, ParamSchema, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema_with_params(
        "storage.local.write",
        "Local Storage Write",
        "Storage",
        &[],
        &["payload", "data", "stats", "image"],
        &["files"],
        LOCAL_WRITE_PARAMS,
        local_write,
    ));
}

const LOCAL_WRITE_PARAMS: &[ParamSchema] = &[ParamSchema::new(
    "output",
    "Output",
    ParamKind::FilePath,
    false,
    None,
    "Output file path",
    &[],
)
.group("Output")
.order(10)
.accept(&[".json", ".png", ".jpg", ".jpeg"])];

fn local_write(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    if ctx.inputs.contains_key("image") {
        let image = crate::util::image(ctx, "image")?;
        let input = crate::util::image_path(&image)?;
        let output = crate::util::output_path(ctx, "output", "storage", "png");
        let path = crate::util::copy_file(input, output)?;
        return Ok(crate::util::one("files", crate::util::file_value(path)));
    }

    let output = crate::util::output_path(ctx, "output", "storage", "json");
    let path = crate::util::write_json(output, &crate::util::payload(ctx))?;
    Ok(crate::util::one("files", crate::util::file_value(path)))
}
