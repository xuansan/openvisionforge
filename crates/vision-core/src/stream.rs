use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Metadata that follows one acquired image through a stream workflow.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrameContext {
    pub frame_id: u64,
    pub captured_at_unix_ms: u128,
    pub source_uri: String,
    pub device_id: Option<String>,
    pub trigger_id: Option<String>,
    pub batch_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamFrame {
    pub context: FrameContext,
    pub image_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackpressurePolicy {
    #[default]
    Block,
    DropNewest,
    DropOldest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamConfig {
    pub queue_capacity: usize,
    pub backpressure: BackpressurePolicy,
    pub max_frames: Option<u64>,
    pub batch_id: Option<String>,
    pub device_id: Option<String>,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 4,
            backpressure: BackpressurePolicy::Block,
            max_frames: None,
            batch_id: None,
            device_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct StreamMetrics {
    pub total_duration_ms: f64,
    pub received_frames: u64,
    pub processed_frames: u64,
    pub failed_frames: u64,
    pub dropped_frames: u64,
    pub processing_fps: f64,
    pub average_frame_ms: f64,
    pub max_queue_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamFrameResult {
    pub context: FrameContext,
    pub run_id: Option<String>,
    pub result: String,
    pub duration_ms: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StreamRunTrace {
    pub workflow_id: String,
    pub frames: Vec<StreamFrameResult>,
    pub metrics: StreamMetrics,
}
