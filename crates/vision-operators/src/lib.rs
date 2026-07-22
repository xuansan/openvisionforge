use std::collections::BTreeMap;

mod recipe;
mod rules;
mod schema;

use vision_core::{
    CaptureInfo, Detection, ImageFrame, ImageMask, NodeContext, OperatorRegistry, OperatorSchema,
    Rect, Region, Result, SignalOutput, TriggerInfo, Value,
};

pub fn register_all(registry: &mut OperatorRegistry) {
    recipe::register(registry);
    rules::register(registry);
    registry.register(schema(
        "source.image.file.mock",
        "Mock Image File",
        "Source",
        &[],
        &[],
        &["image"],
        source_image,
    ));
    registry.register(schema(
        "source.image.folder.mock",
        "Mock Image Folder",
        "Source",
        &[],
        &[],
        &["image", "batch"],
        source_folder,
    ));
    registry.register(schema(
        "trigger.software.mock",
        "Mock Software Trigger",
        "Trigger",
        &[],
        &[],
        &["trigger"],
        software_trigger,
    ));
    registry.register(schema(
        "source.camera.mock",
        "Mock Camera",
        "Source",
        &["trigger"],
        &[],
        &["image", "capture"],
        camera_capture,
    ));
    registry.register(schema(
        "opencv.color.to_gray.mock",
        "Mock To Gray",
        "OpenCv",
        &["image"],
        &[],
        &["image"],
        to_gray,
    ));
    registry.register(schema(
        "opencv.threshold.fixed.mock",
        "Mock Fixed Threshold",
        "OpenCv",
        &["image"],
        &[],
        &["mask"],
        threshold,
    ));
    registry.register(schema(
        "opencv.blob.detect.mock",
        "Mock Blob Detect",
        "OpenCv",
        &["mask"],
        &[],
        &["regions"],
        blob_detect,
    ));
    registry.register(schema(
        "dl.yolo.detect.mock",
        "Mock YOLO Detect",
        "Ai",
        &["image"],
        &[],
        &["detections"],
        yolo_detect,
    ));
    registry.register(schema(
        "judge.area_range",
        "Area Range Judge",
        "Judge",
        &["regions"],
        &[],
        &["passed", "failed_regions"],
        area_range,
    ));
    registry.register(schema(
        "judge.count_range",
        "Count Range Judge",
        "Judge",
        &["detections"],
        &[],
        &["passed", "failed_count"],
        count_range,
    ));
    registry.register(schema(
        "logic.all_pass",
        "All Pass",
        "Logic",
        &["passed"],
        &[],
        &["passed", "status"],
        all_pass,
    ));
    registry.register(schema(
        "record.save_json.mock",
        "Mock Save Json",
        "Record",
        &["status"],
        &[
            "batch",
            "capture",
            "detections",
            "failed_count",
            "failed_regions",
            "signal",
        ],
        &["files"],
        save_json,
    ));
    registry.register(schema(
        "output.signal.mock",
        "Mock Signal Output",
        "Output",
        &["status"],
        &[],
        &["signal"],
        output_signal,
    ));
}

fn schema(
    id: &'static str,
    name: &'static str,
    category: &'static str,
    inputs: &'static [&'static str],
    optional_inputs: &'static [&'static str],
    outputs: &'static [&'static str],
    run: vision_core::OperatorFn,
) -> OperatorSchema {
    OperatorSchema {
        id,
        name,
        category,
        inputs,
        optional_inputs,
        outputs,
        params: &[],
        run,
    }
}

fn source_image(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let p = &ctx.node.params;
    let image = ImageFrame {
        width: number(p, "width", 1280.0) as u32,
        height: number(p, "height", 720.0) as u32,
        channels: number(p, "channels", 3.0) as u8,
        pixel_format: text(p, "pixel_format", "Rgb8"),
        source_uri: p
            .get("source_uri")
            .and_then(|v| v.as_str())
            .map(str::to_string),
    };
    Ok(one("image", Value::Image(image)))
}

fn source_folder(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let p = &ctx.node.params;
    let folder_uri = text(p, "folder_uri", "mock://folder");
    let current_index = number(p, "current_index", 0.0) as u32;
    let image_count = number(p, "image_count", 1.0) as u32;
    Ok(BTreeMap::from([
        (
            "image".to_string(),
            Value::Image(ImageFrame {
                width: number(p, "width", 1280.0) as u32,
                height: number(p, "height", 720.0) as u32,
                channels: number(p, "channels", 3.0) as u8,
                pixel_format: text(p, "pixel_format", "Rgb8"),
                source_uri: Some(format!("{folder_uri}/image_{current_index}.png")),
            }),
        ),
        (
            "batch".to_string(),
            Value::Json(serde_json::json!({
                "folder_uri": folder_uri,
                "current_index": current_index,
                "image_count": image_count,
            })),
        ),
    ]))
}

fn software_trigger(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let p = &ctx.node.params;
    Ok(one(
        "trigger",
        Value::Trigger(TriggerInfo {
            trigger_id: text(p, "trigger_id", "trigger_mock_001"),
            trigger_type: text(p, "trigger_type", "software"),
            source: text(p, "source", "cli"),
        }),
    ))
}

fn camera_capture(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let p = &ctx.node.params;
    let trigger_id = match ctx.inputs.get("trigger") {
        Some(Value::Trigger(trigger)) => Some(trigger.trigger_id.clone()),
        _ => None,
    };
    let width = number(p, "width", 1280.0) as u32;
    let height = number(p, "height", 720.0) as u32;
    let pixel_format = text(p, "pixel_format", "Rgb8");
    let camera_id = text(p, "camera_id", "camera_mock_001");

    Ok(BTreeMap::from([
        (
            "image".to_string(),
            Value::Image(ImageFrame {
                width,
                height,
                channels: number(p, "channels", 3.0) as u8,
                pixel_format: pixel_format.clone(),
                source_uri: Some(format!("camera://{camera_id}")),
            }),
        ),
        (
            "capture".to_string(),
            Value::Capture(CaptureInfo {
                camera_id,
                trigger_id,
                width,
                height,
                pixel_format,
            }),
        ),
    ]))
}

