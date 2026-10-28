use std::{
    collections::{HashMap, VecDeque},
    path::Path,
    sync::{Arc, Mutex, OnceLock},
    time::Instant,
};

#[cfg(feature = "onnxruntime-cuda")]
use ort::execution_providers::CUDAExecutionProvider;
use ort::{session::Session, value::Tensor};
use vision_core::{Result, VisionError};

use crate::{
    backend::onnx::{InferenceBackend, OnnxRunOutput, OnnxRunRequest, TensorOutput},
    preprocess,
};

pub(crate) fn run_onnx(request: &OnnxRunRequest) -> Result<OnnxRunOutput> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_onnx_inner(request))).map_err(
        |payload| {
            to_error(format!(
                "ONNX Runtime initialization panicked: {}",
                panic_message(payload)
            ))
        },
    )?
}

fn run_onnx_inner(request: &OnnxRunRequest) -> Result<OnnxRunOutput> {
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

    let (session, cache_hit, model_load_ms) = cached_session(request)?;
    let mut session = session
        .lock()
        .map_err(|_| to_error("ONNX Runtime session lock is poisoned"))?;

    let input_name = session
        .inputs
        .first()
        .map(|input| input.name.to_string())
        .unwrap_or_else(|| "images".to_string());

    let tensor = Tensor::from_array((input.shape, input.data)).map_err(to_error)?;
    let inputs = ort::inputs![input_name.as_str() => tensor];
    let infer_started = Instant::now();
    let outputs = session.run(inputs).map_err(to_error)?;
    let infer_ms = infer_started.elapsed().as_secs_f64() * 1000.0;

    let mut tensors = Vec::new();
    for (name, value) in outputs.iter() {
        let (shape, data) = value.try_extract_tensor::<f32>().map_err(to_error)?;
        tensors.push(TensorOutput {
            name: name.to_string(),
            shape: shape.iter().copied().collect(),
            data: data.to_vec(),
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
        "session": {
            "cache_hit": cache_hit,
            "model_load_ms": model_load_ms
        },
        "total_time_ms": started.elapsed().as_secs_f64() * 1000.0,
        "total_scope": "preprocess+session+inference+output_copy"
    });

    Ok(OnnxRunOutput { tensors, metadata })
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    "unknown panic payload".to_string()
}

type SharedSession = Arc<Mutex<Session>>;
struct SessionCache {
    entries: HashMap<String, SharedSession>,
    lru: VecDeque<String>,
    capacity: usize,
}
static SESSION_CACHE: OnceLock<Mutex<SessionCache>> = OnceLock::new();

fn session_capacity() -> usize {
    std::env::var("OPENVISIONFORGE_SESSION_CACHE_CAPACITY")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(4)
}

fn touch(cache: &mut SessionCache, key: &str) {
    cache.lru.retain(|item| item != key);
    cache.lru.push_back(key.to_string());
}

fn cached_session(request: &OnnxRunRequest) -> Result<(SharedSession, bool, f64)> {
    let canonical = Path::new(&request.model_path)
        .canonicalize()
        .map_err(|err| to_error(format!("invalid model path {}: {err}", request.model_path)))?;
    let key = format!("{}|{}", canonical.display(), request.backend.as_str());
    let cache = SESSION_CACHE.get_or_init(|| {
        Mutex::new(SessionCache {
            entries: HashMap::new(),
            lru: VecDeque::new(),
            capacity: session_capacity(),
        })
    });

    {
        let mut guard = cache
            .lock()
            .map_err(|_| to_error("ONNX Runtime session cache lock is poisoned"))?;
        if let Some(session) = guard.entries.get(&key).cloned() {
            touch(&mut guard, &key);
            return Ok((session, true, 0.0));
        }
    }

    let started = Instant::now();
    let session = Arc::new(Mutex::new(create_session(request)?));
    let load_ms = started.elapsed().as_secs_f64() * 1000.0;
    let mut cache = cache
        .lock()
        .map_err(|_| to_error("ONNX Runtime session cache lock is poisoned"))?;
    if let Some(entry) = cache.entries.get(&key).cloned() {
        touch(&mut cache, &key);
        return Ok((entry, true, load_ms));
    }
    cache.entries.insert(key.clone(), session.clone());
    touch(&mut cache, &key);
    while cache.entries.len() > cache.capacity {
        let Some(oldest) = cache.lru.pop_front() else {
            break;
        };
        if oldest != key {
            cache.entries.remove(&oldest);
        }
    }
    let entry = cache.entries.get(&key).cloned().unwrap_or(session);
    Ok((entry, false, load_ms))
}

fn create_session(request: &OnnxRunRequest) -> Result<Session> {
    let builder = Session::builder().map_err(to_error)?;
    let builder = match request.backend {
        InferenceBackend::Cuda => cuda_builder(builder)?,
        _ => builder,
    };
    builder
        .commit_from_file(&request.model_path)
        .map_err(to_error)
}

#[cfg(feature = "onnxruntime-cuda")]
fn cuda_builder(
    builder: ort::session::builder::SessionBuilder,
) -> Result<ort::session::builder::SessionBuilder> {
    builder
        .with_execution_providers([CUDAExecutionProvider::default().build()])
        .map_err(to_error)
}

#[cfg(not(feature = "onnxruntime-cuda"))]
fn cuda_builder(
    _builder: ort::session::builder::SessionBuilder,
) -> Result<ort::session::builder::SessionBuilder> {
    Err(to_error(
        "CUDA backend requires the vision-infer/onnxruntime-cuda Cargo feature",
    ))
}

fn to_error(error: impl std::fmt::Display) -> VisionError {
    VisionError::Operator {
        node: "infer.onnxruntime".to_string(),
        message: error.to_string(),
    }
}
