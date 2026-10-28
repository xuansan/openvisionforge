use std::{collections::BTreeMap, fs};

use vision_core::{
    CameraCalibration, NodeContext, OperatorRegistry, ParamKind, ParamSchema, Result, Value,
    VisionError,
};

use crate::{
    backend::{artifact_path, bridge_image, bridge_json, image, image_path},
    schema::schema_with_params,
};

pub(crate) fn register(registry: &mut OperatorRegistry) {
    registry.register(schema_with_params(
        "calibration.board.chessboard_detect",
        "Detect Chessboard",
        "Calibration/Board",
        &["image"],
        &[],
        &["calibration", "stats"],
        BOARD_PARAMS,
        detect_chessboard,
    ));
    registry.register(schema_with_params(
        "calibration.board.circle_grid_detect",
        "Detect Circle Grid",
        "Calibration/Board",
        &["image"],
        &[],
        &["calibration", "stats"],
        BOARD_PARAMS,
        detect_circle_grid,
    ));
    registry.register(schema_with_params(
        "calibration.camera.intrinsics",
        "Calibrate Camera Intrinsics",
        "Calibration/Camera",
        &[],
        &[],
        &["camera_calibration", "stats"],
        INTRINSICS_PARAMS,
        calibrate_intrinsics,
    ));
    registry.register(schema_with_params(
        "calibration.camera.save",
        "Save Camera Calibration",
        "Calibration/Camera",
        &["camera_calibration"],
        &[],
        &["camera_calibration", "record"],
        SAVE_PARAMS,
        save_calibration,
    ));
    registry.register(schema_with_params(
        "calibration.camera.load",
        "Load Camera Calibration",
        "Calibration/Camera",
        &[],
        &[],
        &["camera_calibration", "stats"],
        LOAD_PARAMS,
        load_calibration,
    ));
    registry.register(schema_with_params(
        "calibration.image.undistort",
        "Undistort Image",
        "Calibration/Image",
        &["image", "camera_calibration"],
        &[],
        &["image", "stats"],
        &[],
        undistort,
    ));
}

const BOARD_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "columns",
        "Columns",
        ParamKind::Number,
        true,
        Some("9"),
        "Number of inner points per row",
        &[],
    )
    .group("Board")
    .order(10)
    .range(2.0, 100.0, 1.0),
    ParamSchema::new(
        "rows",
        "Rows",
        ParamKind::Number,
        true,
        Some("6"),
        "Number of inner points per column",
        &[],
    )
    .group("Board")
    .order(20)
    .range(2.0, 100.0, 1.0),
];

const INTRINSICS_PARAMS: &[ParamSchema] = &[
    ParamSchema::new(
        "folder",
        "Calibration Images",
        ParamKind::FolderPath,
        true,
        None,
        "Folder containing calibration images",
        &[],
    )
    .group("Input")
    .order(10),
    ParamSchema::new(
        "pattern",
        "Board Pattern",
        ParamKind::Enum,
        true,
        Some("chessboard"),
        "Calibration board pattern",
        &["chessboard", "circle_grid"],
    )
    .group("Board")
    .order(20),
    ParamSchema::new(
        "columns",
        "Columns",
        ParamKind::Number,
        true,
        Some("9"),
        "Number of inner points per row",
        &[],
    )
    .group("Board")
    .order(30)
    .range(2.0, 100.0, 1.0),
    ParamSchema::new(
        "rows",
        "Rows",
        ParamKind::Number,
        true,
        Some("6"),
        "Number of inner points per column",
        &[],
    )
    .group("Board")
    .order(40)
    .range(2.0, 100.0, 1.0),
    ParamSchema::new(
        "spacing_mm",
        "Point Spacing",
        ParamKind::Number,
        true,
        Some("20"),
        "Chessboard square size or circle center spacing",
        &[],
    )
    .group("Board")
    .order(50)
    .unit("mm")
    .range(0.000001, 100000.0, 0.001),
    ParamSchema::new(
        "min_images",
        "Minimum Valid Images",
        ParamKind::Number,
        true,
        Some("10"),
        "Minimum successfully detected views",
        &[],
    )
    .group("Quality")
    .order(60)
    .range(3.0, 10000.0, 1.0),
    ParamSchema::new(
        "max_reprojection_error",
        "Maximum Mean Error",
        ParamKind::Number,
        true,
        Some("1"),
        "Reject calibration above this mean pixel error; zero disables the limit",
        &[],
    )
    .group("Quality")
    .order(70)
    .unit("px")
    .range(0.0, 1000.0, 0.01),
    ParamSchema::new(
        "recursive",
        "Recursive",
        ParamKind::Boolean,
        false,
        Some("false"),
        "Search image files in subfolders",
        &[],
    )
    .group("Input")
    .order(80),
];

