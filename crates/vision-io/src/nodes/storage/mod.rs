mod local_read;
mod local_write;
mod nas_read;
mod nas_write;
mod oss_upload;
mod rustfs_upload;
mod s3_upload;

use std::path::PathBuf;

use vision_core::{NodeContext, OperatorRegistry, ParamKind, ParamSchema};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    local_write::register(registry);
    local_read::register(registry);
    nas_write::register(registry);
    nas_read::register(registry);
    rustfs_upload::register(registry);
    s3_upload::register(registry);
    oss_upload::register(registry);
}

fn register_upload(
    registry: &mut OperatorRegistry,
    id: &'static str,
    name: &'static str,
    run: vision_core::OperatorFn,
) {
    registry.register(crate::schema::schema_with_params(
        id,
        name,
        "Storage",
        &[],
        &["payload", "data", "stats", "image", "files"],
        &["files"],
        UPLOAD_PARAMS,
        run,
    ));
}

fn upload_manifest(
    ctx: &NodeContext<'_>,
    provider: &str,
) -> vision_core::Result<std::collections::BTreeMap<String, vision_core::Value>> {
    let output = crate::util::output_path(ctx, "manifest", "storage", "upload.json");
    let source = source_path(ctx);
    let value = serde_json::json!({
        "run_id": ctx.execution.run_id,
        "node_id": ctx.node.id,
        "provider": provider,
        "bucket": crate::util::text(&ctx.node.params, "bucket", ""),
        "key": crate::util::text(&ctx.node.params, "key", ""),
        "endpoint": crate::util::text(&ctx.node.params, "endpoint", ""),
        "source": source,
        "payload": crate::util::payload(ctx),
        "inputs": crate::util::input_json(ctx),
        "mode": "manifest",
    });
    let path = crate::util::write_json(output, &value)?;
    Ok(crate::util::one("files", crate::util::file_value(path)))
}

fn source_path(ctx: &NodeContext<'_>) -> Option<PathBuf> {
    if let Ok(image) = crate::util::image(ctx, "image") {
        return image.source_uri.map(PathBuf::from);
    }
    ctx.inputs
        .get("files")
        .and_then(|value| serde_json::to_value(value).ok())
        .and_then(|value| value.get("data").cloned().or(Some(value)))
        .and_then(|value| {
            value
                .get("path")
                .and_then(|v| v.as_str())
                .map(PathBuf::from)
        })
}

const UPLOAD_PARAMS: &[ParamSchema] = &[
    string_param(
        "endpoint",
        "Endpoint",
        true,
        None,
        "Object storage endpoint",
    ),
    string_param("bucket", "Bucket", true, None, "Bucket name"),
    string_param("key", "Key", true, None, "Object key"),
    file_param(
        "manifest",
        "Manifest",
        false,
        None,
        "Manifest JSON path for MVP mode",
    ),
];

const fn string_param(
    key: &'static str,
    label: &'static str,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
) -> ParamSchema {
    ParamSchema::new(
        key,
        label,
        ParamKind::String,
        required,
        default,
        description,
        &[],
    )
    .group("Storage")
}

const fn file_param(
    key: &'static str,
    label: &'static str,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
) -> ParamSchema {
    ParamSchema::new(
        key,
        label,
        ParamKind::FilePath,
        required,
        default,
        description,
        &[],
    )
    .group("Output")
    .accept(&[".json"])
}
