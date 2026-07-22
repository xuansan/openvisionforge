# vision-infer/src

```text
lib.rs
  Registers inference node groups.

schema.rs
  Shared OperatorSchema and ParamSchema helpers.

util.rs
  Shared input extraction and parameter helpers.

backend/
  onnx.rs
  mock.rs
  onnxruntime.rs
  tensorrt.rs
  rknn.rs
  sophon.rs
  Defines deployment backend surfaces. ONNX Runtime has a real feature-gated path;
  TensorRT/RKNN/Sophon are reserved backend slots for later SDK integration.

decode/
  yolo.rs
  Converts backend tensor outputs into vision-core Detection values.

nodes/onnxruntime_run.rs
  infer.onnxruntime.run
  Stable ONNX Runtime node surface. Current mode emits a manifest-like raw output.

nodes/yolo_detect.rs
  infer.yolo.detect
  infer.yolo26.detect
  Backward-compatible aliases that call the Ultralytics detect implementation.

nodes/ultralytics/
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

  Model ecosystem nodes. The detect task is runnable through the shared ONNX/mock
  path. Segment/classify/pose/obb are registered as reserved node contracts, so
  workflow/UI can expose them before their task-specific postprocess is implemented.

nodes/qwen/
  infer.qwen.qwen3_vl.describe
  infer.qwen.qwen3_vl.ocr
  infer.qwen.qwen3_vl.grounding
  infer.qwen.qwen3_vl.inspect

  infer.qwen.qwen2_5_vl.describe
  infer.qwen.qwen2_5_vl.ocr
  infer.qwen.qwen2_5_vl.grounding
  infer.qwen.qwen2_5_vl.inspect

  Qwen VLM API node surfaces. These nodes currently prepare a structured request
  manifest and output text/result/raw. Real HTTP execution should be added behind
  the same node contract, using endpoint/api_key_env/model/prompt/image_url params.
```

## Naming rule

Inference node IDs separate model ecosystem, model version, task and backend:

```text
infer.<ecosystem>.<model_version>.<task>
```

Examples:

```text
infer.ultralytics.yolo26.detect
infer.ultralytics.yolo11.segment
infer.qwen.qwen3_vl.inspect
```

The deployment backend stays in params, not in the node path:

```json
{
  "backend": "onnxruntime",
  "model_path": "models/yolo26.onnx",
  "device": "cpu"
}
```

This keeps Ultralytics/Qwen/SAM/OCR model families independent from ONNX Runtime,
TensorRT, RKNN, Sophon or future backends.
