use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelAsset {
    pub id: String,
    pub name: String,
    pub version: String,
    pub task_type: ModelTaskType,
    pub runtime: ModelRuntimeKind,
    pub backend: InferenceBackendKind,
    pub artifact_uri: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelTaskType {
    Detection,
    Segmentation,
    Classification,
    Ocr,
    Sam,
    Vlm,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelRuntimeKind {
    Yolo,
    Yolo26,
    PaddleOcr,
    Sam,
    ApiVlm,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InferenceBackendKind {
    Mock,
    OnnxRuntimeCpu,
    OnnxRuntimeCuda,
    TensorRt,
    OpenVino,
    Paddle,
    Api,
}
