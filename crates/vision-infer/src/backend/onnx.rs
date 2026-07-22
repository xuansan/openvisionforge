use vision_core::{ImageFrame, Result};

#[cfg(feature = "onnxruntime")]
use crate::preprocess::PreprocessConfig;

#[derive(Debug, Clone)]
pub(crate) struct OnnxRunRequest {
    pub model_path: String,
    pub backend: InferenceBackend,
    pub input_width: i64,
    pub input_height: i64,
    pub mean: [f32; 3],
    pub std: [f32; 3],
    pub letterbox: bool,
    pub image: ImageFrame,
}

#[derive(Debug, Clone)]
pub(crate) struct OnnxRunOutput {
    pub tensors: Vec<TensorOutput>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub(crate) struct TensorOutput {
    pub name: String,
    pub shape: Vec<i64>,
    pub data: Vec<f32>,
}

#[derive(Debug, Clone)]
pub(crate) enum InferenceBackend {
    OnnxRuntime,
    Cpu,
    Cuda,
    TensorRt,
    Rknn,
    Sophon,
    Mock,
}

impl InferenceBackend {
    pub(crate) fn from_param(value: &str) -> Self {
        match value {
            "onnxruntime" => Self::OnnxRuntime,
            "cuda" => Self::Cuda,
            "tensorrt" => Self::TensorRt,
            "rknn" => Self::Rknn,
            "sophon" => Self::Sophon,
            "mock" => Self::Mock,
            _ => Self::Cpu,
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::OnnxRuntime => "onnxruntime",
            Self::Cpu => "cpu",
            Self::Cuda => "cuda",
            Self::TensorRt => "tensorrt",
            Self::Rknn => "rknn",
            Self::Sophon => "sophon",
            Self::Mock => "mock",
        }
    }
}

impl OnnxRunRequest {
    #[cfg(feature = "onnxruntime")]
    pub(crate) fn preprocess_config(&self) -> PreprocessConfig {
        PreprocessConfig {
            input_width: self.input_width,
            input_height: self.input_height,
            mean: self.mean,
            std: self.std,
            letterbox: self.letterbox,
        }
    }
}

#[cfg(feature = "onnxruntime")]
pub(crate) fn run_onnx(request: &OnnxRunRequest) -> Result<OnnxRunOutput> {
    crate::backend::onnxruntime::run_onnx(request)
}

#[cfg(not(feature = "onnxruntime"))]
pub(crate) fn run_onnx(request: &OnnxRunRequest) -> Result<OnnxRunOutput> {
    Ok(crate::backend::mock::run_onnx(request))
}
