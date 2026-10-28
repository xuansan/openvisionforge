use std::collections::BTreeMap;

use vision_core::{
    ImageMask, NodeContext, OperatorRegistry, ParamKind, ParamSchema, Rect, Region, Result, Value,
};

use crate::{
    backend::{artifact_path, bridge_image_and_mask, image, image_path, integer},
    schema::schema_with_params,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema_with_params(
        "opencv.roi.rectangle",
        "Rectangle ROI",
        "OpenCv/ROI",
        &["image"],
        &[],
        &["image", "mask", "regions"],
        RECT_PARAMS,
        rectangle,
    ));
    registry.register(schema_with_params(
        "opencv.roi.circle",
        "Circle ROI",
        "OpenCv/ROI",
        &["image"],
        &[],
        &["image", "mask", "regions"],
        CIRCLE_PARAMS,
        circle,
    ));
    registry.register(schema_with_params(
        "opencv.roi.polygon",
        "Polygon ROI",
        "OpenCv/ROI",
        &["image"],
        &[],
        &["image", "mask", "regions"],
        POLYGON_PARAMS,
        polygon,
    ));
}

const RECT_PARAMS: &[ParamSchema] = &[
    coordinate_param("x", "X", 10),
    coordinate_param("y", "Y", 20),
    ParamSchema::new(
        "width",
        "Width",
        ParamKind::Number,
        true,
        Some("100"),
        "ROI width in pixels",
        &[],
    )
    .group("ROI")
    .order(30)
    .unit("px")
    .range(1.0, 100000.0, 1.0),
    ParamSchema::new(
        "height",
        "Height",
        ParamKind::Number,
        true,
        Some("100"),
        "ROI height in pixels",
        &[],
    )
    .group("ROI")
    .order(40)
    .unit("px")
    .range(1.0, 100000.0, 1.0),
];

const CIRCLE_PARAMS: &[ParamSchema] = &[
    coordinate_param("cx", "Center X", 10),
    coordinate_param("cy", "Center Y", 20),
    ParamSchema::new(
        "radius",
        "Radius",
        ParamKind::Number,
        true,
        Some("100"),
        "Circle radius in pixels",
        &[],
    )
    .group("ROI")
    .order(30)
    .unit("px")
    .range(1.0, 100000.0, 1.0),
];

const POLYGON_PARAMS: &[ParamSchema] = &[ParamSchema::new(
    "points",
    "Points",
    ParamKind::Json,
    true,
    Some("[[0,0],[100,0],[100,100]]"),
    "Polygon vertices as [[x,y],[x,y],...]",
    &[],
)
.group("ROI")
.order(10)];

const fn coordinate_param(key: &'static str, label: &'static str, order: i32) -> ParamSchema {
    ParamSchema::new(
        key,
        label,
        ParamKind::Number,
        true,
        Some("0"),
        "ROI coordinate in pixels",
        &[],
    )
    .group("ROI")
    .order(order)
    .unit("px")
    .range(0.0, 100000.0, 1.0)
}

fn rectangle(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let input = image(ctx, "image")?;
    let source = image_path(&input)?;
    let x = integer(&ctx.node.params, "x", 0);
    let y = integer(&ctx.node.params, "y", 0);
    let width = integer(&ctx.node.params, "width", input.width as i64);
    let height = integer(&ctx.node.params, "height", input.height as i64);
    let output = artifact_path(ctx, "roi_rectangle.png");
    let (image, mask_uri) = bridge_image_and_mask(&[
        "roi_mask",
        source.as_str(),
        output.as_str(),
        "rectangle",
        &x.to_string(),
        &y.to_string(),
        &width.to_string(),
        &height.to_string(),
    ])?;
    roi_out(
        image,
        Rect {
            x: x as f64,
            y: y as f64,
            width: width as f64,
            height: height as f64,
        },
        width.max(0) as f64 * height.max(0) as f64,
        "rectangle",
        mask_uri,
    )
}

fn circle(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let input = image(ctx, "image")?;
    let source = image_path(&input)?;
    let cx = integer(&ctx.node.params, "cx", (input.width / 2) as i64);
    let cy = integer(&ctx.node.params, "cy", (input.height / 2) as i64);
    let radius = integer(&ctx.node.params, "radius", 1);
    let output = artifact_path(ctx, "roi_circle.png");
    let (image, mask_uri) = bridge_image_and_mask(&[
        "roi_mask",
        source.as_str(),
        output.as_str(),
        "circle",
        &cx.to_string(),
        &cy.to_string(),
        &radius.to_string(),
    ])?;
    let diameter = radius.max(0) as f64 * 2.0;
    roi_out(
        image,
        Rect {
            x: cx as f64 - radius as f64,
            y: cy as f64 - radius as f64,
            width: diameter,
            height: diameter,
        },
        std::f64::consts::PI * (radius.max(0) as f64).powi(2),
        "circle",
        mask_uri,
    )
}

