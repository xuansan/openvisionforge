use std::{collections::BTreeMap, path::PathBuf};

use vision_core::{NodeContext, OperatorRegistry, ParamKind, ParamSchema, Result, Value};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(crate::schema::schema_with_params(
        "storage.nas.write",
        "NAS Storage Write",
        "Storage",
        &[],
        &["payload", "data", "stats", "image"],
        &["files"],
        NAS_WRITE_PARAMS,
        nas_write,
    ));
}

const NAS_WRITE_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "path",
        "Path",
        ParamKind::FilePath,
        false,
        None,
        "Full NAS or network share output path",
        &[],
    )
    .group("Output")
    .order(10)
    .accept(&[".json", ".png", ".jpg", ".jpeg"]),
    ParamSchema::new(
        "root",
        "Root",
        ParamKind::FolderPath,
        false,
        Some("nas"),
        "NAS root folder or mounted network share",
        &[],
    )
    .group("Storage")
    .order(20),
    ParamSchema::new(
        "relative_path",
        "Relative Path",
        ParamKind::FilePath,
        false,
        None,
        "Path relative to root",
        &[],
    )
    .group("Output")
    .order(30)
    .accept(&[".json", ".png", ".jpg", ".jpeg"]),
];

fn nas_write(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let output = nas_output_path(ctx);
    if ctx.inputs.contains_key("image") {
        let image = crate::util::image(ctx, "image")?;
        let input = crate::util::image_path(&image)?;
        let path = crate::util::copy_file(input, output)?;
        return Ok(crate::util::one("files", crate::util::file_value(path)));
    }

    let path = crate::util::write_json(output, &crate::util::payload(ctx))?;
    Ok(crate::util::one("files", crate::util::file_value(path)))
}

fn nas_output_path(ctx: &NodeContext<'_>) -> PathBuf {
    if let Some(path) = ctx.node.params.get("path").and_then(|v| v.as_str()) {
        return PathBuf::from(path);
    }

    let root = crate::util::text(&ctx.node.params, "root", "nas");
    let relative = crate::util::text(
        &ctx.node.params,
        "relative_path",
        if ctx.inputs.contains_key("image") {
            "vision-output.png"
        } else {
            "vision-output.json"
        },
    );
    PathBuf::from(root).join(relative)
}