const SAVE_PARAMS: &[ParamSchema] = &[ParamSchema::new(
    "path",
    "Calibration File",
    ParamKind::FilePath,
    true,
    None,
    "Destination JSON file",
    &[],
)
.group("File")
.order(10)
.accept(&["json"])];

const LOAD_PARAMS: &[ParamSchema] = &[ParamSchema::new(
    "path",
    "Calibration File",
    ParamKind::FilePath,
    true,
    None,
    "Camera calibration JSON file",
    &[],
)
.group("File")
.order(10)
.accept(&["json"])];

fn detect_chessboard(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    detect_board(ctx, "chessboard")
}

fn detect_circle_grid(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    detect_board(ctx, "circle_grid")
}

fn detect_board(ctx: &NodeContext<'_>, pattern: &str) -> Result<BTreeMap<String, Value>> {
    let input = image(ctx, "image")?;
    let source = image_path(&input)?;
    let columns = positive_integer(ctx, "columns")?;
    let rows = positive_integer(ctx, "rows")?;
    let result = bridge_json(&[
        "calibration_detect",
        source.as_str(),
        pattern,
        &columns.to_string(),
        &rows.to_string(),
    ])?;
    let found = result
        .get("found")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if !found {
        return Err(operator_error(
            ctx,
            format!("{pattern} points were not detected in {source}"),
        ));
    }
    Ok(BTreeMap::from([
        ("calibration".to_string(), Value::Json(result.clone())),
        (
            "stats".to_string(),
            Value::Json(serde_json::json!({
                "pattern": pattern,
                "found": found,
                "expected_points": columns * rows,
                "detected_points": result.get("point_count").cloned().unwrap_or_default(),
            })),
        ),
    ]))
}

fn calibrate_intrinsics(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let folder = required_text(ctx, "folder")?;
    let pattern = required_text(ctx, "pattern")?;
    let columns = positive_integer(ctx, "columns")?;
    let rows = positive_integer(ctx, "rows")?;
    let spacing = positive_number(ctx, "spacing_mm")?;
    let min_images = positive_integer(ctx, "min_images")?;
    let max_error = number(ctx, "max_reprojection_error").unwrap_or(1.0);
    let recursive = ctx
        .node
        .params
        .get("recursive")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let json = bridge_json(&[
        "calibrate_folder",
        folder.as_str(),
        pattern.as_str(),
        &columns.to_string(),
        &rows.to_string(),
        &spacing.to_string(),
        &min_images.to_string(),
        &max_error.to_string(),
        if recursive { "true" } else { "false" },
    ])?;
    let calibration: CameraCalibration =
        serde_json::from_value(json.clone()).map_err(|err| VisionError::Operator {
            node: ctx.node.id.clone(),
            message: format!("invalid calibration result: {err}"),
        })?;
    validate_calibration(ctx, &calibration)?;
    Ok(BTreeMap::from([
        (
            "camera_calibration".to_string(),
            Value::CameraCalibration(calibration),
        ),
        ("stats".to_string(), Value::Json(json)),
    ]))
}

fn save_calibration(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let calibration = calibration_input(ctx)?;
    let path = required_text(ctx, "path")?;
    if let Some(parent) = std::path::Path::new(&path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_vec_pretty(&calibration)?)?;
    Ok(BTreeMap::from([
        (
            "camera_calibration".to_string(),
            Value::CameraCalibration(calibration),
        ),
        (
            "record".to_string(),
            Value::Json(serde_json::json!({ "path": path })),
        ),
    ]))
}

fn load_calibration(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let path = required_text(ctx, "path")?;
    let calibration: CameraCalibration =
        serde_json::from_slice(&fs::read(&path)?).map_err(|err| VisionError::Operator {
            node: ctx.node.id.clone(),
            message: format!("invalid camera calibration file {path}: {err}"),
        })?;
    validate_calibration(ctx, &calibration)?;
    let stats = serde_json::to_value(&calibration)?;
    Ok(BTreeMap::from([
        (
            "camera_calibration".to_string(),
            Value::CameraCalibration(calibration),
        ),
        ("stats".to_string(), Value::Json(stats)),
    ]))
}

