# Crates

This folder contains the Rust workspace crates.

```text
vision-core
  Stable data protocol, workflow model, operator schema, registry, executor,
  trace, event, and runtime metrics.

vision-operators
  Built-in mock operators plus basic judge, logic, record, and output nodes.
  These nodes are useful for validating workflow behavior without heavy native
  dependencies.

vision-opencv
  Real OpenCV-backed operators. The public workflow interface is Rust
  `OperatorSchema`; OpenCV is hidden behind the backend implementation.

vision-infer
  Deep-learning and VLM inference nodes. It contains ONNX Runtime integration,
  Ultralytics YOLO node contracts, Qwen VLM node contracts, and reserved backend
  slots for TensorRT, RKNN, and Sophon.

vision-io
  Record, storage, communication, database, and action-output nodes. Heavy SDKs
  should be added behind optional features instead of the default runtime.

vision-cli
  Command-line entrypoint. It registers operators, lists available nodes, and
  runs workflow JSON files.
```

## Data Flow

```text
workflow json
  -> vision-cli
  -> OperatorRegistry
  -> vision-core Executor
  -> operator function
  -> Value outputs
  -> records/<run_id>/trace.json
  -> records/<run_id>/event.json
```

## Dependency Direction

```text
vision-core       no dependency on concrete operators
vision-operators  depends on vision-core
vision-opencv     depends on vision-core
vision-infer      depends on vision-core
vision-io         depends on vision-core
vision-cli        depends on core + selected operator crates
```

Keep heavy dependencies out of `vision-core`. OpenCV, ONNX Runtime, TensorRT,
camera SDKs, databases, and UI frameworks should live in separate crates.
