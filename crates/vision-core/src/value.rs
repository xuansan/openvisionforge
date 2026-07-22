use serde::{Deserialize, Serialize};

use crate::{ContourSet, FinalResult, LocationResult, MeasurementSet, RuleResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "data")]
pub enum Value {
    Image(ImageFrame),
    Mask(ImageMask),
    Regions(Vec<Region>),
    Contours(ContourSet),
    Measurements(MeasurementSet),
    Location(LocationResult),
    Detections(Vec<Detection>),
    Classifications(Vec<Classification>),
    Segmentations(Vec<SegmentationMask>),
    Poses(Vec<PoseResult>),
    OrientedBoxes(Vec<OrientedBox>),
    Rule(RuleResult),
    Rules(Vec<RuleResult>),
    Final(FinalResult),
    Trigger(TriggerInfo),
    Capture(CaptureInfo),
    Signal(SignalOutput),
    Bool(bool),
    Text(String),
    Json(serde_json::Value),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageFrame {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub pixel_format: String,
    pub source_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggerInfo {
    pub trigger_id: String,
    pub trigger_type: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaptureInfo {
    pub camera_id: String,
    pub trigger_id: Option<String>,
    pub width: u32,
    pub height: u32,
    pub pixel_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalOutput {
    pub channel: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageMask {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Region {
    pub id: String,
    pub bbox: Rect,
    pub area_px: f64,
    pub score: Option<f64>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Detection {
    pub label: String,
    pub score: f64,
    pub bbox: Rect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Classification {
    pub label: String,
    pub score: f64,
    pub rank: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SegmentationMask {
    pub id: String,
    pub label: String,
    pub score: f64,
    pub bbox: Rect,
    pub area_px: f64,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PoseKeypoint {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PoseResult {
    pub id: String,
    pub label: String,
    pub score: f64,
    pub bbox: Rect,
    pub keypoints: Vec<PoseKeypoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrientedBox {
    pub id: String,
    pub label: String,
    pub score: f64,
    pub cx: f64,
    pub cy: f64,
    pub width: f64,
    pub height: f64,
    pub angle_deg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Value {
    pub fn summary(&self) -> String {
        match self {
            Self::Image(image) => format!(
                "Image({}x{}, {}, {})",
                image.width, image.height, image.channels, image.pixel_format
            ),
            Self::Mask(mask) => format!("Mask({}x{})", mask.width, mask.height),
            Self::Regions(regions) => format!("Regions({})", regions.len()),
            Self::Contours(contours) => format!("Contours({})", contours.count),
            Self::Measurements(measurements) => {
                format!("Measurements({})", measurements.measurements.len())
            }
            Self::Location(location) => format!("Location(found={})", location.found),
            Self::Detections(detections) => format!("Detections({})", detections.len()),
            Self::Classifications(classes) => format!("Classifications({})", classes.len()),
            Self::Segmentations(masks) => format!("Segmentations({})", masks.len()),
            Self::Poses(poses) => format!("Poses({})", poses.len()),
            Self::OrientedBoxes(boxes) => format!("OrientedBoxes({})", boxes.len()),
            Self::Rule(rule) => format!("Rule({}={})", rule.rule_id, rule.ok),
            Self::Rules(rules) => format!("Rules({})", rules.len()),
            Self::Final(result) => format!("Final({})", result.status),
            Self::Trigger(trigger) => format!("Trigger({})", trigger.trigger_id),
            Self::Capture(capture) => format!("Capture({})", capture.camera_id),
            Self::Signal(signal) => format!("Signal({}={})", signal.channel, signal.value),
            Self::Bool(value) => format!("Bool({value})"),
            Self::Text(value) => format!("Text({value})"),
            Self::Json(_) => "Json".to_string(),
            Self::None => "None".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_image() {
        let value = Value::Image(ImageFrame {
            width: 640,
            height: 480,
            channels: 3,
            pixel_format: "Rgb8".to_string(),
            source_uri: None,
        });

        assert_eq!(value.summary(), "Image(640x480, 3, Rgb8)");
    }
}
