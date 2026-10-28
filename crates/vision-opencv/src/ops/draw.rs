use std::{collections::BTreeMap, fs, path::Path};

use vision_core::{NodeContext, OperatorRegistry, ParamKind, ParamSchema, Result, Value};

use crate::{
    backend::{artifact_path, bridge_image, image, image_path, integer, number, one, text},
    schema::schema_with_params,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema_with_params(
        "opencv.draw.boxes",
        "OpenCV Draw Boxes",
        "OpenCv/Draw",
        &["image"],
        &["detections", "regions"],
        &["image"],
        DRAW_PARAMS,
        boxes,
    ));
    registry.register(schema_with_params(
        "opencv.draw.segmentations",
        "OpenCV Draw Segmentations",
        "OpenCv/Draw",
        &["image"],
        &["masks"],
        &["image"],
        DRAW_PARAMS,
        segmentations,
    ));
    registry.register(schema_with_params(
        "opencv.draw.poses",
        "OpenCV Draw Poses",
        "OpenCv/Draw",
        &["image"],
        &["poses"],
        &["image"],
        DRAW_PARAMS,
        poses,
    ));
    registry.register(schema_with_params(
        "opencv.draw.oriented_boxes",
        "OpenCV Draw Oriented Boxes",
        "OpenCv/Draw",
        &["image"],
        &["oriented_boxes"],
        &["image"],
        DRAW_PARAMS,
        oriented_boxes,
    ));
    registry.register(schema_with_params(
        "opencv.draw.contours",
        "OpenCV Draw Contours",
        "OpenCv/Draw",
        &["image", "contours"],
        &[],
        &["image"],
        DRAW_PARAMS,
        contours,
    ));
    registry.register(schema_with_params(
        "opencv.draw.measurements",
        "OpenCV Draw Measurements",
        "OpenCv/Draw",
        &["image", "measurements"],
        &["contours"],
        &["image"],
        DRAW_PARAMS,
        measurements,
    ));
    registry.register(schema_with_params(
        "opencv.draw.text",
        "OpenCV Draw Text",
        "OpenCv/Draw",
        &["image"],
        &[],
        &["image"],
        TEXT_PARAMS,
        draw_text,
    ));
}

const DRAW_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "thickness",
        "Thickness",
        ParamKind::Number,
        false,
        Some("2"),
        "Drawing line thickness",
        &[],
    )
    .group("Draw")
    .order(10)
    .range(1.0, 20.0, 1.0),
    ParamSchema::new(
        "font_scale",
        "Font Scale",
        ParamKind::Number,
        false,
        Some("0.6"),
        "Text font scale",
        &[],
    )
    .group("Draw")
    .order(20)
    .range(0.1, 5.0, 0.1),
];

const TEXT_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "text",
        "Text",
        ParamKind::String,
        false,
        Some("OK"),
        "Text to draw",
        &[],
    )
    .group("Text")
    .order(10),
    ParamSchema::new(
        "x",
        "X",
        ParamKind::Number,
        false,
        Some("24"),
        "Text x position",
        &[],
    )
    .group("Text")
    .order(20),
    ParamSchema::new(
        "y",
        "Y",
        ParamKind::Number,
        false,
        Some("40"),
        "Text y position",
        &[],
    )
    .group("Text")
    .order(30),
    ParamSchema::new(
        "thickness",
        "Thickness",
        ParamKind::Number,
        false,
        Some("2"),
        "Drawing line thickness",
        &[],
    )
    .group("Draw")
    .order(40),
    ParamSchema::new(
        "font_scale",
        "Font Scale",
        ParamKind::Number,
        false,
        Some("0.8"),
        "Text font scale",
        &[],
    )
    .group("Draw")
    .order(50),
];

fn boxes(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let mut rows = Vec::new();
    if let Some(Value::Detections(detections)) = ctx.inputs.get("detections") {
        for detection in detections {
            rows.push(format!(
                "box\t{}\t{}\t{}\t{}\t{}\t{:.3}",
                detection.bbox.x,
                detection.bbox.y,
                detection.bbox.width,
                detection.bbox.height,
                clean(&detection.label),
                detection.score
            ));
        }
    }
    if let Some(Value::Regions(regions)) = ctx.inputs.get("regions") {
        for region in regions {
            rows.push(format!(
                "box\t{}\t{}\t{}\t{}\t{}\t{}",
                region.bbox.x,
                region.bbox.y,
                region.bbox.width,
                region.bbox.height,
                clean(region.label.as_deref().unwrap_or(&region.id)),
                region
                    .score
                    .map(|score| format!("{score:.3}"))
                    .unwrap_or_default()
            ));
        }
    }
    draw_overlay(ctx, rows, "boxes.png")
}

