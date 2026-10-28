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
    if matches!(request.backend, InferenceBackend::Mock) {
        return Ok(crate::backend::mock::run_onnx(request));
    }
    Err(vision_core::VisionError::Operator {
        node: "infer.onnxruntime".to_string(),
        message: format!(
            "backend={} requires the vision-infer/onnxruntime-cpu or onnxruntime-cuda Cargo feature; use backend=mock only for explicit development tests",
            request.backend.as_str()
        ),
    })
}

#[cfg(all(test, not(feature = "onnxruntime")))]
mod tests {
    use super::*;

    fn request(backend: InferenceBackend) -> OnnxRunRequest {
        OnnxRunRequest {
            model_path: "models/yolo11n.onnx".to_string(),
            backend,
            input_width: 640,
            input_height: 640,
            mean: [0.0; 3],
            std: [1.0; 3],
            letterbox: true,
            image: ImageFrame {
                width: 640,
                height: 480,
                channels: 3,
                pixel_format: "RGB8".to_string(),
                source_uri: None,
            },
        }
    }

    #[test]
    fn cpu_backend_never_silently_falls_back_to_mock() {
        let error = run_onnx(&request(InferenceBackend::Cpu)).unwrap_err();
        assert!(error.to_string().contains("requires the vision-infer"));
    }

    #[test]
    fn mock_backend_requires_explicit_selection() {
        let output = run_onnx(&request(InferenceBackend::Mock)).unwrap();
        assert_eq!(output.metadata["mode"], "mock_backend");
    }
}
