use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use vision_core::{ImageFrame, NodeContext, Result, Value, VisionError};

pub(crate) fn one(port: &str, value: Value) -> BTreeMap<String, Value> {
    BTreeMap::from([(port.to_string(), value)])
}

pub(crate) fn text(value: &serde_json::Value, key: &str, default: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

pub(crate) fn input_json(ctx: &NodeContext<'_>) -> serde_json::Value {
    serde_json::to_value(&ctx.inputs).unwrap_or_else(|_| serde_json::json!({}))
}

pub(crate) fn payload(ctx: &NodeContext<'_>) -> serde_json::Value {
    ctx.inputs
        .get("payload")
        .or_else(|| ctx.inputs.get("data"))
        .or_else(|| ctx.inputs.get("stats"))
        .cloned()
        .map(|value| serde_json::to_value(value).unwrap_or_else(|_| serde_json::json!(null)))
        .unwrap_or_else(|| input_json(ctx))
}

pub(crate) fn image(ctx: &NodeContext<'_>, port: &str) -> Result<ImageFrame> {
    match ctx.inputs.get(port) {
        Some(Value::Image(image)) => Ok(image.clone()),
        _ => fail(format!("expected image input on port {port}")),
    }
}

pub(crate) fn image_path(image: &ImageFrame) -> Result<String> {
    image
        .source_uri
        .clone()
        .filter(|path| !path.is_empty())
        .ok_or_else(|| VisionError::Operator {
            node: "io".to_string(),
            message: "image source_uri is required".to_string(),
        })
}

pub(crate) fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub(crate) fn copy_file(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<PathBuf> {
    let output = output.as_ref();
    ensure_parent(output)?;
    fs::copy(input, output)?;
    Ok(output.to_path_buf())
}

pub(crate) fn write_json(path: impl AsRef<Path>, value: &serde_json::Value) -> Result<PathBuf> {
    let path = path.as_ref();
    ensure_parent(path)?;
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(path.to_path_buf())
}

pub(crate) fn default_record_path(ctx: &NodeContext<'_>, folder: &str, extension: &str) -> PathBuf {
    ctx.execution
        .records_dir
        .join(folder)
        .join(format!("{}.{}", ctx.node.id, extension))
}

pub(crate) fn output_path(
    ctx: &NodeContext<'_>,
    key: &str,
    default_folder: &str,
    default_ext: &str,
) -> PathBuf {
    ctx.node
        .params
        .get(key)
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| default_record_path(ctx, default_folder, default_ext))
}

pub(crate) fn file_value(path: impl AsRef<Path>) -> Value {
    Value::Json(serde_json::json!({
        "path": path.as_ref(),
    }))
}

pub(crate) fn fail<T>(message: impl Into<String>) -> Result<T> {
    Err(VisionError::Operator {
        node: "io".to_string(),
        message: message.into(),
    })
}
