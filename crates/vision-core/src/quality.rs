use serde::{Deserialize, Serialize};

use crate::Rect;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScaleContext {
    pub px_per_mm: Option<f64>,
    pub unit: String,
}

impl Default for ScaleContext {
    fn default() -> Self {
        Self {
            px_per_mm: None,
            unit: "px".to_string(),
        }
    }
}

impl ScaleContext {
    pub fn px_to_mm(&self, value_px: f64) -> Option<f64> {
        self.px_per_mm
            .filter(|scale| *scale > 0.0)
            .map(|scale| value_px / scale)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeasurementValue {
    pub name: String,
    pub value_px: Option<f64>,
    pub value_mm: Option<f64>,
    pub unit: String,
    pub confidence: Option<f64>,
    pub quality_score: Option<f64>,
    pub ok: Option<bool>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeasurementSet {
    pub measurements: Vec<MeasurementValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContourResult {
    pub id: String,
    pub bbox: Rect,
    pub area_px: f64,
    pub area_mm2: Option<f64>,
    pub perimeter_px: Option<f64>,
    pub perimeter_mm: Option<f64>,
    pub roundness: Option<f64>,
    pub aspect_ratio: Option<f64>,
    pub center_x: f64,
    pub center_y: f64,
    pub confidence: Option<f64>,
    pub quality_score: Option<f64>,
    pub ok: Option<bool>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContourSet {
    pub width: u32,
    pub height: u32,
    pub count: usize,
    pub contours: Vec<ContourResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocationResult {
    pub found: bool,
    pub x: f64,
    pub y: f64,
    pub angle_deg: Option<f64>,
    pub score: Option<f64>,
    pub bbox: Option<Rect>,
    pub ok: Option<bool>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityLevel {
    Ok,
    Warning,
    Ng,
    Recheck,
}

impl QualityLevel {
    pub fn as_status(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Warning => "WARNING",
            Self::Ng => "NG",
            Self::Recheck => "RECHECK",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleResult {
    pub rule_id: String,
    pub ok: bool,
    pub level: QualityLevel,
    pub reason: String,
    pub actual: serde_json::Value,
    pub expected: serde_json::Value,
    pub source_node: Option<String>,
    pub source_field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinalResult {
    pub final_ok: bool,
    pub level: QualityLevel,
    pub status: String,
    pub reason: String,
    pub failed_rules: Vec<RuleResult>,
    pub all_rules: Vec<RuleResult>,
    pub summary: serde_json::Value,
}
