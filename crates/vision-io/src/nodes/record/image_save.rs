use std::collections::BTreeMap;

use vision_core::{NodeContext, OperatorRegistry, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema(
        "record.image.save",
        "Save Record Image",
        "Record",
        &["image"],
        &[],
        &["files"],
        image_save,
    ));
}

fn image_save(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = crate::util::image(ctx, "image")?;
    let input = crate::util::image_path(&image)?;
    let output = crate::util::output_path(ctx, "output", "record", "png");
    let path = crate::util::copy_file(input, output)?;
    Ok(crate::util::one("files", crate::util::file_value(path)))
}
