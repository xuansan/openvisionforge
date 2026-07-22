use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortTrace {
    pub port: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTrace {
    pub node_id: String,
    pub operator: String,
    pub inputs: Vec<PortTrace>,
    pub outputs: Vec<PortTrace>,
    pub status: String,
    pub metrics: ProcessingMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTrace {
    pub run_id: String,
    pub workflow_id: String,
    pub nodes: Vec<NodeTrace>,
    pub metrics: RunMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEvent {
    pub run_id: String,
    pub workflow_id: String,
    pub status: String,
    pub result: String,
    pub metrics: RunMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetrics {
    pub duration_ms: f64,
    pub input_images: u64,
    pub output_images: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    pub total_duration_ms: f64,
    pub processed_frames: u64,
    pub image_outputs: u64,
    pub fps: f64,
}