fn undistort(ctx: &NodeContext<'_>) -> Result<BTreeMap<String, Value>> {
    let input = image(ctx, "image")?;
    let source = image_path(&input)?;
    let calibration = calibration_input(ctx)?;
    validate_calibration(ctx, &calibration)?;
    let matrix = csv(&calibration.camera_matrix);
    let distortion = csv(&calibration.distortion_coefficients);
    let output = artifact_path(ctx, "undistorted.png");
    let image = bridge_image(&[
        "undistort",
        source.as_str(),
        output.as_str(),
        matrix.as_str(),
        distortion.as_str(),
        &calibration.image_width.to_string(),
        &calibration.image_height.to_string(),
    ])?;
    Ok(BTreeMap::from([
        ("image".to_string(), Value::Image(image)),
        (
            "stats".to_string(),
            Value::Json(serde_json::json!({
                "source": source, "output": output, "calibration_error": calibration.mean_reprojection_error,
            })),
        ),
    ]))
}

fn calibration_input(ctx: &NodeContext<'_>) -> Result<CameraCalibration> {
    match ctx.inputs.get("camera_calibration") {
        Some(Value::CameraCalibration(value)) => Ok(value.clone()),
        _ => Err(operator_error(ctx, "camera_calibration input is required")),
    }
}

fn validate_calibration(ctx: &NodeContext<'_>, value: &CameraCalibration) -> Result<()> {
    if value.image_width == 0 || value.image_height == 0 || value.views_used == 0 {
        return Err(operator_error(
            ctx,
            "camera calibration has invalid image size or view count",
        ));
    }
    if !value.camera_matrix.iter().all(|value| value.is_finite())
        || value.camera_matrix[0] <= 0.0
        || value.camera_matrix[4] <= 0.0
        || value.distortion_coefficients.len() < 4
        || !value
            .distortion_coefficients
            .iter()
            .all(|value| value.is_finite())
    {
        return Err(operator_error(
            ctx,
            "camera calibration contains invalid matrix values",
        ));
    }
    Ok(())
}

fn required_text(ctx: &NodeContext<'_>, key: &str) -> Result<String> {
    ctx.node
        .params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| operator_error(ctx, format!("parameter {key} is required")))
}

fn number(ctx: &NodeContext<'_>, key: &str) -> Option<f64> {
    ctx.node.params.get(key).and_then(serde_json::Value::as_f64)
}

fn positive_number(ctx: &NodeContext<'_>, key: &str) -> Result<f64> {
    number(ctx, key)
        .filter(|value| *value > 0.0)
        .ok_or_else(|| operator_error(ctx, format!("parameter {key} must be positive")))
}

fn positive_integer(ctx: &NodeContext<'_>, key: &str) -> Result<i64> {
    number(ctx, key)
        .filter(|value| *value > 0.0 && value.fract() == 0.0)
        .map(|value| value as i64)
        .ok_or_else(|| operator_error(ctx, format!("parameter {key} must be a positive integer")))
}

fn operator_error(ctx: &NodeContext<'_>, message: impl Into<String>) -> VisionError {
    VisionError::Operator {
        node: ctx.node.id.clone(),
        message: message.into(),
    }
}

fn csv(values: &[f64]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use vision_core::{CalibrationBoard, ExecutionContext, WorkflowNode};

    fn context<'a>(
        node: &'a WorkflowNode,
        inputs: BTreeMap<String, Value>,
        execution: &'a ExecutionContext,
    ) -> NodeContext<'a> {
        NodeContext {
            node,
            inputs,
            execution,
        }
    }

    fn sample_calibration() -> CameraCalibration {
        CameraCalibration {
            image_width: 640,
            image_height: 480,
            camera_matrix: [500.0, 0.0, 320.0, 0.0, 500.0, 240.0, 0.0, 0.0, 1.0],
            distortion_coefficients: vec![0.1, -0.2, 0.0, 0.0, 0.0],
            rms_error: 0.2,
            mean_reprojection_error: 0.1,
            views_used: 12,
            board: CalibrationBoard {
                pattern: "chessboard".to_string(),
                columns: 9,
                rows: 6,
                spacing_mm: 20.0,
            },
        }
    }

    #[test]
    fn calibration_json_round_trip() {
        let value = sample_calibration();
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(
            serde_json::from_str::<CameraCalibration>(&json).unwrap(),
            value
        );
    }

    #[test]
    fn rejects_invalid_camera_matrix() {
        let node = WorkflowNode {
            id: "load".to_string(),
            operator: "calibration.camera.load".to_string(),
            params: serde_json::json!({}),
            execution: Default::default(),
        };
        let execution = ExecutionContext {
            run_id: "test".to_string(),
            records_dir: "records/test".into(),
        };
        let mut value = sample_calibration();
        value.camera_matrix[0] = 0.0;
        let ctx = context(&node, BTreeMap::new(), &execution);
        assert!(validate_calibration(&ctx, &value).is_err());
    }
}