fn to_gray(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let mut image = image(ctx, "image")?;
    image.channels = 1;
    image.pixel_format = "Gray8".to_string();
    Ok(one("image", Value::Image(image)))
}

fn threshold(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    Ok(one(
        "mask",
        Value::Mask(ImageMask {
            width: image.width,
            height: image.height,
        }),
    ))
}

fn blob_detect(_ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    Ok(one(
        "regions",
        Value::Regions(vec![
            Region {
                id: "r1".to_string(),
                bbox: Rect {
                    x: 20.0,
                    y: 30.0,
                    width: 10.0,
                    height: 8.0,
                },
                area_px: 80.0,
                score: Some(0.9),
                label: Some("spot".to_string()),
            },
            Region {
                id: "r2".to_string(),
                bbox: Rect {
                    x: 80.0,
                    y: 50.0,
                    width: 18.0,
                    height: 12.0,
                },
                area_px: 216.0,
                score: Some(0.95),
                label: Some("spot".to_string()),
            },
        ]),
    ))
}

fn yolo_detect(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let p = &ctx.node.params;
    let threshold = number(p, "score_threshold", 0.5);
    let classes = p
        .get("classes")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let detections = vec![
        Detection {
            label: "part".to_string(),
            score: 0.93,
            bbox: Rect {
                x: 100.0,
                y: 120.0,
                width: 80.0,
                height: 60.0,
            },
        },
        Detection {
            label: "defect".to_string(),
            score: 0.71,
            bbox: Rect {
                x: 260.0,
                y: 180.0,
                width: 24.0,
                height: 18.0,
            },
        },
    ]
    .into_iter()
    .filter(|detection| detection.score >= threshold)
    .filter(|detection| classes.is_empty() || classes.contains(&detection.label))
    .collect::<Vec<_>>();

    Ok(one("detections", Value::Detections(detections)))
}

fn area_range(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let max_area = number(&ctx.node.params, "max_area", 100.0);
    let regions = match ctx.inputs.get("regions") {
        Some(Value::Regions(regions)) => regions.clone(),
        _ => Vec::new(),
    };
    let failed = regions
        .into_iter()
        .filter(|region| region.area_px > max_area)
        .collect::<Vec<_>>();
    let mut out = BTreeMap::new();
    out.insert("passed".to_string(), Value::Bool(failed.is_empty()));
    out.insert("failed_regions".to_string(), Value::Regions(failed));
    Ok(out)
}

fn count_range(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let p = &ctx.node.params;
    let label = p.get("label").and_then(|v| v.as_str());
    let min = number(p, "min", 0.0) as usize;
    let max = number(p, "max", usize::MAX as f64) as usize;
    let count = match ctx.inputs.get("detections") {
        Some(Value::Detections(detections)) => detections
            .iter()
            .filter(|detection| label.is_none_or(|label| detection.label == label))
            .count(),
        _ => 0,
    };
    let passed = count >= min && count <= max;
    let mut out = BTreeMap::new();
    out.insert("passed".to_string(), Value::Bool(passed));
    out.insert(
        "failed_count".to_string(),
        Value::Json(serde_json::json!({
            "label": label,
            "count": count,
            "min": min,
            "max": max,
            "passed": passed,
        })),
    );
    Ok(out)
}

fn all_pass(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let passed = matches!(ctx.inputs.get("passed"), Some(Value::Bool(true)));
    let mut out = BTreeMap::new();
    out.insert("passed".to_string(), Value::Bool(passed));
    out.insert(
        "status".to_string(),
        Value::Text(if passed { "OK" } else { "NG" }.to_string()),
    );
    Ok(out)
}

fn save_json(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let status = match ctx.inputs.get("status") {
        Some(Value::Text(status)) => status.clone(),
        _ => "UNKNOWN".to_string(),
    };
    let result_path = ctx.execution.records_dir.join("result.json");
    std::fs::write(
        &result_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "run_id": ctx.execution.run_id,
            "status": status,
            "inputs": ctx.inputs,
        }))?,
    )?;
    Ok(one(
        "files",
        Value::Json(serde_json::json!({
            "run_id": ctx.execution.run_id,
            "status": status,
            "records_dir": ctx.execution.records_dir,
            "result": result_path,
        })),
    ))
}

fn output_signal(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let status = match ctx.inputs.get("status") {
        Some(Value::Text(status)) => status.clone(),
        _ => "UNKNOWN".to_string(),
    };
    Ok(one(
        "signal",
        Value::Signal(SignalOutput {
            channel: text(&ctx.node.params, "channel", "DO0"),
            value: status,
        }),
    ))
}

fn image(ctx: &NodeContext<'_>, port: &str) -> Result<ImageFrame> {
    match ctx.inputs.get(port) {
        Some(Value::Image(image)) => Ok(image.clone()),
        _ => Ok(ImageFrame {
            width: 0,
            height: 0,
            channels: 0,
            pixel_format: "Unknown".to_string(),
            source_uri: None,
        }),
    }
}

fn one(port: &str, value: Value) -> BTreeMap<String, Value> {
    BTreeMap::from([(port.to_string(), value)])
}

fn number(value: &serde_json::Value, key: &str, default: f64) -> f64 {
    value.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
}

fn text(value: &serde_json::Value, key: &str, default: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}
