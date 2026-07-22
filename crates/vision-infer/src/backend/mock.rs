use crate::backend::onnx::{OnnxRunOutput, OnnxRunRequest, TensorOutput};

pub(crate) fn run_onnx(request: &OnnxRunRequest) -> OnnxRunOutput {
    let metadata = serde_json::json!({
        "runtime": "onnxruntime",
        "mode": "mock_backend",
        "backend": request.backend.as_str(),
        "model_path": request.model_path,
        "input_shape": {
            "n": 1,
            "c": 3,
            "h": request.input_height,
            "w": request.input_width
        },
        "preprocess": {
            "letterbox": request.letterbox,
            "mean": request.mean,
            "std": request.std,
            "scale": 1.0,
            "pad_x": 0.0,
            "pad_y": 0.0
        },
        "source_image": request.image,
        "note": "replace backend::mock with real ONNX Runtime session execution"
    });
    OnnxRunOutput {
        tensors: vec![TensorOutput {
            name: "output0".to_string(),
            shape: vec![1, 1, 6],
            data: vec![0.25, 0.25, 0.75, 0.75, 0.88, 0.0],
        }],
        metadata,
    }
}
