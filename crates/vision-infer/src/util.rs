use vision_core::{ImageFrame, NodeContext, Result, Value, VisionError};

pub(crate) fn image(ctx: &NodeContext<'_>, port: &str) -> Result<ImageFrame> {
    match ctx.inputs.get(port) {
        Some(Value::Image(image)) => Ok(image.clone()),
        _ => Err(VisionError::Operator {
            node: ctx.node.id.clone(),
            message: format!("expected image input on port {port}"),
        }),
    }
}

pub(crate) fn text(value: &serde_json::Value, key: &str, default: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

pub(crate) fn number(value: &serde_json::Value, key: &str, default: f64) -> f64 {
    value.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
}

pub(crate) fn integer(value: &serde_json::Value, key: &str, default: i64) -> i64 {
    value.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
}

pub(crate) fn string_array(value: &serde_json::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn f32_triplet(value: &serde_json::Value, key: &str, default: [f32; 3]) -> [f32; 3] {
    let Some(items) = value.get(key).and_then(|v| v.as_array()) else {
        return default;
    };
    let mut result = default;
    for (index, item) in items.iter().take(3).enumerate() {
        if let Some(number) = item.as_f64() {
            result[index] = number as f32;
        }
    }
    result
}

pub(crate) fn boolean(value: &serde_json::Value, key: &str, default: bool) -> bool {
    value.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}