fn polygon(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let input = image(ctx, "image")?;
    let source = image_path(&input)?;
    let points = polygon_points(&ctx.node.params)?;
    let output = artifact_path(ctx, "roi_polygon.png");
    let (image, mask_uri) = bridge_image_and_mask(&[
        "roi_mask",
        source.as_str(),
        output.as_str(),
        "polygon",
        points.encoded.as_str(),
    ])?;
    roi_out(image, points.bounds, points.area_px, "polygon", mask_uri)
}

struct PolygonPoints {
    encoded: String,
    bounds: Rect,
    area_px: f64,
}

fn polygon_points(params: &serde_json::Value) -> Result<PolygonPoints> {
    let points = params
        .get("points")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| vision_core::VisionError::Operator {
            node: "opencv.roi.polygon".to_string(),
            message: "points must be a JSON array of [x,y] pairs".to_string(),
        })?;
    if points.len() < 3 {
        return Err(vision_core::VisionError::Operator {
            node: "opencv.roi.polygon".to_string(),
            message: "polygon ROI requires at least three points".to_string(),
        });
    }
    let mut values = Vec::with_capacity(points.len());
    let mut xs = Vec::with_capacity(points.len());
    let mut ys = Vec::with_capacity(points.len());
    let mut vertices = Vec::with_capacity(points.len());
    for point in points {
        let pair = point
            .as_array()
            .ok_or_else(|| vision_core::VisionError::Operator {
                node: "opencv.roi.polygon".to_string(),
                message: "each polygon point must be [x,y]".to_string(),
            })?;
        let x = pair
            .first()
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| vision_core::VisionError::Operator {
                node: "opencv.roi.polygon".to_string(),
                message: "polygon x must be an integer".to_string(),
            })?;
        let y = pair
            .get(1)
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| vision_core::VisionError::Operator {
                node: "opencv.roi.polygon".to_string(),
                message: "polygon y must be an integer".to_string(),
            })?;
        xs.push(x);
        ys.push(y);
        vertices.push((x as f64, y as f64));
        values.push(format!("{x},{y}"));
    }
    let min_x = *xs.iter().min().unwrap_or(&0) as f64;
    let max_x = *xs.iter().max().unwrap_or(&0) as f64;
    let min_y = *ys.iter().min().unwrap_or(&0) as f64;
    let max_y = *ys.iter().max().unwrap_or(&0) as f64;
    let mut twice_area = 0.0;
    for index in 0..vertices.len() {
        let current = vertices[index];
        let next = vertices[(index + 1) % vertices.len()];
        twice_area += current.0 * next.1 - next.0 * current.1;
    }
    Ok(PolygonPoints {
        encoded: values.join(";"),
        bounds: Rect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        },
        area_px: twice_area.abs() * 0.5,
    })
}

fn roi_out(
    image: vision_core::ImageFrame,
    bbox: Rect,
    area_px: f64,
    label: &str,
    mask_uri: Option<String>,
) -> Result<BTreeMap<String, Value>> {
    let bbox = clamp_rect(bbox, image.width, image.height);
    Ok(BTreeMap::from([
        ("image".to_string(), Value::Image(image.clone())),
        (
            "mask".to_string(),
            Value::Mask(ImageMask {
                width: image.width,
                height: image.height,
                source_uri: mask_uri,
            }),
        ),
        (
            "regions".to_string(),
            Value::Regions(vec![Region {
                id: format!("roi_{label}"),
                bbox,
                area_px,
                score: Some(1.0),
                label: Some(label.to_string()),
            }]),
        ),
    ]))
}

fn clamp_rect(rect: Rect, width: u32, height: u32) -> Rect {
    let x = rect.x.clamp(0.0, width as f64);
    let y = rect.y.clamp(0.0, height as f64);
    let right = (rect.x + rect.width).clamp(0.0, width as f64);
    let bottom = (rect.y + rect.height).clamp(0.0, height as f64);
    Rect {
        x,
        y,
        width: (right - x).max(0.0),
        height: (bottom - y).max(0.0),
    }
}
