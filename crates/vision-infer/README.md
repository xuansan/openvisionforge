# vision-infer

`vision-infer` owns deep-learning and VLM inference nodes.

It is separate from `vision-opencv` on purpose:

```text
vision-opencv
  traditional OpenCV operators

vision-infer
  ONNX Runtime / Ultralytics / Qwen VLM / future SAM / OCR / backend adapters
```

## Naming Rule

Workflow node IDs separate model ecosystem, model version and task:

```text
infer.<ecosystem>.<model_version>.<task>
```

Examples:

```text
infer.ultralytics.yolo26.detect
infer.ultralytics.yolo11.segment
infer.qwen.qwen3_vl.inspect
```

Deployment backend is a node parameter, not part of the operator ID:

```json
{
  "backend": "onnxruntime",
  "model_id": "yolo11n-coco"
}
```

This keeps model families independent from ONNX Runtime, TensorRT, RKNN, Sophon
or other deployment runtimes.

## Current Nodes

```text
infer.onnxruntime.run

infer.yolo.detect
infer.yolo26.detect
  Backward-compatible aliases.

infer.ultralytics.yolo26.detect
infer.ultralytics.yolo26.segment
infer.ultralytics.yolo26.classify
infer.ultralytics.yolo26.pose
infer.ultralytics.yolo26.obb

infer.ultralytics.yolo11.detect
infer.ultralytics.yolo11.segment
infer.ultralytics.yolo11.classify
infer.ultralytics.yolo11.pose
infer.ultralytics.yolo11.obb

infer.ultralytics.yolov8.detect
infer.ultralytics.yolov8.segment
infer.ultralytics.yolov8.classify
infer.ultralytics.yolov8.pose
infer.ultralytics.yolov8.obb

infer.qwen.qwen3_vl.describe
infer.qwen.qwen3_vl.ocr
infer.qwen.qwen3_vl.grounding
infer.qwen.qwen3_vl.inspect

infer.qwen.qwen2_5_vl.describe
infer.qwen.qwen2_5_vl.ocr
infer.qwen.qwen2_5_vl.grounding
infer.qwen.qwen2_5_vl.inspect
```

Runnable today:

```text
Ultralytics detect through ONNX Runtime CPU with real YOLO decode and NMS
Ultralytics classification through ONNX Runtime with real score ranking
Mock inference for workflow/UI development, explicitly marked mode=mock_backend
Qwen VLM request-manifest nodes
```

Reserved today:

```text
Ultralytics segment / pose / obb task contracts; synthetic outputs are disabled
TensorRT / RKNN / Sophon backend slots
Qwen real HTTP execution
```

## Internal Layers

```text
nodes/
  Workflow-facing operator IDs, ports and params.

nodes/ultralytics/detect_pipeline.rs
  Shared Ultralytics detect execution path:
  DetectParams -> backend request -> YOLO decode -> detections/raw outputs.

backend/
  Deployment backend surfaces.
  onnxruntime is feature-gated and uses the `ort` crate.
  Sessions are cached by canonical model path and backend.
  tensorrt / rknn / sophon are reserved for later SDK integration.

preprocess.rs
  Image loading, resize, letterbox and tensor preparation for ONNX-like models.

decode/
  Task-specific postprocess.
  yolo.rs decodes tensor output into `vision_core::Detection`.
  classification.rs ranks real classification scores.

model_assets.rs
  Resolves model_path directly or model_id through models/registry.json.

schema.rs
  Shared helpers for node parameter schemas.
```

## Param Schema Metadata

`vision-core::ParamSchema` now exports UI-ready metadata:

```text
key / label / kind / required / default / description / options
group / order / advanced / unit / placeholder
min / max / step
accept / deployment_required / overridable
visible_if / validation_regex
```

This lets the UI generate parameter panels directly from runtime metadata.

Example:

```powershell
cargo run -p vision-cli -- describe-operator infer.ultralytics.yolo26.detect
cargo run -p vision-cli -- export-schema
```

## ONNX Runtime

Current dependency:

```toml
ort = { version = "=2.0.0-rc.10", optional = true, default-features = false, features = ["ndarray", "load-dynamic"] }
```

The `load-dynamic` feature means real ONNX Runtime execution loads:

```text
onnxruntime.dll
```

at runtime.

Typical setup on Windows:

```powershell
$env:ORT_DYLIB_PATH="vendor\onnxruntime\1.22.0-win-x64\onnxruntime.dll"
```

or add the DLL directory to `PATH`.

Build check:

```powershell
cargo check -p vision-cli --features vision-infer/onnxruntime-cpu
```

Run with real ONNX Runtime:

```powershell
cargo run -p vision-cli --features vision-infer/onnxruntime-cpu -- run examples/workflows/phase7_yolo11_detection_demo.json
```

The raw output includes preprocessing, inference, postprocessing and total
timings plus `session.cache_hit` and model load time. CUDA requires both the
`vision-infer/onnxruntime-cuda` feature and a CUDA-enabled ONNX Runtime binary.

The checked-in model catalog uses Ultralytics YOLO11n opset 22, so the current
runtime pair is `ort 2.0.0-rc.10` with ONNX Runtime 1.22.x.

Without the ONNX Runtime feature, `cpu`, `cuda`, and `onnxruntime` fail with a
clear configuration error. Mock output is available only when the workflow
explicitly selects `backend=mock`; real backends never silently fall back.

## Next Work

Near-term work should stay light:

```text
1. Implement and validate real segmentation mask decode and mask artifacts.
2. Implement and validate real pose and OBB decoders with task-specific models.
3. Validate the CUDA execution provider on an NVIDIA deployment environment.
4. Add session cache sizing/eviction for long-running multi-model deployments.
5. Add real Qwen VLM HTTP execution after the local detect loop is stable.
```

The priority is a real, debuggable detection loop plus clean schema metadata.
Heavy SDKs and deployment packaging should come after that.
