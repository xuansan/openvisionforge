use std::time::Instant;

use ort::{Session, Tensor};
use vision_core::{Result, VisionError};

use crate::{
    backend::onnx::{InferenceBackend, OnnxRunOutput, OnnxRunRequest, TensorOutput},
    preprocess,
};

pub(crate) fn run_onnx(request: &OnnxRunRequest) -> Result<OnnxRunOutput> {
    if matches!(
        request.backend,
        InferenceBackend::TensorRt
            | InferenceBackend::Rknn
            | InferenceBackend::Sophon
            | InferenceBackend::Mock
    ) {
        return Err(VisionError::Operator {
            node: "infer.onnxruntime".to_string(),
            message: format!(
                "{} backend is planned; use backend=onnxruntime, cpu or cuda for ONNX Runtime",
                request.backend.as_str()
            ),
        });
    }

    let started = Instant::now();
    let input = preprocess::image_to_nchw(&request.image, &request.preprocess_config())?;
    let preprocess_ms = started.elapsed().as_secs_f64() * 1000.0;

    let session = Session::builder()
        .map_err(to_error)?
        .commit_from_file(&request.model_path)
        .map_err(to_error)?;

    let input_name = session
        .inputs
        .first()
        .map(|input| input.name.to_string())
        .unwrap_or_else(|| "images".to_string());

    let tensor = Tensor::from_array((input.shape, input.data)).map_err(to_error)?;
    let inputs = ort::inputs![input_name.as_str() => tensor].map_err(to_error)?;
    let infer_started = Instant::now();
    let outputs = session.run(inputs).map_err(to_error)?;
    let infer_ms = infer_started.elapsed().as_secs_f64() * 1000.0;

    let mut tensors = Vec::new();
    for (name, value) in outputs.iter() {
        let array = value.try_extract_tensor::<f32>().map_err(to_error)?;
        tensors.push(TensorOutput {
            name: name.to_string(),
            shape: array.shape().iter().map(|dim| *dim as i64).collect(),
            data: array.iter().copied().collect(),
        });
    }

    let metadata = serde_json::json!({
        "runtime": "onnxruntime",
        "mode": "real_backend",
        "backend": request.backend.as_str(),
        "model_path": request.model_path,
        "input_name": input_name,
        "input_shape": input.shape,
        "source_image": request.image,
        "preprocess": {
            "letterbox": request.letterbox,
            "scale": input.scale,
            "pad_x": input.pad_x,
            "pad_y": input.pad_y,
            "resized_width": input.resized_width,
            "resized_height": input.resized_height,
            "time_ms": preprocess_ms
        },
        "inference": {
            "time_ms": infer_ms
        },
        "total_time_ms": started.elapsed().as_secs_f64() * 1000.0
    });

    Ok(OnnxRunOutput { tensors, metadata })
}

fn to_error(error: impl std::fmt::Display) -> VisionError {
    VisionError::Operator {
        node: "infer.onnxruntime".to_string(),
        message: error.to_string(),
    }
}