fn contours(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let mut rows = Vec::new();
    if let Some(Value::Contours(contours)) = ctx.inputs.get("contours") {
        for contour in &contours.contours {
            rows.push(format!(
                "contour\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                contour.bbox.x,
                contour.bbox.y,
                contour.bbox.width,
                contour.bbox.height,
                contour.center_x,
                contour.center_y,
                clean(&contour.id)
            ));
        }
    }
    draw_overlay(ctx, rows, "contours.png")
}

fn segmentations(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let mut rows = Vec::new();
    if let Some(Value::Segmentations(items)) = ctx.inputs.get("masks") {
        for item in items {
            if let Some(mask) = &item.source_uri {
                rows.push(format!("mask\t{}\t{}", mask, clean(&item.label)));
            }
            rows.push(format!(
                "box\t{}\t{}\t{}\t{}\t{}\t{:.3}",
                item.bbox.x,
                item.bbox.y,
                item.bbox.width,
                item.bbox.height,
                clean(&item.label),
                item.score
            ));
        }
    }
    draw_overlay(ctx, rows, "segmentations.png")
}

fn poses(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let mut rows = Vec::new();
    if let Some(Value::Poses(items)) = ctx.inputs.get("poses") {
        for item in items {
            rows.push(format!(
                "box\t{}\t{}\t{}\t{}\t{}\t{:.3}",
                item.bbox.x,
                item.bbox.y,
                item.bbox.width,
                item.bbox.height,
                clean(&item.label),
                item.score
            ));
            let points = item
                .keypoints
                .iter()
                .map(|point| format!("{:.3},{:.3},{:.3}", point.x, point.y, point.score))
                .collect::<Vec<_>>()
                .join(";");
            rows.push(format!("keypoints\t{}", points));
        }
    }
    draw_overlay(ctx, rows, "poses.png")
}

fn oriented_boxes(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let mut rows = Vec::new();
    if let Some(Value::OrientedBoxes(items)) = ctx.inputs.get("oriented_boxes") {
        for item in items {
            rows.push(format!(
                "obb\t{}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}",
                item.cx,
                item.cy,
                item.width,
                item.height,
                item.angle_deg,
                clean(&item.label),
                item.score
            ));
        }
    }
    draw_overlay(ctx, rows, "oriented_boxes.png")
}

fn measurements(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let mut rows = Vec::new();
    let mut x = 24.0;
    let mut y = 36.0;
    if let Some(Value::Contours(contours)) = ctx.inputs.get("contours") {
        if let Some(first) = contours.contours.first() {
            x = first.bbox.x;
            y = (first.bbox.y - 10.0).max(20.0);
        }
    }
    if let Some(Value::Measurements(set)) = ctx.inputs.get("measurements") {
        for (index, item) in set.measurements.iter().enumerate() {
            let value = item
                .value_mm
                .or(item.value_px)
                .map(|value| format!("{value:.3}"))
                .unwrap_or_else(|| "-".to_string());
            let unit = if item.value_mm.is_some() {
                "mm"
            } else {
                item.unit.as_str()
            };
            let status = item.ok.map(|ok| if ok { "OK" } else { "NG" }).unwrap_or("");
            rows.push(format!(
                "text\t{}\t{}\t{}: {} {} {}",
                x,
                y + index as f64 * 24.0,
                clean(&item.name),
                clean(&value),
                clean(unit),
                status
            ));
        }
    }
    draw_overlay(ctx, rows, "measurements.png")
}

fn draw_text(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let row = format!(
        "text\t{}\t{}\t{}",
        number(&ctx.node.params, "x", 24.0),
        number(&ctx.node.params, "y", 40.0),
        clean(&text(&ctx.node.params, "text", "OK"))
    );
    draw_overlay(ctx, vec![row], "text.png")
}

fn draw_overlay(
    ctx: &NodeContext<'_>,
    rows: Vec<String>,
    suffix: &str,
) -> Result<BTreeMap<String, Value>> {
    let image = image(ctx, "image")?;
    let input = image_path(&image)?;
    let overlay = artifact_path(ctx, "overlay.tsv");
    let output = artifact_path(ctx, suffix);
    ensure_parent_dir(&overlay)?;
    ensure_parent_dir(&output)?;
    fs::write(&overlay, rows.join("\n"))?;
    let thickness = integer(&ctx.node.params, "thickness", 2).to_string();
    let font_scale = number(&ctx.node.params, "font_scale", 0.6).to_string();
    let out = bridge_image(&[
        "draw",
        input.as_str(),
        output.as_str(),
        overlay.as_str(),
        thickness.as_str(),
        font_scale.as_str(),
    ])?;
    Ok(one("image", Value::Image(out)))
}

fn ensure_parent_dir(path: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn clean(value: &str) -> String {
    value
        .replace(['\t', '\n', '\r'], " ")
        .chars()
        .take(120)
        .collect()
}
